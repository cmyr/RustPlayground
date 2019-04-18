//! Tracking undo state

use std::collections::VecDeque;
use xi_core_lib::edit_types::BufferEvent;
use xi_core_lib::selection::Selection;
use xi_rope::Rope;

const DEFAULT_UNDO_STACK_SIZE: usize = 40;

/// Undo state that is aware of user selections
pub(crate) struct ViewUndo {
    pub(crate) text: Rope,
    /// The selection state to be restored when this group is undone.
    /// This is `None` if it would be equal to the previous group's `after`.
    pub(crate) sel_before: Selection,
    /// The selection to be restored when this group is redone.
    pub(crate) sel_after: Selection,
}

impl ViewUndo {
    pub(crate) fn new(text: Rope, sel_before: Selection, sel_after: Selection) -> Self {
        ViewUndo { text, sel_before, sel_after }
    }
}

#[derive(Debug)]
pub(crate) struct UndoStack<T> {
    max_undo_count: usize,
    stack: VecDeque<T>,
    /// The index in `stack` of the current document.
    live_index: usize,
}

impl<T> UndoStack<T> {
    pub(crate) fn new(init_state: T) -> Self {
        Self::new_sized(DEFAULT_UNDO_STACK_SIZE, init_state)
    }

    fn new_sized(max_undo_count: usize, init_state: T) -> Self {
        let mut stack = VecDeque::new();
        stack.push_back(init_state);
        UndoStack { max_undo_count, stack, live_index: 0 }
    }

    pub(crate) fn undo(&mut self) -> Option<&T> {
        if self.live_index == 0 {
            return None;
        }
        self.live_index -= 1;
        self.stack.get(self.live_index)
    }

    pub(crate) fn redo(&mut self) -> Option<&T> {
        if self.live_index == self.stack.len() - 1 {
            return None;
        }
        self.live_index += 1;
        self.stack.get(self.live_index)
    }

    pub(crate) fn add_undo_group(&mut self, item: T) {
        if self.live_index < self.stack.len() - 1 {
            self.stack.split_off(self.live_index + 1);
        }

        self.live_index += 1;
        self.stack.push_back(item);

        if self.stack.len() > self.max_undo_count {
            self.stack.pop_front();
            self.live_index -= 1;
        }
    }

    /// Modify the state for the currently active undo group.
    /// This might be done if an edit occurs that combines with the previous undo,
    /// or if we want to save selection state.
    pub(crate) fn update_current_undo(&mut self, mut f: impl FnMut(&mut T)) {
        f(self.stack.get_mut(self.live_index).unwrap())
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum EditType {
    /// A catchall for edits that don't fit elsewhere, and which should
    /// always have their own undo groups; used for things like cut/copy/paste.
    Other,
    /// An insert from the keyboard/IME (not a paste or a yank).
    InsertChars,
    InsertNewline,
    /// An indentation adjustment.
    Indent,
    Delete,
    Undo,
    Redo,
    Transpose,
    //Surround,
}

impl EditType {
    /// Checks whether a new undo group should be created between two edits.
    pub(crate) fn breaks_undo_group(self, previous: EditType) -> bool {
        self == EditType::Other || self == EditType::Transpose || self != previous
    }

    pub(crate) fn from_event(event: &BufferEvent) -> Self {
        use BufferEvent as B;
        match event {
            B::Delete { .. } | B::Backspace => EditType::Delete,
            B::Transpose => EditType::Transpose,
            B::Undo => EditType::Undo,
            B::Redo => EditType::Redo,
            B::Indent | B::Outdent => EditType::Indent,
            B::Insert(_) | B::InsertTab => EditType::InsertChars,
            B::Paste(_) => EditType::Other,
            B::InsertNewline => EditType::InsertNewline,
            _ => EditType::Other,
            //B::Yank,
            //B::ReplaceNext,
            //B::ReplaceAll,
            //B::DuplicateLine,
            //B::IncreaseNumber,
            //B::DecreaseNumber,
            //B::Uppercase,
            //B::Lowercase,
            //B::Capitalize,
            //B::Cut,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test() {
        let mut stack = UndoStack::new_sized(5, 'a');
        assert_eq!(stack.undo(), None);
        assert_eq!(stack.redo(), None);
        stack.add_undo_group('b');
        assert_eq!(stack.undo(), Some(&'a'));
        assert_eq!(stack.redo(), Some(&'b'));

        stack.add_undo_group('c');
        assert_eq!(stack.undo(), Some(&'b'));
        assert_eq!(stack.redo(), Some(&'c'));

        stack.add_undo_group('d');
        assert_eq!(stack.undo(), Some(&'c'));
        assert_eq!(stack.redo(), Some(&'d'));

        stack.add_undo_group('e');
        assert_eq!(stack.undo(), Some(&'d'));
        assert_eq!(stack.redo(), Some(&'e'));
        assert_eq!(stack.undo(), Some(&'d'));
        assert_eq!(stack.undo(), Some(&'c'));
        assert_eq!(stack.redo(), Some(&'d'));
        assert_eq!(stack.redo(), Some(&'e'));

        // this should have popped 'a', since we're over our capacity
        stack.add_undo_group('f');
        assert_eq!(stack.undo(), Some(&'e'));
        assert_eq!(stack.undo(), Some(&'d'));
        assert_eq!(stack.undo(), Some(&'c'));
        assert_eq!(stack.undo(), Some(&'b'));
        assert_eq!(stack.undo(), None);

        assert_eq!(stack.redo(), Some(&'c'));
        assert_eq!(stack.redo(), Some(&'d'));
        assert_eq!(stack.redo(), Some(&'e'));

        // this should drop the 'f' group, which was toggled
        stack.add_undo_group('g');
        assert_eq!(stack.redo(), None);
        assert_eq!(stack.undo(), Some(&'e'));
        assert_eq!(stack.redo(), Some(&'g'));
        assert_eq!(stack.redo(), None);
    }
}
