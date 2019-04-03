use std::ffi::{CStr, CString};

use libc::{c_char, int32_t, size_t, uint32_t};
extern crate xi_modal_input;
use xi_modal_input::{EventCtx, EventPayload, KeyEvent, OneView, Plumber, Vim, XiCore};

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
    width_measure_fn: extern "C" fn(*const c_char) -> size_t,
) -> *const XiCore {
    let r = Box::into_raw(Box::new(XiCore {
        rpc_callback,
        invalidate_callback,
        state: OneView::new(width_measure_fn),
        plumber: None,
        handler: None,
    }));
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

    let needs_render = core.handler.as_mut().unwrap().handle_event(event, ctx);
    if needs_render {
        core.send_update();
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

    let (line, cursor, sel) = core.state.get_line(idx as usize).unwrap();
    let cstring = CString::new(line.as_ref()).expect("bad string, very sad");
    Box::into_raw(Box::new(XiLine { text: cstring.into_raw(), cursor, selection: [sel.0, sel.1] }))
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
