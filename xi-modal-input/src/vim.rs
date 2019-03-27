use super::{EventHandler, Handler, KeyEvent, PendingToken};

const KEY_TIMEOUT_MILLIS: u32 = 500;

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
    StartOfLine,
    EndOfLine,
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
            '0' => Some(Motion::StartOfLine),
            '$' => Some(Motion::EndOfLine),
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
            Motion::StartOfLine => "start_of_line",
            Motion::EndOfLine => "end_of_line",
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
            if ['i', 'a', 'A'].contains(&chr) {
                self.mode = Mode::Insert;
                if chr != 'i' {
                    let motion = if chr == 'a' { "right" } else { "end_of_line" };
                    handler.send_action("move", Some(json!({"motion": motion, "dist": 1})));
                }
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
