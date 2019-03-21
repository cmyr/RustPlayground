
extern crate libc;
#[macro_use] extern crate serde_json;

use libc::{c_char, uint32_t};
use std::ffi::{CString, CStr};


pub enum EventPayload {}

pub struct KeyEvent<'a> {
    modifiers: uint32_t,
    characters: &'a str,
    payload: *const EventPayload,
}

#[no_mangle]
pub extern "C" fn xiEventHandlerCreate(event_cb: extern fn(*const EventPayload), action_cb: extern fn(*const c_char)) -> *mut XiEventHandler {
    let handler = EventHandler::new(move |pl| { event_cb(pl) }, move |cstr| { action_cb(cstr) });
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
pub extern "C" fn xiEventHandlerHandleInput(handler: *mut XiEventHandler, modifiers: uint32_t, characters: *const c_char, payload: *const EventPayload) {
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

#[repr(C)]
pub struct XiCoreEvent(u32);

struct EventHandler {
    event_callback: Box<dyn Fn(*const EventPayload)>,
    action_callback: Box<dyn Fn(*const c_char)>,
}

impl EventHandler {
    fn new<F1, F2>(event_callback: F1, action_callback: F2) -> EventHandler
        where
            F1: Fn(*const EventPayload) + 'static,
            F2: Fn(*const c_char) + 'static,
    {
        EventHandler {
            event_callback: Box::new(event_callback),
            action_callback: Box::new(action_callback),
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

    fn send_event<'a>(&self, event: KeyEvent<'a>) {
        let KeyEvent { payload, .. } = event;
        (self.event_callback)(payload);
    }
}


trait Handler {
    fn handle_event<'a>(&mut self, event: KeyEvent<'a>, handler: &EventHandler);
}

mod vim {
    use super::{EventHandler, Handler, KeyEvent};

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
                Motion::BackWord => "back",
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
    }

    impl Handler for Machine {
        fn handle_event<'a>(&mut self, event: KeyEvent<'a>, handler: &EventHandler) {
            match self.mode {
                Mode::Insert => self.handle_insert(event, handler),
                Mode::Command => self.handle_command(event, handler),
            }
        }
    }

    impl Machine {
        pub(crate) fn new() -> Self {
            Machine {
                mode: Mode::Insert,
                state: CommandState::Ready,
            }
        }

        fn handle_insert<'a>(&mut self, event: KeyEvent<'a>, handler: &EventHandler) {
            if event.characters == "Z" {
                self.mode = Mode::Command;
                handler.send_action("mode_change", Some(json!({"mode": "command"})));
            } else {
                handler.send_event(event);
            }
        }

        fn handle_command<'a>(&mut self, event: KeyEvent<'a>, handler: &EventHandler) {
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
                    return;
                }
            }

            self.state = match &self.state {
                CommandState::Ready => {
                    if let Some(motion) = Motion::from_char(chr) {
                        CommandState::Done(Command { motion, ty: CommandType::Move, distance: 1 })
                    } else if let Some(cmd) = CommandType::from_char(chr) {
                        CommandState::AwaitMotion(cmd, 0)
                    } else if let Some(num) = chr.to_digit(10) {
                        CommandState::AwaitMotion(CommandType::Move, num as usize)
                    } else {
                        CommandState::Failed
                    }
                }

                CommandState::AwaitMotion(ty, dist) => {
                    if let Some(motion) = Motion::from_char(chr) {
                        CommandState::Done(Command { motion, ty: *ty, distance: *dist.max(&1) })
                    } else if let Some(num) = chr.to_digit(10) {
                        let new_dist = dist * 10 + num as usize;
                        CommandState::AwaitMotion(*ty, new_dist)
                    } else {
                        CommandState::Failed
                    }
                }
                _ => unreachable!(),
            };

            if let CommandState::Done(Command { motion, ty, distance }) = &self.state {
                handler.send_action(ty.as_str(), Some(json!({"motion": motion.as_str(), "dist": distance})));
                self.state = CommandState::Ready;
            } else if let CommandState::Failed = self.state {
                eprintln!("parse failed");
                self.state = CommandState::Ready;
            }
        }
    }
}

