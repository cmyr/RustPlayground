use crate::input_handler::{EventCtx, Handler, KeyEvent, PendingToken};
use crate::update::{Update, UpdateBuilder};
use xi_core_lib::edit_types::{BufferEvent, ViewEvent};
use xi_core_lib::movement::Movement;

const KEY_TIMEOUT_MILLIS: u32 = 500;

fn movement_from_str(s: &str) -> Option<Movement> {
    match s {
        "h" => Some(Movement::Left),
        "l" => Some(Movement::Right),
        "j" => Some(Movement::Down),
        "k" => Some(Movement::Up),
        "w" => Some(Movement::RightWord),
        "b" => Some(Movement::LeftWord),
        "0" => Some(Movement::LeftOfLine),
        "$" => Some(Movement::RightOfLine),
        _ => None,
    }
}

enum Mode {
    Insert,
    Command,
    Visual,
}

#[derive(Debug, Clone, Copy)]
enum CommandType {
    Move,
    Delete,
}

impl CommandType {
    fn from_char(chr: &str) -> Option<CommandType> {
        match chr {
            "d" => Some(CommandType::Delete),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Command {
    ty: CommandType,
    motion: Movement,
    distance: usize,
}

enum CommandState {
    Ready,
    AwaitMotion(CommandType, usize),
    Done(Command),
    Failed,
}

pub struct Machine {
    mode: Mode,
    state: CommandState,
    raw: String,
    timeout_token: Option<PendingToken>,
}

impl Handler for Machine {
    fn handle_event(&mut self, event: KeyEvent, mut ctx: EventCtx) -> Option<Update> {
        match self.mode {
            Mode::Insert => {
                self.handle_insert(event, &ctx);
                None
            }
            Mode::Command => {
                let r = self.handle_command(&event, &mut ctx);
                ctx.free_event(event);
                r
            }
            Mode::Visual => {
                let r = self.handle_visual(&event, &mut ctx);
                ctx.free_event(event);
                r
            }
        }
    }

    fn clear_pending(&mut self, _token: PendingToken) {
        self.timeout_token = None;
    }
}

impl Machine {
    pub fn new() -> Self {
        Machine {
            mode: Mode::Insert,
            state: CommandState::Ready,
            raw: String::new(),
            timeout_token: None,
        }
    }

    fn handle_insert(&mut self, event: KeyEvent, ctx: &EventCtx) {
        let timeout_token = self.timeout_token.take();
        let mut to_command_mode = |event| {
            self.mode = Mode::Command;
            ctx.send_client_rpc("mode_change", json!({"mode": "command"}));
            ctx.free_event(event);
        };

        if event.characters == "Escape" {
            to_command_mode(event);
        } else if event.characters == "j" {
            if let Some(token) = timeout_token {
                ctx.cancel_timer(token);
                to_command_mode(event);
            } else {
                let token = ctx.schedule_event(event, KEY_TIMEOUT_MILLIS);
                self.timeout_token = Some(token);
            }
        } else {
            ctx.send_event(event);
        }
    }

    /// returns `true` if an event has happened, and we should rerender.
    fn handle_command(&mut self, event: &KeyEvent, ctx: &mut EventCtx) -> Option<Update> {
        let chr = event.characters;

        let mut update = UpdateBuilder::new();

        if let CommandState::Ready = self.state {
            if ["i", "a", "A", "o", "O"].contains(&chr) {
                self.mode = Mode::Insert;
                if chr == "a" || chr == "A" {
                    let motion = match chr {
                        "a" => Movement::Right,
                        "A" => Movement::RightOfLine,
                        _ => unreachable!(),
                    };
                    ctx.do_core_event(ViewEvent::Move(motion).into(), 1, &mut update);
                } else if chr == "o" {
                    ctx.do_core_event(
                        ViewEvent::Move(Movement::RightOfLine).into(),
                        1,
                        &mut update,
                    );
                    ctx.do_core_event(BufferEvent::InsertNewline.into(), 1, &mut update);
                } else if chr == "O" {
                    ctx.do_core_event(ViewEvent::Move(Movement::LeftOfLine).into(), 1, &mut update);
                    ctx.do_core_event(BufferEvent::InsertNewline.into(), 1, &mut update);
                    ctx.do_core_event(ViewEvent::Move(Movement::Up).into(), 1, &mut update);
                }
                ctx.send_client_rpc("mode_change", json!({"mode": "insert"}));
                ctx.send_client_rpc("parse_state", json!({"state": ""}));
                return Some(update.build());
            }
        }

        self.state = match &self.state {
            CommandState::Ready => {
                self.raw.push_str(chr);
                if let Some(motion) = movement_from_str(chr) {
                    CommandState::Done(Command { motion, ty: CommandType::Move, distance: 1 })
                } else if let Some(cmd) = CommandType::from_char(chr) {
                    CommandState::AwaitMotion(cmd, 0)
                } else if let Ok(num) = chr.parse() {
                    CommandState::AwaitMotion(CommandType::Move, num)
                } else {
                    CommandState::Failed
                }
            }

            CommandState::AwaitMotion(ty, dist) => {
                self.raw.push_str(chr);
                if let Some(motion) = movement_from_str(chr) {
                    CommandState::Done(Command { motion, ty: *ty, distance: *dist.max(&1) })
                } else if let Ok(num) = chr.parse::<usize>() {
                    let new_dist = num + (dist * 10);
                    CommandState::AwaitMotion(*ty, new_dist)
                } else {
                    CommandState::Failed
                }
            }
            _ => unreachable!(),
        };

        if let CommandState::Done(Command { motion, ty, distance }) = &self.state {
            if let CommandType::Delete = ty {
                ctx.do_core_event(
                    ViewEvent::ModifySelection(*motion).into(),
                    *distance,
                    &mut update,
                );
                ctx.do_core_event(BufferEvent::Backspace.into(), 1, &mut update);
            } else {
                ctx.do_core_event(ViewEvent::Move(*motion).into(), *distance, &mut update);
            }
            self.change_state(CommandState::Ready, ctx);
            Some(update.build())
        } else if let CommandState::Failed = self.state {
            self.change_state(CommandState::Ready, ctx);
            None
        } else {
            ctx.send_client_rpc("parse_state", json!({"state": &self.raw}));
            None
        }
    }

    fn change_state(&mut self, new_state: CommandState, ctx: &mut EventCtx) {
        self.state = new_state;
        ctx.send_client_rpc("parse_state", json!({"state": &self.raw}));
        self.raw.clear();
    }

    fn handle_visual(&mut self, event: &KeyEvent, ctx: &mut EventCtx) -> Option<Update> {
        None
    }
}
