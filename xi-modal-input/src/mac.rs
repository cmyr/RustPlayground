use super::{EventHandler, Handler, KeyEvent, PendingToken};

struct Modifiers {
    cmd: bool,
    opt: bool,
    ctrl: bool,
}

impl Modifiers {
    fn from_mask(mask: u32) -> Self {
        let ctrl = mask & (1 << 18) != 0;
        let opt = mask & (1 << 19) != 0;
        let cmd = mask & (1 << 20) != 0;
        Modifiers { cmd, opt, ctrl }
    }

    fn canonical_str(&self) -> &'static str {
        let Modifiers { cmd, opt, ctrl } = self;
        match (cmd, opt, ctrl) {
            (true, true, true) => "cmd+opt+ctrl",
            (true, true, false) => "cmd+opt",
            (true, false, true) => "cmd+ctrl",
            (false, true, true) => "opt+ctrl",
            (true, false, false) => "cmd",
            (false, true, false) => "opt",
            (false, false, true) => "ctrl",
            (false, false, false) => "",
        }
    }
}

pub struct Mac;

impl Handler for Mac {

    fn handle_event(&mut self, event: KeyEvent, handler: &EventHandler) {
        let modifiers = Modifiers::from_mask(event.modifiers);
        match (modifiers.canonical_str(), event.characters) {
            ("ctrl", "a") =>	do_movement("moveToBeginningOfParagraph:", handler),
            ("ctrl", "b") =>	do_movement("moveBackward:", handler),
            ("ctrl", "d") =>	do_movement("deleteForward:", handler),
            ("ctrl", "e") =>	do_movement("moveToEndOfParagraph:", handler),
            ("ctrl", "f") =>	do_movement("moveForward:", handler),
            ("ctrl", "h") =>	do_movement("deleteBackward:", handler),
            ("ctrl", "k") =>	do_movement("deleteToEndOfParagraph:", handler),
            ("ctrl", "l") =>	do_movement("centerSelectionInVisibleArea:", handler),
            ("ctrl", "n") =>	do_movement("moveDown:", handler),
            ("ctrl", "p") =>	do_movement("moveUp:", handler),
            ("ctrl", "t") =>	do_movement("transpose:", handler),
            ("ctrl", "v") =>	do_movement("pageDown:", handler),
            _else => {
                handler.send_event(event);
                return;
            }
        }

        handler.free_event(event);
    }

    fn clear_pending(&mut self, _token: PendingToken) {  }
}

fn do_movement(selector: &str, handler: &EventHandler) {
    handler.send_action("selector", Some(json!({"sel": selector})));
}

