extern crate libc;
#[macro_use]
extern crate serde_json;

mod input_handler;
mod vim;

use std::borrow::Cow;
use libc::{c_char, size_t};

use xi_core_lib::edit_types::{BufferEvent, EventDomain, ViewEvent};
use xi_core_lib::selection::{InsertDrift, SelRegion, Selection};
use xi_core_lib::view::NoView;
use xi_core_lib::{edit_ops, movement, rpc::EditNotification, BufferConfig};
use xi_rope::{Interval, LinesMetric, Rope, RopeDelta};

pub use input_handler::{EventCtx, EventPayload, Handler, KeyEvent, Plumber};
pub use vim::Machine as Vim;

pub struct XiCore {
    pub rpc_callback: extern "C" fn(*const c_char),
    pub invalidate_callback: extern "C" fn(size_t, size_t),
    pub state: OneView,
    pub plumber: Option<Plumber>,
    pub handler: Option<Box<dyn Handler>>,
}

pub struct OneView {
    selection: Selection,
    text: Rope,
    config: BufferConfig,
}

impl OneView {
    pub fn new() -> Self {
        OneView {
            selection: SelRegion::caret(0).into(),
            text: Rope::from(""),
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

        let line_sel = region
            .map(|r| (r.min().saturating_sub(start), r.max() - start))
            .unwrap_or((0, 0));
        let sel_start = line_sel.0;
        let sel_end = line_sel.1.min(line.len());
        Some((line, caret, (sel_start as i32, sel_end as i32)))
    }

    fn handle_event(&mut self, event: EventDomain) -> Option<Interval> {
        match event {
            EventDomain::View(event) => self.handle_view_event(event),
            EventDomain::Buffer(event) => self.handle_edit(event),
            EventDomain::Special(_other) => None,
        }
    }

    fn handle_view_event(&mut self, event: ViewEvent) -> Option<Interval> {
        let new_selection = self.selection_for_event(event)?;
        let change = self.selection.interval().union(new_selection.interval());
        self.selection = new_selection;
        Some(change)
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

    fn handle_edit(&mut self, event: BufferEvent) -> Option<Interval> {
        let delta = self.edit_for_event(event)?;
        eprintln!("handling edit {:?}", &delta);
        let newtext = delta.apply(&self.text);
        let newsel = self
            .selection
            .apply_delta(&delta, true, InsertDrift::Default);
        self.text = newtext;
        self.selection = newsel;
        None
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

    fn count_lines(&self) -> usize {
        self.text.count::<LinesMetric>(self.text.len()) + 1
    }
}

impl XiCore {
    pub fn handle_message(&mut self, msg: &str) {
        use xi_core_lib::rpc::*;
        use EditNotification as E;

        eprintln!("core handle_msg {:?}", msg);
        let event = match msg {
            m if m.starts_with("insert ") => E::Insert {
                chars: msg["insert ".len()..].into(),
            },
            _else => match event_from_str(_else) {
                Some(event) => event,
                None => {
                    eprintln!("no event for '{}'", _else);
                    return;
                }
            },
        };

        let domain: EventDomain = event.into();
        self.state.handle_event(domain);
        self.send_update();
    }

    pub fn send_update(&self) {
        let n_lines = self.state.count_lines();
        (self.invalidate_callback)(0, n_lines)
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
