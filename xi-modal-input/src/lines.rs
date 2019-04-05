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

pub(crate) fn rewrap_region<IV: IntervalBounds>(
    text: &Rope,
    interval: IV,
    width_cache: &WidthCache,
) -> Breaks {
    let interval = interval.into_interval(text.len());

    let mut builder = BreakBuilder::new();
    let mut last_word = interval.start;
    let mut last_line = interval.start;
    let mut break_cursor = LineBreakCursor::new(&text, interval.start);
    let mut cur_width = 0;
    //eprintln!("updating for iv {:?}, new_len {} old_len {}", iv, new_len, text.len());
    loop {
        if last_word == interval.end {
            break;
        }
        let (next, is_hard) = break_cursor.next();
        let this_word = text.slice_to_cow(last_word..next);
        let Size { width, .. } = width_cache.measure_layout_size(&this_word);
        cur_width += width;
        //eprintln!("word '{}', width {} next {}", this_word, width, next);

        if is_hard || next >= text.len() {
            builder.add_break(next - last_line, cur_width);
            last_line = next;
            cur_width = 0;
            if last_word >= interval.end {
                break;
            }
        }
        last_word = next;
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

        let cstr = CString::new(line).unwrap();
        let size = (self.measure_fn)(cstr.as_ptr());
        cache.insert(line.to_owned(), size);
        size
    }
}
