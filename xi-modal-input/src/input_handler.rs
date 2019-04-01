use libc::{c_char, uint32_t};
use std::ffi::{CStr, CString};

use xi_core_lib::edit_types::EventDomain;

use super::OneView;

// a token for an event that has been scheduled with a delay.
pub type PendingToken = u32;

type Milliseconds = u32;

pub enum EventPayload {}

pub struct KeyEvent {
    pub modifiers: uint32_t,
    pub characters: &'static str,
    pub payload: *const EventPayload,
}

#[no_mangle]
pub extern "C" fn xiEventHandlerCreate(
    event_cb: extern "C" fn(*const EventPayload, bool),
    action_cb: extern "C" fn(*const c_char),
    timer_cb: extern "C" fn(*const EventPayload, uint32_t) -> uint32_t,
    cancel_timer_cb: extern "C" fn(uint32_t),
) -> *const XiEventHandler {
    let handler = Plumber::new(event_cb, action_cb, timer_cb, cancel_timer_cb);
    let machine = crate::vim::Machine::new();
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

//#[no_mangle]
//pub extern "C" fn xiEventHandlerHandleInput(
    //handler: *mut XiEventHandler,
    //modifiers: uint32_t,
    //characters: *const c_char,
    //payload: *const EventPayload,
//) {
    //let cstr = unsafe {
        //assert!(!characters.is_null());
        //CStr::from_ptr(characters)
    //};

    //let characters = match cstr.to_str() {
        //Ok(s) => s,
        //Err(e) => {
            //eprintln!("invalid cstr: {}, {:?}", e, cstr.to_bytes());
            //""
        //}
    //};

    //let handler = unsafe {
        //assert!(!handler.is_null());
        //&mut *handler
    //};

    //let event = KeyEvent {
        //characters,
        //modifiers,
        //payload,
    //};

    ////let XiEventHandler(handler, machine) = handler;
    ////machine.handle_event(event, handler);
//}

#[repr(C)]
pub struct XiEventHandler(Plumber, Box<dyn Handler>);

pub struct Plumber {
    event_callback: extern "C" fn(*const EventPayload, bool),
    action_callback: extern "C" fn(*const c_char),
    timer_callback: extern "C" fn(*const EventPayload, uint32_t) -> uint32_t,
    cancel_timer_callback: extern "C" fn(uint32_t),
}

impl Plumber {
    pub(crate) fn new(
        event_callback: extern "C" fn(*const EventPayload, bool),
        action_callback: extern "C" fn(*const c_char),
        timer_callback: extern "C" fn(*const EventPayload, uint32_t) -> uint32_t,
        cancel_timer_callback: extern "C" fn(uint32_t),
    ) -> Plumber {
        Plumber {
            event_callback,
            action_callback,
            timer_callback,
            cancel_timer_callback,
        }
    }
}


pub struct EventCtx<'a> {
    pub plumber: &'a Plumber,
    pub state: &'a mut OneView,
}

impl<'a> EventCtx<'a> {
    pub(crate) fn send_event(&self, event: KeyEvent) {
        let KeyEvent { payload, .. } = event;
        (self.plumber.event_callback)(payload, false);
    }

    pub(crate) fn free_event(&self, event: KeyEvent) {
        let KeyEvent { payload, .. } = event;
        (self.plumber.event_callback)(payload, true);
    }

    pub(crate) fn schedule_event(&self, event: KeyEvent, delay: Milliseconds) -> PendingToken {
        let KeyEvent { payload, .. } = event;
        (self.plumber.timer_callback)(payload, delay)
    }

    pub(crate) fn cancel_timer(&self, token: PendingToken) {
        (self.plumber.cancel_timer_callback)(token);
    }

    pub(crate) fn send_client_rpc<V>(&self, method: &str, params: V)
        where V: Into<Option<serde_json::Value>>,
    {
        let params = params.into().unwrap_or(serde_json::Map::new().into());
        let json = json!({
            "method": method,
            "params": params,
        });
        let action_str = serde_json::to_string(&json).expect("Value is always valid json");
        let action_cstr = CString::new(action_str).expect("json should be well formed");
        (self.plumber.action_callback)(action_cstr.as_ptr());
    }

    pub(crate) fn do_core_event(&mut self, action: EventDomain, repeat: usize) {
        for _ in 0..repeat {
            self.state.handle_event(action.clone());
        }
    }
}


pub trait Handler {
    /// Returns `true` if we should update after this event
    fn handle_event(&mut self, event: KeyEvent, ctx: EventCtx) -> bool;
    /// Informs the handler that the given delayed event has fired.
    fn clear_pending(&mut self, token: PendingToken);
}
