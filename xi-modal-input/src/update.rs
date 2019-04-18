//! Handling core -> client updates

use std::ops::Range;

use crate::lines::Size;
use crate::style::{Style, StyleId};
use crate::view::LineCol;

/// Represents state changes that ocurred while handling a user event.
#[derive(Debug, Clone, Default)]
pub struct Update {
    pub(crate) lines: Option<Range<usize>>,
    pub(crate) size: Option<Size>,
    pub(crate) scroll: Option<LineCol>,
    pub(crate) styles: Option<Vec<(StyleId, Style)>>,
    pub(crate) pasteboard: Option<String>,
}

/// A type for collecting changes that occur while handling an event.
pub(crate) struct UpdateBuilder {
    pub(crate) inner: Update,
}

impl UpdateBuilder {
    pub(crate) fn new() -> Self {
        UpdateBuilder { inner: Update::default() }
    }

    /// mark a range of lines as invalid
    pub(crate) fn inval_lines(&mut self, range: Range<usize>) {
        self.inner.lines = match self.inner.lines.as_ref() {
            Some(prev) => Some(prev.start.min(range.start)..prev.end.max(range.end)),
            None => Some(range),
        }
    }

    /// indicate that content size has changed
    pub(crate) fn content_size(&mut self, new_size: Size) {
        self.inner.size = Some(new_size);
    }

    pub(crate) fn scroll_to(&mut self, point: LineCol) {
        self.inner.scroll = Some(point);
    }

    pub(crate) fn new_styles(&mut self, styles: Vec<(StyleId, Style)>) {
        debug_assert!(self.inner.styles.is_none(), "styles should be None");
        self.inner.styles = Some(styles);
    }

    /// This text will be sent to the client's system pasteboard.
    pub(crate) fn set_pasteboard(&mut self, text: String) {
        self.inner.pasteboard = text.into()
    }

    pub(crate) fn build(self) -> Update {
        self.inner
    }
}
