use std::ffi::{CStr, CString};

use libc::{c_char, int32_t, size_t, uint32_t};
extern crate xi_modal_input;
use xi_modal_input::{EventCtx, EventPayload, KeyEvent, OneView, Plumber, Size, Vim, XiCore};

#[repr(C)]
pub struct XiLine {
    text: *const c_char,
    cursor: int32_t,
    selection: [int32_t; 2],
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
    timer_cb: extern "C" fn(*const EventPayload, uint32_t) -> uint32_t,
    cancel_timer_cb: extern "C" fn(uint32_t),
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
    modifiers: uint32_t,
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

    let event = KeyEvent { characters, modifiers, payload };

    let ctx = EventCtx { plumber: core.plumber.as_ref().unwrap(), state: &mut core.state };
    if let Some(update) = core.handler.as_mut().unwrap().handle_event(event, ctx) {
        core.send_update(update);
    }
}

#[no_mangle]
pub extern "C" fn xiCoreClearPending(ptr: *mut XiCore, token: uint32_t) {
    let core = unsafe {
        assert!(!ptr.is_null());
        &mut *ptr
    };

    if core.handler.is_none() {
        eprintln!("unexpected None in xiCoreClearPending");
    }

    core.handler.as_mut().map(|h| h.clear_pending(token));
}

#[no_mangle]
pub extern "C" fn xiCoreGetLine(ptr: *mut XiCore, idx: uint32_t) -> *const XiLine {
    let core = unsafe {
        assert!(!ptr.is_null(), "null pointer in xiCoreGetLine");
        &mut *ptr
    };

    match core.state.get_line(idx as usize) {
        Some((line, cursor, sel)) => {
            let cstring = CString::new(line.as_ref()).expect("bad string, very sad");
            Box::into_raw(Box::new(XiLine { text: cstring.into_raw(), cursor, selection: [sel.0, sel.1] }))
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
