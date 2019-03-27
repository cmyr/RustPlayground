extern crate libc;
#[macro_use]
extern crate serde_json;

mod vim;

use libc::{c_char, uint32_t};
use std::ffi::{CStr, CString};

// a token for an event that has been scheduled with a delay.
type PendingToken = u32;

type Milliseconds = u32;

pub enum EventPayload {}

pub struct KeyEvent {
    modifiers: uint32_t,
    characters: &'static str,
    payload: *const EventPayload,
}

#[no_mangle]
pub extern "C" fn xiEventHandlerCreate(
    event_cb: extern "C" fn(*const EventPayload, bool),
    action_cb: extern "C" fn(*const c_char),
    timer_cb: extern "C" fn(*const EventPayload, uint32_t) -> uint32_t,
    cancel_timer_cb: extern "C" fn(uint32_t),
) -> *const XiEventHandler {
    let handler = EventHandler::new(event_cb, action_cb, timer_cb, cancel_timer_cb);
    let machine = vim::Machine::new();
    let r = Box::into_raw(Box::new(XiEventHandler(handler, Box::new(machine))));
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
pub extern "C" fn xiEventHandlerClearPending(handler: *mut XiEventHandler, token: uint32_t) {
    let handler = unsafe {
        assert!(!handler.is_null());
        &mut *handler
    };

    let XiEventHandler(_, machine) = handler;
    machine.clear_pending(token);
}

#[no_mangle]
pub extern "C" fn xiEventHandlerHandleInput(
    handler: *mut XiEventHandler,
    modifiers: uint32_t,
    characters: *const c_char,
    payload: *const EventPayload,
) {
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

    let handler = unsafe {
        assert!(!handler.is_null());
        &mut *handler
    };

    let event = KeyEvent {
        characters,
        modifiers,
        payload,
    };

    let XiEventHandler(handler, machine) = handler;
    machine.handle_event(event, handler);
}

#[repr(C)]
pub struct XiEventHandler(EventHandler, Box<dyn Handler>);

struct EventHandler {
    event_callback: extern "C" fn(*const EventPayload, bool),
    action_callback: extern "C" fn(*const c_char),
    timer_callback: extern "C" fn(*const EventPayload, uint32_t) -> uint32_t,
    cancel_timer_callback: extern "C" fn(uint32_t),
}

impl EventHandler {
    fn new(
        event_callback: extern "C" fn(*const EventPayload, bool),
        action_callback: extern "C" fn(*const c_char),
        timer_callback: extern "C" fn(*const EventPayload, uint32_t) -> uint32_t,
        cancel_timer_callback: extern "C" fn(uint32_t),
    ) -> EventHandler {
        EventHandler {
            event_callback,
            action_callback,
            timer_callback,
            cancel_timer_callback,
        }
    }

    fn send_action(&self, method: &str, params: Option<serde_json::Value>) {
        let params = params.unwrap_or(serde_json::Map::new().into());
        let json = json!({
            "method": method,
            "params": params,
        });
        let action_str = serde_json::to_string(&json).expect("Value is always valid json");
        let action_cstr = CString::new(action_str).expect("json should be well formed");
        (self.action_callback)(action_cstr.as_ptr());
    }

    fn send_event(&self, event: KeyEvent) {
        let KeyEvent { payload, .. } = event;
        (self.event_callback)(payload, false);
    }

    fn free_event(&self, event: KeyEvent) {
        let KeyEvent { payload, .. } = event;
        (self.event_callback)(payload, true);
    }

    fn schedule_event(&self, event: KeyEvent, delay: Milliseconds) -> PendingToken {
        let KeyEvent { payload, .. } = event;
        (self.timer_callback)(payload, delay)
    }

    fn cancel_timer(&self, token: PendingToken) {
        (self.cancel_timer_callback)(token);
    }
}

trait Handler {
    fn handle_event(&mut self, event: KeyEvent, handler: &EventHandler);
    /// Informs the handler that the given delayed event has fired.
    fn clear_pending(&mut self, token: PendingToken);
}
