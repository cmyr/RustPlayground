//! Handling core -> client updates

use std::ops::Range;

use crate::Size;

/// Represents state changes that ocurred while handling a user event.
#[derive(Debug, Clone, Default)]
pub(crate) struct Update {
    pub(crate) lines: Option<Range<usize>>,
    pub(crate) size: Option<Size>,
}

/// A type for collecting changes that occur while handling an event.
pub(crate) struct UpdateBuilder {
    inner: Update,
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

    pub(crate) fn build(self) -> Update {
        self.inner
    }
}
