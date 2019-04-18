//! State for a single view/document

use libc::c_char;
use std::borrow::Cow;

use xi_core_lib::edit_types::{BufferEvent, EventDomain, SpecialEvent, ViewEvent};
use xi_core_lib::rpc::{GestureType, Rect};
use xi_core_lib::selection::{InsertDrift, SelRegion, Selection};
use xi_core_lib::view::ViewMovement;
use xi_core_lib::{edit_ops, movement, BufferConfig};
use xi_rope::breaks2::{Breaks, BreaksMetric};
use xi_rope::spans::Spans;
use xi_rope::{LinesMetric, Rope, RopeDelta};

use crate::gesture::{DragState, GestureContext};
use crate::highlighting::HighlightState;
use crate::lines::Size;
use crate::lines::WidthCache;
use crate::style::StyleId;
use crate::update::{Update, UpdateBuilder};

const ENABLE_WORDWRAP: bool = true;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineCol {
    pub line: usize,
    pub col: usize,
}

pub struct Line<'a> {
    pub line: Cow<'a, str>,
    pub caret: Option<usize>,
    pub selection: (usize, usize),
    /// ranges and ids, as (start, len, id) triplets.
    pub styles: Vec<usize>,
}

pub struct OneView {
    selection: Selection,
    text: Rope,
    drag_state: Option<DragState>,
    config: BufferConfig,
    breaks: Breaks,
    spans: Spans<StyleId>,
    highlighter: HighlightState,
    frame: Rect,
    width_cache: WidthCache,
    line_height: usize,
    /// The total size of the document, in logical pixels
    content_size: Size,
}

impl OneView {
    pub fn new(width_measure_fn: extern "C" fn(*const c_char) -> Size) -> Self {
        let width_cache = WidthCache::new(width_measure_fn);
        let line_height = width_cache.measure_layout_size("a").height;
        OneView {
            selection: SelRegion::caret(0).into(),
            text: Rope::from(""),
            drag_state: None,
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
                word_wrap: ENABLE_WORDWRAP,
                autodetect_whitespace: false,
                surrounding_pairs: vec![],
                save_with_newline: false,
            },
            frame: Rect::zero(),
            breaks: Breaks::default(),
            spans: Spans::default(),
            highlighter: HighlightState::new(),
            line_height,
            content_size: Size::zero(),
        }
    }

    fn offset_of_line(&self, line: usize) -> usize {
        match line {
            n if n >= self.count_lines() => self.text.len(),
            _else => self.breaks.count_base_units::<BreaksMetric>(_else),
        }
    }

    fn line_of_offset(&self, offset: usize) -> usize {
        let offset = offset.min(self.text.len());
        self.breaks.count::<BreaksMetric>(offset)
    }

    pub fn get_line<'a>(&'a self, idx: usize) -> Option<Line<'a>> {
        if idx == 6942069 {
            return Some(self.whole_thing());
        }
        if idx > self.count_lines() {
            return None;
        }
        let start = self.offset_of_line(idx);
        let end = self.offset_of_line(idx + 1);
        let line = self.text.slice_to_cow(start..end);
        let spans = self.spans.subseq(start..end);

        let region = self.selection.regions_in_range(start, end).first();

        let caret = match region {
            Some(region) => {
                let c = region.end;
                if (c > start && c < end)
                    || (!region.is_upstream() && c == start)
                    || (region.is_upstream() && c == end)
                    || (c == end && c == self.text.len() && self.line_of_offset(c) == idx)
                {
                    Some(c - start)
                } else {
                    None
                }
            }
            None => None,
        };

        let line_sel =
            region.map(|r| (r.min().saturating_sub(start), r.max() - start)).unwrap_or((0, 0));
        let selection = (line_sel.0, line_sel.1.min(line.len()));

        let styles = spans.iter().fold(Vec::new(), |mut v, (iv, style)| {
            v.extend([iv.start(), iv.end() - iv.start(), *style as usize].into_iter());
            v
        });

        Some(Line { line, caret, selection, styles })
    }

    /// returns the whole document
    fn whole_thing(&self) -> Line<'static> {
        let mut text = self.text.to_string();
        text.push('\n');
        Line { line: Cow::Owned(text), caret: None, selection: (0, 0), styles: Vec::new() }
    }

    pub(crate) fn handle_event(&mut self, event: EventDomain) -> Update {
        let mut builder = UpdateBuilder::new();
        match event {
            EventDomain::View(event) => self.handle_view_event(event, &mut builder),
            EventDomain::Buffer(event) => self.handle_edit(event, &mut builder),
            EventDomain::Special(other) => match other {
                SpecialEvent::ViewportChange(rect) => self.viewport_change(rect, &mut builder),
                _other => eprintln!("unhandled special event {:?}", _other),
            },
        }
        builder.build()
    }

    fn handle_view_event(&mut self, event: ViewEvent, update: &mut UpdateBuilder) {
        if let ViewEvent::Copy = event {
            if let Some(s) = edit_ops::extract_sel_regions(&self.text, &self.selection) {
                update.set_pasteboard(s.to_string());
            }
            return;
        }

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
                &self.breaks,
                &self.text,
                false,
            )),
            ViewEvent::ModifySelection(mvment) => Some(movement::selection_movement(
                mvment,
                &self.selection,
                &self.breaks,
                &self.text,
                true,
            )),

            ViewEvent::SelectAll => Some(SelRegion::new(0, self.text.len()).into()),
            ViewEvent::CollapseSelections => {
                let mut region = self.selection[0].clone();
                region.start = region.end;
                Some(region.into())
            }
            ViewEvent::Gesture { line, col, ty } => self.handle_gesture(line, col, ty),
            _other => None,
        }
    }

    fn handle_gesture(&mut self, line: u64, col: u64, ty: GestureType) -> Option<Selection> {
        let line = line as usize;
        let col = col as usize;
        let offset = self.breaks.line_col_to_offset(&self.text, line, col);
        let mut ctx = GestureContext::new(&self.text, &self.selection, &mut self.drag_state);
        let new_sel = ctx.selection_for_gesture(offset, ty);
        Some(new_sel)
    }

    fn handle_edit(&mut self, event: BufferEvent, update: &mut UpdateBuilder) {
        if let BufferEvent::Cut = event {
            if let Some(s) = edit_ops::extract_sel_regions(&self.text, &self.selection) {
                update.set_pasteboard(s.to_string());
            } else {
                // cut with no non-caret selections is a noop
                return;
            }
        }

        if let Some(delta) = self.edit_for_event(event) {
            eprintln!("handling edit {:?}", &delta);
            let newtext = delta.apply(&self.text);
            let newsel = self.selection.apply_delta(&delta, true, InsertDrift::Default);
            self.text = newtext;
            let view_width = if self.config.word_wrap { Some(self.frame.width) } else { None };
            self.rewrap_all(view_width);
            self.update_spans(&delta);

            if let Some(styles) = self.highlighter.take_new_styles() {
                update.new_styles(styles);
            }

            self.compute_scroll_point(&newsel, update);
            self.selection = newsel;

            let newsize = self.compute_content_size();
            if newsize != self.content_size {
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
                &self.breaks,
                &self.selection,
                movement,
                false,
                &mut kring,
            ),
            BufferEvent::Backspace => {
                edit_ops::delete_backward(text, &self.breaks, &self.selection, &self.config)
            }
            BufferEvent::Cut => {
                debug_assert!(self.selection.iter().all(|s| !s.is_caret()));
                edit_ops::delete_backward(text, &self.breaks, &self.selection, &self.config)
            }
            BufferEvent::Insert(chars) => Some(edit_ops::insert(text, &self.selection, chars)),
            BufferEvent::InsertNewline => Some(edit_ops::insert(text, &self.selection, "\n")),
            BufferEvent::InsertTab => Some(edit_ops::insert(text, &self.selection, "\t")),
            _other => None,
        }
    }

    fn update_spans(&mut self, _delta: &RopeDelta) {
        self.spans = self.highlighter.highlight_all(&self.text);
    }

    fn viewport_change(&mut self, new_frame: Rect, update: &mut UpdateBuilder) {
        if self.config.word_wrap && new_frame.width != self.frame.width {
            self.rewrap_all(new_frame.width);
            update.inval_lines(0..self.count_lines());

            let newsize = self.compute_content_size();
            if newsize != self.content_size {
                self.content_size = newsize;
                update.content_size(newsize);
            }
        }

        self.frame = new_frame;
        //eprintln!("viewport changed to {:?}", new_frame);
    }

    fn compute_scroll_point(&self, sel: &Selection, update: &mut UpdateBuilder) {
        let end = sel.last().expect("compute_scroll no selection?").end;
        let line = self.line_of_offset(end);
        let line_off = self.offset_of_line(line);
        let col = end - line_off;

        let linecol = LineCol { line, col };
        update.scroll_to(linecol)
    }

    fn rewrap_all<T: Into<Option<usize>>>(&mut self, new_width: T) {
        //eprintln!("old breaks {:?}", &self.breaks);
        self.breaks = crate::lines::rewrap_region(&self.text, .., &self.width_cache, new_width);
        //eprintln!("NEW breaks {:?}", &self.breaks);
    }

    //fn update_breaks(&mut self, delta: &RopeDelta) {
    //let (iv, new_len) = delta.summary();
    //// first get breaks to be the right size
    //let empty_breaks = Breaks::new_no_break(new_len, 0);
    //self.breaks.edit(iv, empty_breaks);

    //assert_eq!(self.breaks.len(), self.text.len(), "breaks are all messed up iv {:?}", iv);

    //let mut cursor = Cursor::new(&self.text, iv.start());
    //cursor.at_or_prev::<LinesMetric>();

    //let start = cursor.pos();
    //let end = cursor.next::<LinesMetric>().unwrap_or(self.text.len());
    //let view_width = if self.config.word_wrap { Some(self.frame.width) } else { None };
    //let new_breaks =
    //lines::rewrap_region(&self.text, start..end, &self.width_cache, view_width);
    //eprintln!("editing {}..{} {:?}", start, end, &new_breaks);
    //self.breaks.edit(start..end, new_breaks);
    //eprintln!("NEW {:?}", &self.breaks);
    //}

    fn compute_content_size(&self) -> Size {
        let height = self.count_lines() * self.line_height;
        let width = self.breaks.max_width();
        Size { width, height }
    }

    fn count_lines(&self) -> usize {
        if self.config.word_wrap {
            self.breaks.count::<BreaksMetric>(self.breaks.len()) + 1
        } else {
            self.text.count::<LinesMetric>(self.text.len()) + 1
        }
    }
}
