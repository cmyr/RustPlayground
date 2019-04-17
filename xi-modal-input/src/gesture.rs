//! Gesture (mouse) based movement

use xi_core_lib::rpc::{GestureType, SelectionGranularity};
use xi_core_lib::selection::{SelRegion, Selection};
use xi_core_lib::word_boundaries::WordCursor;
use xi_rope::Rope;

pub(crate) fn region_for_gesture(
    text: &Rope,
    offset: usize,
    granularity: SelectionGranularity,
) -> SelRegion {
    match granularity {
        SelectionGranularity::Point => SelRegion::caret(offset),
        SelectionGranularity::Word => {
            let mut word_cursor = WordCursor::new(text, offset);
            let (start, end) = word_cursor.select_word();
            SelRegion::new(start, end)
        }
        SelectionGranularity::Line => {
            let line = text.line_of_offset(offset);
            let start = text.offset_of_line(line);
            let end = text.offset_of_line(line + 1);
            SelRegion::new(start, end)
        }
    }
}

pub(crate) struct GestureContext<'a> {
    text: &'a Rope,
    sel: &'a Selection,
}

impl<'a> GestureContext<'a> {
    pub(crate) fn new(text: &'a Rope, sel: &'a Selection) -> Self {
        GestureContext { text, sel }
    }

    pub(crate) fn selection_for_gesture(&self, offset: usize, gesture: GestureType) -> Selection {
        if let GestureType::Select { granularity: SelectionGranularity::Point, multi: true } =
            gesture
        {
            if !self.sel.regions_in_range(offset, offset).is_empty() && self.sel.len() > 1 {
                // we don't allow toggling the last selection
                let mut new = self.sel.clone();
                new.delete_range(offset, offset, true);
                return new;
            }
        }

        match gesture {
            GestureType::Select { granularity, multi } => {
                let new_region = region_for_gesture(&self.text, offset, granularity);
                if multi {
                    let mut new = self.sel.clone();
                    new.add_region(new_region);
                    new
                } else {
                    new_region.into()
                }
            }
            GestureType::SelectExtend { granularity } => {
                if self.sel.len() == 0 {
                    return self.sel.clone();
                }
                let active_region = self.sel.last().clone().unwrap();
                let new_region = region_for_gesture(self.text, offset, granularity);
                let merged_region = if offset >= new_region.start {
                    SelRegion::new(active_region.start, new_region.end)
                } else {
                    SelRegion::new(active_region.start, new_region.start)
                };
                let mut new = self.sel.clone();
                new.add_region(merged_region);
                new
            }
            GestureType::Drag => self.sel.clone(),
            _other => panic!("unexpected gesture type {:?}", _other),
        }
    }
}
