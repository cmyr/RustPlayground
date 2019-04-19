//! State for a single view/document

use libc::c_char;
use std::borrow::Cow;
use std::ops::Range;

use xi_core_lib::edit_types::{BufferEvent, EventDomain, SpecialEvent, ViewEvent};
use xi_core_lib::rpc::{GestureType, Rect};
use xi_core_lib::selection::{InsertDrift, SelRegion, Selection};
use xi_core_lib::view::ViewMovement;
use xi_core_lib::{edit_ops, movement, BufferConfig};
use xi_rope::breaks2::{Breaks, BreaksMetric};
use xi_rope::spans::Spans;
use xi_rope::{DeltaBuilder, Interval, LinesMetric, Rope, RopeDelta, RopeInfo};

use crate::gesture::{DragState, GestureContext};
use crate::highlighting::{lines_for_selection, HighlightState};
use crate::lines::Size;
use crate::lines::WidthCache;
use crate::style::StyleId;
use crate::undo::{EditType, UndoStack, ViewUndo};
use crate::update::{Update, UpdateBuilder};

const ENABLE_WORDWRAP: bool = true;

type RopeDeltaBuilder = DeltaBuilder<RopeInfo>;

type LineRange = Range<usize>;

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
    undo_stack: UndoStack<ViewUndo>,
    last_edit: EditType,
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
        let selection: Selection = SelRegion::caret(0).into();
        let text = Rope::from("");
        let init_undo = ViewUndo::new(text.clone(), selection.clone(), selection.clone());
        OneView {
            drag_state: None,
            text,
            selection,
            undo_stack: UndoStack::new(init_undo),
            last_edit: EditType::Other,
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
            // we always break undo if the user does a move/gesture
            self.last_edit = EditType::Other;
            self.undo_stack.update_current_undo(|undo| undo.sel_before = new_selection.clone());
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

        let this_edit_type = EditType::from_event(&event);
        let mut edit_delta: Option<RopeDelta> = None;

        match event {
            BufferEvent::Undo => {
                if let Some(undo_state) = self.undo_stack.undo() {
                    self.text = undo_state.text.clone();
                    self.selection = undo_state.sel_before.clone();
                } else {
                    return;
                }
            }
            BufferEvent::Redo => {
                if let Some(redo_state) = self.undo_stack.redo() {
                    self.text = redo_state.text.clone();
                    self.selection = redo_state.sel_after.clone();
                } else {
                    return;
                }
            }
            other => {
                let delta = match self.edit_for_event(other) {
                    Some(d) => d,
                    None => return,
                };
                let newtext = delta.apply(&self.text);
                let newsel = self.selection.apply_delta(&delta, true, InsertDrift::Default);
                if this_edit_type.breaks_undo_group(self.last_edit) {
                    let new_undo = ViewUndo::new(newtext.clone(), newsel.clone(), newsel.clone());
                    self.undo_stack.add_undo_group(new_undo);
                } else {
                    self.undo_stack.update_current_undo(|undo| {
                        undo.text = newtext.clone();
                        undo.sel_after = newsel.clone();
                        undo.sel_before = newsel.clone();
                    })
                }
                self.text = newtext;
                self.selection = newsel;
                edit_delta = Some(delta);
                self.last_edit = this_edit_type;
            }
        }

        let view_width = if self.config.word_wrap { Some(self.frame.width) } else { None };

        //TODO: do we need to update spans before we compute autoindent?
        // let's try not first :o
        if let Some(delta) = edit_delta.take() {
            if let Some(indent_delta) = self.auto_indent(&delta, this_edit_type) {
                self.text = indent_delta.apply(&self.text);
                let new_sel = self.selection.apply_delta(&indent_delta, true, InsertDrift::Default);
                self.selection = new_sel;
            }
        }

        self.rewrap_all(view_width);
        self.update_spans();

        if let Some(styles) = self.highlighter.take_new_styles() {
            update.new_styles(styles);
        }

        self.compute_scroll_point(&self.selection, update);

        let newsize = self.compute_content_size();
        if newsize != self.content_size {
            self.content_size = newsize;
            update.content_size(newsize);
        }

        //TODO: more careful inval
        update.inval_lines(0..self.count_lines());
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
            BufferEvent::InsertTab => {
                let (delta, _) = edit_ops::insert_tab(text, &self.selection, &self.config);
                Some(delta)
            }
            BufferEvent::ToggleComment => self.toggle_comment(),
            BufferEvent::Indent => Some(self.modify_indent(true)),
            BufferEvent::Outdent => Some(self.modify_indent(false)),
            _other => {
                eprintln!("unhandled edit event {:?}", _other);
                None
            }
        }
    }

    fn modify_indent(&self, increase: bool) -> RopeDelta {
        let lines = lines_for_selection(&self.text, &self.selection);
        let tab_text = if self.config.translate_tabs_to_spaces {
            n_spaces(self.config.tab_size)
        } else {
            "\t"
        };
        if increase {
            edit_ops::indent(&self.text, lines.iter().cloned().flatten(), tab_text)
        } else {
            edit_ops::outdent(&self.text, lines.iter().cloned().flatten(), tab_text)
        }
    }

    fn auto_indent(&mut self, delta: &RopeDelta, edit_type: EditType) -> Option<RopeDelta> {
        let mut builder = RopeDeltaBuilder::new(self.text.len());
        for region in delta.iter_inserts() {
            let line = self.line_of_offset(region.new_offset);
            match edit_type {
                EditType::InsertNewline => self.indent_after_newline(line + 1, &mut builder),
                EditType::InsertChars => {
                    let start = region.new_offset;
                    let end = region.new_offset + region.len;
                    if !self
                        .text
                        .slice_to_cow(start..end)
                        .as_bytes()
                        .iter()
                        .all(u8::is_ascii_whitespace)
                    {
                        self.indent_after_insert(line, &mut builder);
                    }
                }
                _ => (),
            }
        }
        if builder.is_empty() {
            None
        } else {
            Some(builder.build())
        }
    }

    fn indent_after_newline(&mut self, new_line_num: usize, builder: &mut RopeDeltaBuilder) {
        let tab_size = self.config.tab_size;
        let current_indent = self.indent_level_of_line(new_line_num);
        let base_indent = self
            .previous_nonblank_line(new_line_num)
            .map(|l| self.indent_level_of_line(l))
            .unwrap_or(0);

        let increase_level = self.test_increase(new_line_num);
        let decrease_level = self.test_decrease(new_line_num);
        let increase = if increase_level { tab_size } else { 0 };
        let decrease = if decrease_level { tab_size } else { 0 };
        let final_level = base_indent + increase - decrease;
        if final_level != current_indent {
            self.set_indent(builder, new_line_num, final_level);
        }
    }

    fn indent_after_insert(&mut self, line: usize, builder: &mut RopeDeltaBuilder) {
        let tab_size = self.config.tab_size;
        let current_indent = self.indent_level_of_line(line);
        if line == 0 || current_indent == 0 {
            return;
        }

        let just_increased = self.test_increase(line);
        let decrease = self.test_decrease(line);
        let prev_line = self.previous_nonblank_line(line);
        let mut indent_level = prev_line.map(|l| self.indent_level_of_line(l)).unwrap_or(0);
        if decrease {
            // the first line after an increase should just match the previous line
            if !just_increased {
                indent_level = indent_level.saturating_sub(tab_size);
            }
            // we don't want to change indent level if this line doesn't
            // match `test_decrease`, because the user could have changed
            // it manually, and we respect that.
            if indent_level != current_indent {
                self.set_indent(builder, line, indent_level);
            }
        }
    }

    fn set_indent(&self, builder: &mut RopeDeltaBuilder, line: usize, level: usize) {
        let edit_start = self.offset_of_line(line);
        let edit_len = {
            let line = self.get_line_str(line);
            line.as_bytes().iter().take_while(|b| **b == b' ' || **b == b'\t').count()
        };

        let use_spaces = self.config.translate_tabs_to_spaces;
        let tab_size = self.config.tab_size;

        let indent_text = if use_spaces { n_spaces(level) } else { n_tabs(level / tab_size) };

        let iv = Interval::new(edit_start, edit_start + edit_len);
        builder.replace(iv, indent_text.into());
    }

    fn indent_level_of_line(&self, line_num: usize) -> usize {
        let tab_size = self.config.tab_size;
        let line = self.get_line_str(line_num);
        line.as_bytes()
            .iter()
            .take_while(|b| **b == b' ' || **b == b'\t')
            .map(|b| if b == &b' ' { 1 } else { tab_size })
            .sum()
    }

    fn previous_nonblank_line(&self, mut line_num: usize) -> Option<usize> {
        debug_assert!(line_num > 0);
        while line_num > 0 {
            line_num -= 1;
            let line = self.get_line_str(line_num);
            if !line.bytes().all(|b| b.is_ascii_whitespace()) {
                return Some(line_num);
            }
        }
        None
    }

    /// Test whether the indent level should be increased for this line,
    /// by testing the _previous_ line against a regex.
    fn test_increase(&self, line: usize) -> bool {
        debug_assert!(line > 0, "increasing indent requires a previous line");
        let prev_line = match self.previous_nonblank_line(line) {
            Some(l) => l,
            None => return false,
        };
        let metadata = self.highlighter.metadata_for_line(prev_line);
        let line = self.get_line_str(prev_line);
        metadata.increase_indent(&line)
    }

    /// Test whether the indent level for this line should be decreased, by
    /// checking this line against a regex.
    fn test_decrease(&mut self, line: usize) -> bool {
        assert!(line <= self.count_lines());
        if line == 0 || line == self.count_lines() {
            return false;
        }
        let metadata = self.highlighter.metadata_for_line(line);
        let line = self.get_line_str(line);
        metadata.decrease_indent(&line)
    }

    fn toggle_comment(&self) -> Option<RopeDelta> {
        let mut builder = RopeDeltaBuilder::new(self.text.len());
        let line_ranges = lines_for_selection(&self.text, &self.selection);
        for range in line_ranges {
            self.toggle_comment_line_range(range, &mut builder);
        }
        if builder.is_empty() {
            None
        } else {
            Some(builder.build())
        }
    }

    fn toggle_comment_line_range(&self, range: LineRange, builder: &mut RopeDeltaBuilder) {
        let metadata = self.highlighter.metadata_for_line(range.start);
        let comment_str = match metadata.line_comment() {
            Some(s) => s,
            None => return,
        };

        let line = self.get_line_str(range.start);
        if line.trim() == comment_str.trim() || line.trim().starts_with(&comment_str) {
            self.remove_comment(range, comment_str, builder);
        } else {
            self.add_comment(range, comment_str, builder);
        }
    }

    fn remove_comment(&self, range: LineRange, comment_str: &str, builder: &mut RopeDeltaBuilder) {
        for num in range {
            let offset = self.offset_of_line(num);
            let line = self.get_line_str(num);
            let (comment_start, len) = match line.find(&comment_str) {
                Some(off) => (offset + off, comment_str.len()),
                None if line.trim() == comment_str.trim() => (offset, comment_str.trim().len()),
                None => continue,
            };

            let iv = Interval::new(comment_start, comment_start + len);
            builder.delete(iv);
        }
    }

    fn add_comment(&self, range: LineRange, comment_str: &str, builder: &mut RopeDeltaBuilder) {
        // when commenting out multiple lines, we insert all comment markers at
        // the same indent level: that of the least indented line.
        let line_offset = range
            .clone()
            .map(|num| {
                let line = self.get_line_str(num);
                line.as_bytes().iter().position(|b| *b != b' ' && *b != b'\t').unwrap_or(0)
            })
            .min()
            .unwrap_or(0);

        let comment_txt = Rope::from(&comment_str);
        for num in range {
            let offset = self.offset_of_line(num);
            let line = self.get_line_str(num);
            if line.trim().starts_with(&comment_str) {
                continue;
            }

            let iv = Interval::new(offset + line_offset, offset + line_offset);
            builder.replace(iv, comment_txt.clone());
        }
    }

    fn update_spans(&mut self) {
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

    fn get_line_str(&self, line_num: usize) -> Cow<str> {
        let line_start = self.text.offset_of_line(line_num);
        let line_end = self.text.offset_of_line(line_num + 1);
        self.text.slice_to_cow(line_start..line_end)
    }

    fn count_lines(&self) -> usize {
        if self.config.word_wrap {
            self.breaks.count::<BreaksMetric>(self.breaks.len()) + 1
        } else {
            self.text.count::<LinesMetric>(self.text.len()) + 1
        }
    }
}

fn n_spaces(n: usize) -> &'static str {
    // when someone opens an issue complaining about this we know we've made it
    const MAX_SPACES: usize = 160;
    static MANY_SPACES: [u8; MAX_SPACES] = [b' '; MAX_SPACES];
    unsafe { ::std::str::from_utf8_unchecked(&MANY_SPACES[..n.min(MAX_SPACES)]) }
}

fn n_tabs(n: usize) -> &'static str {
    const MAX_TABS: usize = 40;
    static MANY_TABS: [u8; MAX_TABS] = [b'\t'; MAX_TABS];
    unsafe { ::std::str::from_utf8_unchecked(&MANY_TABS[..n.min(MAX_TABS)]) }
}
