extern crate libc;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

mod callbacks;
mod input_handler;
mod lines;
mod rpc;
mod update;
mod vim;

use libc::c_char;
use std::borrow::Cow;

use xi_core_lib::edit_types::{BufferEvent, EventDomain, SpecialEvent, ViewEvent};
use xi_core_lib::rpc::{EditNotification, Rect};
use xi_core_lib::selection::{InsertDrift, SelRegion, Selection};
use xi_core_lib::view::NoView;
use xi_core_lib::{edit_ops, movement, BufferConfig};
use xi_rope::breaks2::Breaks;
use xi_rope::{Cursor, LinesMetric, Rope, RopeDelta};

use callbacks::{InvalidateCallback, RpcCallback};
pub use input_handler::{EventCtx, EventPayload, Handler, KeyEvent, Plumber};
pub use lines::Size;
use lines::WidthCache;
use rpc::Rpc;
use update::{Update, UpdateBuilder};
pub use vim::Machine as Vim;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineCol {
    pub line: usize,
    pub col: usize,
}

pub struct XiCore {
    pub rpc_callback: RpcCallback,
    pub invalidate_callback: InvalidateCallback,
    pub state: OneView,
    pub plumber: Option<Plumber>,
    pub handler: Option<Box<dyn Handler>>,
}

pub struct OneView {
    selection: Selection,
    text: Rope,
    config: BufferConfig,
    breaks: Breaks,
    frame: Rect,
    width_cache: WidthCache,
    //width_measure_fn: extern "C" fn(*const c_char) -> Size,
    line_height: usize,
    content_size: Size,
}

impl OneView {
    pub fn new(width_measure_fn: extern "C" fn(*const c_char) -> Size) -> Self {
        let width_cache = WidthCache::new(width_measure_fn);
        let line_height = width_cache.measure_layout_size("a").height;
        OneView {
            selection: SelRegion::caret(0).into(),
            text: Rope::from(""),
            width_cache,
            config: BufferConfig {
                line_ending: "\n".to_string(),
                tab_size: 4,
                translate_tabs_to_spaces: true,
                use_tab_stops: true,
                font_face: "Inconsolata".into(),
                font_size: 14.0,
                auto_indent: true,
                scroll_past_end: true,
                wrap_width: 0,
                word_wrap: false,
                autodetect_whitespace: false,
                surrounding_pairs: vec![],
                save_with_newline: false,
            },
            frame: Rect::zero(),
            breaks: Breaks::default(),
            line_height,
            content_size: Size::zero(),
        }
    }

    pub fn get_line(&self, idx: usize) -> Option<(Cow<str>, i32, (i32, i32))> {
        if idx > self.text.count::<LinesMetric>(self.text.len()) {
            return None;
        }
        let start = self.text.offset_of_line(idx);
        let end = self.text.offset_of_line(idx + 1);
        let line = self.text.slice_to_cow(start..end);
        let region = self.selection.regions_in_range(start, end).first();

        let caret = match region {
            Some(region) => {
                let c = region.end;
                if (c > start && c < end)
                    || (!region.is_upstream() && c == start)
                    || (region.is_upstream() && c == end)
                    || (c == end && c == self.text.len() && self.text.line_of_offset(c) == idx)
                {
                    (c - start) as i32
                } else {
                    -1
                }
            }
            None => -1,
        };

        let line_sel =
            region.map(|r| (r.min().saturating_sub(start), r.max() - start)).unwrap_or((0, 0));
        let sel_start = line_sel.0;
        let sel_end = line_sel.1.min(line.len());
        Some((line, caret, (sel_start as i32, sel_end as i32)))
    }

    fn handle_event(&mut self, event: EventDomain) -> Update {
        let mut builder = UpdateBuilder::new();
        match event {
            EventDomain::View(event) => self.handle_view_event(event, &mut builder),
            EventDomain::Buffer(event) => self.handle_edit(event, &mut builder),
            EventDomain::Special(other) => match other {
                SpecialEvent::ViewportChange(rect) => self.viewport_change(rect),
                _other => eprintln!("unhandled special event {:?}", _other),
            },
        }
        builder.build()
    }

    fn handle_view_event(&mut self, event: ViewEvent, update: &mut UpdateBuilder) {
        if let Some(new_selection) = self.selection_for_event(event) {
            self.compute_scroll_point(&new_selection, update);
            self.selection = new_selection;
        }
        //TODO: more careful inval
        update.inval_lines(0..self.count_lines());
    }

    fn selection_for_event(&mut self, event: ViewEvent) -> Option<Selection> {
        match event {
            ViewEvent::Move(mvment) => Some(movement::selection_movement(
                mvment,
                &self.selection,
                &NoView,
                &self.text,
                false,
            )),
            ViewEvent::ModifySelection(mvment) => Some(movement::selection_movement(
                mvment,
                &self.selection,
                &NoView,
                &self.text,
                true,
            )),

            ViewEvent::SelectAll => Some(SelRegion::new(0, self.text.len()).into()),
            ViewEvent::CollapseSelections => {
                let mut region = self.selection[0].clone();
                region.start = region.end;
                Some(region.into())
            }
            _other => None,
        }
    }

    fn handle_edit(&mut self, event: BufferEvent, update: &mut UpdateBuilder) {
        if let Some(delta) = self.edit_for_event(event) {
            eprintln!("handling edit {:?}", &delta);
            let newtext = delta.apply(&self.text);
            let newsel = self.selection.apply_delta(&delta, true, InsertDrift::Default);
            self.text = newtext;
            self.update_breaks(&delta);
            self.compute_scroll_point(&newsel, update);
            self.selection = newsel;

            let newsize = self.compute_content_size();
            if newsize != self.content_size {
                //eprintln!("size changed {:?} -> {:?}", self.content_size, newsize);
                self.content_size = newsize;
                update.content_size(newsize);
            }

            //TODO: more careful inval
            update.inval_lines(0..self.count_lines());
        }
    }

    fn edit_for_event(&mut self, event: BufferEvent) -> Option<RopeDelta> {
        let text = &self.text;
        let mut kring = Rope::from("");
        match event {
            BufferEvent::Delete { movement, .. } => edit_ops::delete_by_movement(
                text,
                &NoView,
                &self.selection,
                movement,
                false,
                &mut kring,
            ),
            BufferEvent::Backspace => {
                edit_ops::delete_backward(text, &NoView, &self.selection, &self.config)
            }
            BufferEvent::Insert(chars) => Some(edit_ops::insert(text, &self.selection, chars)),
            BufferEvent::InsertNewline => Some(edit_ops::insert(text, &self.selection, "\n")),
            BufferEvent::InsertTab => Some(edit_ops::insert(text, &self.selection, "\t")),
            _other => None,
        }
    }

    fn viewport_change(&mut self, new_frame: Rect) {
        self.frame = new_frame;
        eprintln!("viewport changed to {:?}", new_frame);
    }

    fn compute_scroll_point(&self, sel: &Selection, update: &mut UpdateBuilder) {
        let end = sel.last().unwrap().end;
        let line = self.text.line_of_offset(end);
        let line_off = self.text.offset_of_line(line);
        let col = end - line_off;

        let linecol = LineCol { line, col };
        update.scroll_to(linecol)
    }

    fn update_breaks(&mut self, delta: &RopeDelta) {
        let (iv, new_len) = delta.summary();
        // first get breaks to be the right size
        let empty_breaks = Breaks::new_no_break(new_len);
        self.breaks.edit(iv, empty_breaks);

        assert_eq!(self.breaks.len(), self.text.len(), "breaks are all messed up");

        let mut cursor = Cursor::new(&self.text, iv.start());
        cursor.at_or_prev::<LinesMetric>();

        let start = cursor.pos();
        let end = cursor.next::<LinesMetric>().unwrap_or(self.text.len());
        let new_breaks = lines::rewrap_region(&self.text, start..end, &self.width_cache);
        //eprintln!("editing {}..{} {:?}", start, end, &new_breaks);
        self.breaks.edit(start..end, new_breaks);
    }

    fn compute_content_size(&self) -> Size {
        let height = self.count_lines() * self.line_height;
        let width = self.breaks.max_width();
        Size { width, height }
    }

    fn count_lines(&self) -> usize {
        self.text.count::<LinesMetric>(self.text.len()) + 1
    }
}

impl XiCore {
    pub fn new<R, I>(rpc: R, invalidate: I, state: OneView) -> Self
    where
        I: Into<InvalidateCallback>,
        R: Into<RpcCallback>,
    {
        let invalidate_callback = invalidate.into();
        let rpc_callback = rpc.into();
        XiCore { invalidate_callback, rpc_callback, state, plumber: None, handler: None }
    }

    pub fn handle_message(&mut self, msg: &str) {
        use xi_core_lib::rpc::*;
        use EditNotification as E;
        let msg: Rpc = match serde_json::from_str(msg) {
            Ok(rpc) => rpc,
            Err(e) => {
                eprintln!("invalid json '{}': {}", msg, e);
                return;
            }
        };

        eprintln!("core handle_msg {:?}", msg.method);
        let event = match msg.method {
            "insert" => E::Insert { chars: msg.params["chars"].as_str().unwrap().to_owned() },
            "viewport_change" => E::ViewportChange(msg.get_params()),
            other => match event_from_str(other) {
                Some(event) => event,
                None => {
                    eprintln!("no event for '{}'", other);
                    return;
                }
            },
        };

        let domain: EventDomain = event.into();
        let update = self.state.handle_event(domain);
        self.send_update(update);
    }

    pub fn send_update(&self, mut update: Update) {
        if let Some(newsize) = update.size.take() {
            self.rpc_callback
                .call("content_size", json!({"width": newsize.width, "height": newsize.height}));
        }
        if let Some(range) = update.lines.take() {
            self.invalidate_callback.call(range);
        }

        if let Some(scroll) = update.scroll.take() {
            self.rpc_callback.call("scroll_to", scroll)
        }
    }
}

fn event_from_str(string: &str) -> Option<EditNotification> {
    use EditNotification as E;
    match string {
        "deleteBackward:" => Some(E::DeleteBackward),
        "deleteForward:" => Some(E::DeleteForward),
        "deleteToBeginningOfLine:" => Some(E::DeleteToBeginningOfLine),
        "deleteToEndOfParagraph:" => Some(E::DeleteToEndOfParagraph),
        "deleteWordBackward:" => Some(E::DeleteWordBackward),
        "deleteWordForward:" => Some(E::DeleteForward),
        "insertNewline:" => Some(E::InsertNewline),
        "insertTab:" => Some(E::InsertTab),
        "moveBackward:" => Some(E::MoveBackward),
        "moveDown:" => Some(E::MoveDown),
        "moveDownAndModifySelection:" => Some(E::MoveDownAndModifySelection),
        "moveForward:" => Some(E::MoveForward),
        "moveLeft:" => Some(E::MoveLeft),
        "moveLeftAndModifySelection:" => Some(E::MoveLeftAndModifySelection),
        "moveRight:" => Some(E::MoveRight),
        "moveRightAndModifySelection:" => Some(E::MoveRightAndModifySelection),
        "moveToBeginningOfDocument:" => Some(E::MoveToBeginningOfDocument),
        "moveToBeginningOfDocumentAndModifySelection:" => {
            Some(E::MoveToBeginningOfDocumentAndModifySelection)
        }
        "moveToBeginningOfLine:" => Some(E::MoveToLeftEndOfLine),
        "moveToBeginningOfLineAndModifySelection:" => {
            Some(E::MoveToLeftEndOfLineAndModifySelection)
        }
        "moveToBeginningOfParagraph:" => Some(E::MoveToBeginningOfParagraph),
        "moveToBeginningOfParagraphAndModifySelection:" => {
            Some(E::MoveToBeginningOfParagraphAndModifySelection)
        }
        "moveToEndOfDocument:" => Some(E::MoveToEndOfDocument),
        "moveToEndOfDocumentAndModifySelection:" => Some(E::MoveToEndOfDocumentAndModifySelection),
        "moveToEndOfLine:" => Some(E::MoveToRightEndOfLine),
        "moveToEndOfLineAndModifySelection:" => Some(E::MoveToRightEndOfLineAndModifySelection),
        "moveToEndOfParagraph:" => Some(E::MoveToEndOfParagraph),
        "moveToEndOfParagraphAndModifySelection:" => {
            Some(E::MoveToEndOfParagraphAndModifySelection)
        }
        "moveToLeftEndOfLine:" => Some(E::MoveToLeftEndOfLine),
        "moveToLeftEndOfLineAndModifySelection:" => Some(E::MoveToLeftEndOfLineAndModifySelection),
        "moveToRightEndOfLine:" => Some(E::MoveToRightEndOfLine),
        "moveToRightEndOfLineAndModifySelection:" => {
            Some(E::MoveToRightEndOfLineAndModifySelection)
        }
        "moveUp:" => Some(E::MoveUp),
        "moveUpAndModifySelection:" => Some(E::MoveUpAndModifySelection),
        "moveWordLeft:" => Some(E::MoveWordLeft),
        "moveWordLeftAndModifySelection:" => Some(E::MoveWordLeftAndModifySelection),
        "moveWordRight:" => Some(E::MoveWordRight),
        "moveWordRightAndModifySelection:" => Some(E::MoveWordRightAndModifySelection),
        "pageDownAndModifySelection:" => Some(E::PageDownAndModifySelection),
        "pageUpAndModifySelection:" => Some(E::PageUpAndModifySelection),
        "transpose:" => Some(E::Transpose),
        "selectAll:" => Some(E::SelectAll),
        "cancelOperation:" => Some(E::CollapseSelections),
        _other => None,
        //(Some("scrollPageDown:"), None) => E::ScrollPageDown
        //(Some("scrollPageUp:"), None) =>
        //(Some("scrollToBeginningOfDocument:"), None) =>
        //(Some("scrollToEndOfDocument:"), None) =>
    }
}
// okay so:
//
// we're going to take some message from our client (a selector)
// and then we're going to convert that to some command we can handle
// and send that to core.
//
// the trixy bit is that we want to figure out how the actual update is
// going to get handled by the client...
//
//
// Okay so that's cool, but now we need to figure out our strategy for updating views.
// What makes sense for this?
//
// We don't want to get too fancy and cool.
// AT ALL
//
// we've changed the document. now we want to change the selections and update the linebreaks (?)
// and then return some representation of the state.
//
//
// Okay so: ignore the linebreaks.
// we just want to update selections
//
// in fact, maybe we can just own selections and not a whole view thingie?
//
//
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_smoke_test() {
        let rpc_json = json!({"method": "hello", "params": {"foo": "bar"}});
        let rpc_str = serde_json::to_string(&rpc_json).unwrap();
        let rpc: Rpc = serde_json::from_str(&rpc_str).unwrap();
        assert_eq!(rpc.method, "hello");
        assert_eq!(rpc.params.as_object().unwrap().get("foo").unwrap().as_str(), Some("bar"));
    }
}
