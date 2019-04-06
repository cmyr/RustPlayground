use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::CString;

use libc::c_char;
use xi_core_lib::linewrap::LineBreakCursor;
use xi_rope::breaks2::{BreakBuilder, Breaks};
use xi_rope::interval::IntervalBounds;
use xi_rope::Rope;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    pub width: usize,
    pub height: usize,
}

impl Size {
    pub fn zero() -> Self {
        Size { width: 0, height: 0 }
    }
}

pub(crate) fn rewrap_region<IV, T>(
    text: &Rope,
    interval: IV,
    width_cache: &WidthCache,
    view_width: T,
) -> Breaks
where
    IV: IntervalBounds,
    T: Into<Option<usize>>,
{
    let interval = interval.into_interval(text.len());
    let view_width = view_width.into();

    let mut builder = BreakBuilder::new();
    let mut last_break = interval.start;

    let mut breaks_iter = BreaksIter::new(text, interval.start, width_cache, view_width);
    while let Some(Break { offset, width, hard }) = breaks_iter.next() {
        if offset == interval.end && !hard {
            builder.add_no_break(offset - last_break, width);
            break;
        }
        if last_break >= interval.end {
            break;
        }

        builder.add_break(offset - last_break, width);
        last_break = offset;
    }
    builder.build()
}

#[derive(Debug, Clone)]
pub struct WidthCache {
    cache: RefCell<HashMap<String, Size>>,
    measure_fn: extern "C" fn(*const c_char) -> Size,
}

impl WidthCache {
    pub fn new(measure_fn: extern "C" fn(*const c_char) -> Size) -> Self {
        WidthCache { cache: RefCell::new(HashMap::new()), measure_fn }
    }

    pub fn measure_layout_size(&self, line: &str) -> Size {
        //HACK: we want this method to take &self so we're using a refcell
        let mut cache = self.cache.borrow_mut();
        if let Some(size) = cache.get(line) {
            return *size;
        }

        let cstr = CString::new(line).expect("measure layout bad string?");
        let size = (self.measure_fn)(cstr.as_ptr());
        cache.insert(line.to_owned(), size);
        size
    }
}

struct BreaksIter<'a> {
    cache: &'a WidthCache,
    cursor: LineBreakCursor<'a>,
    /// the size used for soft wrapping.
    view_width: usize,
    last: usize,
    cur_line_width: usize,
    done: bool,
}

#[derive(Debug, Clone, Copy)]
struct Break {
    offset: usize,
    width: usize,
    hard: bool,
}

impl<'a> BreaksIter<'a> {
    fn new<T>(text: &'a Rope, start: usize, cache: &'a WidthCache, width: T) -> Self
    where
        T: Into<Option<usize>>,
    {
        let cursor = LineBreakCursor::new(text, start);
        let view_width = width.into().unwrap_or(usize::max_value());
        BreaksIter { cursor, cache, view_width, last: start, cur_line_width: 0, done: false }
    }

    /// Returns the offset of the next potential break, the width from that
    /// to the previous break, and whether it is a hard break (newline).
    fn next_pot_break(&mut self) -> Break {
        let cur_pos = self.last;
        let (next, hard) = self.cursor.next();

        let word = self.cursor.get_text().slice_to_cow(cur_pos..next);
        let width = self.cache.measure_layout_size(&word).width;
        self.last = next;

        Break { offset: next, width, hard }
    }
}

impl<'a> Iterator for BreaksIter<'a> {
    type Item = Break;

    fn next(&mut self) -> Option<Break> {
        let text_len = self.cursor.get_text().len();
        let mut line_width = self.cur_line_width;
        let mut cur_offset = self.last;

        while cur_offset < text_len {
            let Break { offset, width, hard } = self.next_pot_break();
            if !hard {
                if line_width == 0 && width >= self.view_width {
                    // we don't care about soft breaks at EOF
                    if offset == text_len {
                        return None;
                    }
                    // this is a single word longer than a line; always break afterwords
                    return Some(Break { offset, width, hard });
                }
                line_width += width;
                if line_width > self.view_width {
                    // stash this width, it's the starting width of our next line
                    self.cur_line_width = width;
                    return Some(Break { offset: cur_offset, width: line_width - width, hard });
                }
                cur_offset = offset;
            } else if line_width > 0 && width + line_width > self.view_width {
                // if this is a hard break but we would have broken at the previous
                // pos otherwise, we still break at the previous pos.
                self.cur_line_width = width;
                return Some(Break { offset: cur_offset, width: line_width, hard });
            } else {
                self.cur_line_width = 0;
                return Some(Break { offset, width: width + line_width, hard });
            }
        }

        // only return last break if hard?
        if !self.done && cur_offset != 0 {
            self.done = true;
            Some(Break { offset: cur_offset, width: line_width, hard: false })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;
    use xi_core_lib::view::ViewMovement;

    fn dummy_width_cache() -> WidthCache {
        extern "C" fn dummy_width(string: *const c_char) -> Size {
            let cstr = unsafe { CStr::from_ptr(string).to_str().unwrap() };
            Size { width: cstr.len(), height: 0 }
        }
        WidthCache::new(dummy_width)
    }

    #[test]
    fn my_understanding() {
        let text: Rope = "hell".into();
        assert_eq!(text.line_of_offset(4), 0);

        let breaks = Breaks::new_no_break(text.len());
        assert_eq!(breaks.line_of_offset(&text, 4), 0, "breaks");

        let text: Rope = "hell\n".into();
        let mut builder = BreakBuilder::new();
        builder.add_break(5, 0);
        let breaks = builder.build();

        assert_eq!(text.line_of_offset(4), 0);
        assert_eq!(breaks.line_of_offset(&text, 4), 0, "breaks");

        assert_eq!(text.line_of_offset(5), 1);
        assert_eq!(breaks.line_of_offset(&text, 5), 1, "breaks");
    }

    #[test]
    fn offset_stuff() {
        let text: Rope = "hello\nworld".into();
        let wc = dummy_width_cache();
        let breaks = rewrap_region(&text, .., &wc, None);
        assert_eq!(breaks.line_of_offset(&text, 0), 0);
        assert_eq!(breaks.line_of_offset(&text, 5), 0);
        assert_eq!(breaks.line_of_offset(&text, 6), 1);
        assert_eq!(breaks.line_of_offset(&text, text.len()), 1);
        assert_eq!(breaks.offset_of_line(&text, 1), 6);
    }
}
