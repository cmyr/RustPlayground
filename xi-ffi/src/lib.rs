use std::ffi::{CStr, CString};

use libc::{c_char, size_t};
extern crate xi_modal_input;
use xi_modal_input::{EventCtx, EventPayload, KeyEvent, Line, OneView, Plumber, Size, Vim, XiCore};

#[repr(C)]
pub struct XiLine {
    text: *const c_char,
    cursor: i32,
    selection: [size_t; 2],
    styles: *const [size_t],
    styles_len: size_t,
}

#[no_mangle]
pub extern "C" fn xiCoreCreate(
    rpc_callback: extern "C" fn(*const c_char),
    invalidate_callback: extern "C" fn(size_t, size_t),
    width_measure_fn: extern "C" fn(*const c_char) -> Size,
) -> *const XiCore {
    let r = Box::into_raw(Box::new(XiCore::new(
        rpc_callback,
        invalidate_callback,
        OneView::new(width_measure_fn),
    )));
    eprintln!("xiCore alloc {:?}", &r);
    r
}

#[no_mangle]
pub extern "C" fn xiCoreRegisterEventHandler(
    ptr: *mut XiCore,
    event_cb: extern "C" fn(*const EventPayload, bool),
    action_cb: extern "C" fn(*const c_char),
    timer_cb: extern "C" fn(*const EventPayload, u32) -> u32,
    cancel_timer_cb: extern "C" fn(u32),
) {
    let core = unsafe {
        assert!(!ptr.is_null(), "null pointer in xiCoreRegisterEventHandler");
        &mut *ptr
    };

    let machine = Vim::new();
    let plumber = Plumber::new(event_cb, action_cb, timer_cb, cancel_timer_cb);
    core.plumber = Some(plumber);
    core.handler = Some(Box::new(machine));
}

#[no_mangle]
pub extern "C" fn xiCoreHandleInput(
    ptr: *mut XiCore,
    modifiers: u32,
    characters: *const c_char,
    payload: *const EventPayload,
) {
    let core = unsafe {
        assert!(!ptr.is_null());
        &mut *ptr
    };

    let cstr = unsafe {
        assert!(!characters.is_null());
        CStr::from_ptr(characters)
    };

    let characters = match cstr.to_str() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("invalid cstr: {}, {:?}", e, cstr.to_bytes());
            ""
        }
    };

    let event = KeyEvent { modifiers, characters, payload };

    let ctx = EventCtx { plumber: core.plumber.as_ref().unwrap(), state: &mut core.state };
    if let Some(update) = core.handler.as_mut().unwrap().handle_event(event, ctx) {
        core.send_update(update);
    }
}

#[no_mangle]
pub extern "C" fn xiCoreClearPending(ptr: *mut XiCore, token: u32) {
    let core = unsafe {
        assert!(!ptr.is_null());
        &mut *ptr
    };

    if core.handler.is_none() {
        eprintln!("unexpected None in xiCoreClearPending");
    }

    if let Some(h) = core.handler.as_mut() {
        h.clear_pending(token)
    };
}

#[no_mangle]
pub extern "C" fn xiCoreGetLine(ptr: *mut XiCore, idx: u32) -> *const XiLine {
    let core = unsafe {
        assert!(!ptr.is_null(), "null pointer in xiCoreGetLine");
        &mut *ptr
    };

    match core.state.get_line(idx as usize) {
        Some(Line { line, caret, selection, styles }) => {
            let text = CString::new(line.as_ref()).expect("bad string, very sad").into_raw();
            let styles_len = styles.len();
            let styles = Box::into_raw(styles.into_boxed_slice());

            let cursor = caret.map(|v| v as i32).unwrap_or(-1);
            let xiline =
                XiLine { text, cursor, selection: [selection.0, selection.1], styles, styles_len };
            Box::into_raw(Box::new(xiline))
        }
        None => std::ptr::null(),
    }
}

#[no_mangle]
pub extern "C" fn xiCoreFree(ptr: *mut XiCore) {
    eprintln!("xiCore free {:?}", &ptr);
    if ptr.is_null() {
        return;
    }

    unsafe {
        Box::from_raw(ptr);
    }
}

#[no_mangle]
pub extern "C" fn xiCStringFree(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }

    unsafe {
        CString::from_raw(ptr);
    }
}

#[no_mangle]
pub extern "C" fn xiLineFree(ptr: *mut XiLine) {
    if ptr.is_null() {
        return;
    }

    unsafe {
        let line = Box::from_raw(ptr);
        CString::from_raw(line.text as *mut _);
        Box::from_raw(line.styles as *mut [usize]);
    }
}

#[no_mangle]
pub extern "C" fn xiCoreSendMessage(ptr: *mut XiCore, msg: *const c_char) {
    let core = unsafe {
        assert!(!ptr.is_null(), "null pointer in xiCoreSendMessage");
        &mut *ptr
    };

    let cstr = unsafe {
        assert!(!ptr.is_null(), "null msg pointer in xiCoreSendMessage");
        CStr::from_ptr(msg)
    };

    let msg = match cstr.to_str() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("invalid cstr: {}, {:?}", e, cstr.to_bytes());
            return;
        }
    };

    core.handle_message(msg);
}
