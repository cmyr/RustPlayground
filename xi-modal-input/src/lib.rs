extern crate libc;
#[macro_use]
extern crate serde_json;

use libc::{c_char, uint32_t};
use std::ffi::{CStr, CString};

const KEY_TIMEOUT_MILLIS: uint32_t = 500;

// a token for an event that has been scheduled with a delay.
type PendingToken = uint32_t;

type Milliseconds = uint32_t;

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
    let handler = EventHandler::new(
        move |event, discard| event_cb(event, discard),
        move |json_cstr| action_cb(json_cstr),
        timer_cb,
        cancel_timer_cb,
    );
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

//FIXME: the first two here can also just be extern C fns
struct EventHandler {
    event_callback: Box<dyn Fn(*const EventPayload, bool)>,
    action_callback: Box<dyn Fn(*const c_char)>,
    timer_callback: extern "C" fn(*const EventPayload, uint32_t) -> uint32_t,
    cancel_timer_callback: extern "C" fn(uint32_t),
}

impl EventHandler {
    fn new<F1, F2>(
        event_callback: F1,
        action_callback: F2,
        timer_callback: extern "C" fn(*const EventPayload, uint32_t) -> uint32_t,
        cancel_timer_callback: extern "C" fn(uint32_t),
    ) -> EventHandler
    where
        F1: Fn(*const EventPayload, bool) + 'static,
        F2: Fn(*const c_char) + 'static,
    {
        EventHandler {
            event_callback: Box::new(event_callback),
            action_callback: Box::new(action_callback),
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

mod vim {
    use super::*;

    enum Mode {
        Insert,
        Command,
    }

    #[derive(Debug, Clone, Copy)]
    enum CommandType {
        Move,
        Delete,
    }

    impl CommandType {
        fn from_char(chr: char) -> Option<CommandType> {
            match chr {
                'd' => Some(CommandType::Delete),
                _ => None,
            }
        }

        fn as_str(&self) -> &'static str {
            match self {
                CommandType::Move => "move",
                CommandType::Delete => "delete",
            }
        }
    }

    #[derive(Debug, Clone, Copy)]
    struct Command {
        ty: CommandType,
        motion: Motion,
        distance: usize,
    }

    #[derive(Debug, Clone, Copy)]
    enum Motion {
        Left,
        Right,
        Up,
        Down,
        Word,
        BackWord,
    }

    impl Motion {
        fn from_char(chr: char) -> Option<Motion> {
            match chr {
                'h' => Some(Motion::Left),
                'l' => Some(Motion::Right),
                'j' => Some(Motion::Down),
                'k' => Some(Motion::Up),
                'w' => Some(Motion::Word),
                'b' => Some(Motion::BackWord),
                _ => None,
            }
        }

        fn as_str(&self) -> &'static str {
            match self {
                Motion::Left => "left",
                Motion::Right => "right",
                Motion::Down => "down",
                Motion::Up => "up",
                Motion::Word => "word",
                Motion::BackWord => "word_back",
            }
        }
    }

    enum CommandState {
        Ready,
        AwaitMotion(CommandType, usize),
        Done(Command),
        Failed,
    }

    pub(crate) struct Machine {
        mode: Mode,
        state: CommandState,
        raw: String,
        timeout_token: Option<PendingToken>,
    }

    impl Handler for Machine {
        fn handle_event(&mut self, event: KeyEvent, handler: &EventHandler) {
            match self.mode {
                Mode::Insert => self.handle_insert(event, handler),
                Mode::Command => {
                    self.handle_command(&event, handler);
                    handler.free_event(event);
                }
            }
        }

        fn clear_pending(&mut self, _token: PendingToken) {
            self.timeout_token = None;
        }
    }

    impl Machine {
        pub(crate) fn new() -> Self {
            Machine {
                mode: Mode::Insert,
                state: CommandState::Ready,
                raw: String::new(),
                timeout_token: None,
            }
        }

        fn handle_insert(&mut self, event: KeyEvent, handler: &EventHandler) {
            let timeout_token = self.timeout_token.take();
            let mut to_command_mode = |event| {
                self.mode = Mode::Command;
                handler.send_action("mode_change", Some(json!({"mode": "command"})));
                handler.free_event(event);
            };

            if event.characters == "␛" {
                to_command_mode(event);
            } else if event.characters == "j" {
                if let Some(token) = timeout_token {
                    handler.cancel_timer(token);
                    to_command_mode(event);
                } else {
                    let token = handler.schedule_event(event, KEY_TIMEOUT_MILLIS);
                    self.timeout_token = Some(token);
                }
            } else {
                handler.send_event(event);
            }
        }

        fn handle_command(&mut self, event: &KeyEvent, handler: &EventHandler) {
            let chr = match event.characters.chars().next() {
                Some(c) => c,
                None => {
                    eprintln!("no chars for event");
                    return;
                }
            };

            if let CommandState::Ready = self.state {
                if chr == 'i' {
                    self.mode = Mode::Insert;
                    handler.send_action("mode_change", Some(json!({"mode": "insert"})));
                    handler.send_action("parse_state", Some(json!({"state": ""})));
                    return;
                }
            }

            self.state = match &self.state {
                CommandState::Ready => {
                    self.raw.push(chr);
                    if let Some(motion) = Motion::from_char(chr) {
                        CommandState::Done(Command {
                            motion,
                            ty: CommandType::Move,
                            distance: 1,
                        })
                    } else if let Some(cmd) = CommandType::from_char(chr) {
                        CommandState::AwaitMotion(cmd, 0)
                    } else if let Some(num) = chr.to_digit(10) {
                        CommandState::AwaitMotion(CommandType::Move, num as usize)
                    } else {
                        CommandState::Failed
                    }
                }

                CommandState::AwaitMotion(ty, dist) => {
                    self.raw.push(chr);
                    if let Some(motion) = Motion::from_char(chr) {
                        CommandState::Done(Command {
                            motion,
                            ty: *ty,
                            distance: *dist.max(&1),
                        })
                    } else if let Some(num) = chr.to_digit(10) {
                        let new_dist = dist * 10 + num as usize;
                        CommandState::AwaitMotion(*ty, new_dist)
                    } else {
                        CommandState::Failed
                    }
                }
                _ => unreachable!(),
            };

            if let CommandState::Done(Command {
                motion,
                ty,
                distance,
            }) = &self.state
            {
                handler.send_action(
                    ty.as_str(),
                    Some(json!({"motion": motion.as_str(), "dist": distance})),
                );
                handler.send_action("parse_state", Some(json!({"state": &self.raw})));
                self.state = CommandState::Ready;
                self.raw = String::new();
            } else if let CommandState::Failed = self.state {
                self.state = CommandState::Ready;
                handler.send_action("parse_state", Some(json!({"state": &self.raw})));
                self.raw = String::new();
            } else {
                handler.send_action("parse_state", Some(json!({"state": &self.raw})));
            }
        }
    }
}
