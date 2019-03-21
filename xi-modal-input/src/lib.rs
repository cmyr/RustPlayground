
extern crate libc;

use libc::{c_char, uint32_t};
use std::ffi::CStr;


pub enum EventPayload {}

pub struct KeyEvent<'a> {
    modifiers: uint32_t,
    characters: &'a CStr,
    payload: *const EventPayload,
}

#[no_mangle]
pub extern "C" fn xiEventHandlerCreate(cb: extern fn(*const EventPayload)) -> *mut XiEventHandler {
    let handler = EventHandler::new(move |pl| { cb(pl) });
    let r = Box::into_raw(Box::new(XiEventHandler(handler)));
    eprintln!("event handler alloc {:?}", &r);
    r
}

#[no_mangle]
pub extern "C" fn xiEventHandlerFree(ptr: *mut XiEventHandler) {
    eprintln!("event handler free {:?}", ptr);
    if ptr.is_null() {
        return;
    }

    unsafe {
        Box::from_raw(ptr);
    }
}

#[no_mangle]
pub extern "C" fn xiEventHandlerHandleInput(handler: *mut XiEventHandler, modifiers: uint32_t, characters: *const c_char, payload: *const EventPayload) {
    let characters = unsafe {
        assert!(!characters.is_null());
        CStr::from_ptr(characters)
    };

    let handler = unsafe {
        assert!(!handler.is_null());
        &mut *handler
    };

    let event = KeyEvent {
        characters,
        modifiers,
        payload,
    };

    handler.0.handle_input(event);
}

#[repr(C)]
pub struct XiEventHandler(EventHandler);

#[repr(C)]
pub struct XiCoreEvent(u32);

struct EventHandler {
    callback: Box<dyn Fn(*const EventPayload)>,
}

impl EventHandler {
    fn new<F: Fn(*const EventPayload) + 'static>(callback: F) -> EventHandler {
        EventHandler {
            callback: Box::new(callback),
        }
    }

    /// Returns `true` if this event should be passed to IME.
    fn handle_input<'a>(&mut self, input: KeyEvent<'a>) {
        let KeyEvent { payload, .. } = input;
        (self.callback)(payload);
    }
}
