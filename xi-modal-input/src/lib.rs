
extern crate libc;

use libc::{boolean_t, uint32_t};


#[no_mangle]
pub extern "C" fn xiEventHandlerCreate(cb: extern fn(*const XiCoreEvent)) -> *mut XiEventHandler {
    let handler = EventHandler::new(|_ev| {});
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
pub extern "C" fn xiEventHandlerHandleInput(handler: *mut XiEventHandler, event: uint32_t) -> boolean_t {
    let event = event as u32;
    let handler = unsafe {
        assert!(!handler.is_null());
        &mut *handler
    };
    handler.0.handle_input(event) as boolean_t
}

#[repr(C)]
pub struct XiEventHandler(EventHandler);

#[repr(C)]
pub struct XiCoreEvent(u32);

type RawEvent = u32;

struct EventHandler {
    callback: Box<dyn Fn(XiCoreEvent)>,
}

impl EventHandler {
    fn new<F: Fn(XiCoreEvent) + 'static>(callback: F) -> EventHandler {
        EventHandler {
            callback: Box::new(callback),
        }
    }

    /// Returns `true` if this event should be passed to IME.
    fn handle_input(&mut self, input: RawEvent) -> bool {
        input % 2 == 0
    }
}
