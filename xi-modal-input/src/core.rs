//! Message routing etc

use xi_core_lib::edit_types::EventDomain;
use xi_core_lib::rpc::EditNotification;

use crate::callbacks::{InvalidateCallback, RpcCallback};
use crate::input_handler::{Handler, Plumber};
use crate::rpc::Rpc;
use crate::update::Update;
use crate::view::OneView;

pub struct XiCore {
    pub rpc_callback: RpcCallback,
    pub invalidate_callback: InvalidateCallback,
    pub state: OneView,
    pub plumber: Option<Plumber>,
    pub handler: Option<Box<dyn Handler>>,
}

impl XiCore {
    pub fn new<R, I>(rpc: R, invalidate: I, state: OneView) -> Self
    where
        I: Into<InvalidateCallback>,
        R: Into<RpcCallback>,
    {
        let invalidate_callback = invalidate.into();
        let rpc_callback = rpc.into();
        XiCore { invalidate_callback, rpc_callback, state, plumber: None, handler: None }
    }

    pub fn handle_message(&mut self, msg: &str) {
        use xi_core_lib::rpc::*;
        use EditNotification as E;
        let msg: Rpc = match serde_json::from_str(msg) {
            Ok(rpc) => rpc,
            Err(e) => {
                eprintln!("invalid json '{}': {}", msg, e);
                return;
            }
        };

        eprintln!("core handle_msg {:?}", msg.method);
        let event = match msg.method {
            "insert" => E::Insert { chars: msg.params["chars"].as_str().unwrap().to_owned() },
            "viewport_change" => E::ViewportChange(msg.get_params()),
            "gesture" => {
                #[derive(Deserialize)]
                struct Params {
                    line: u64,
                    col: u64,
                    ty: GestureType,
                };
                let Params { line, col, ty } = msg.get_params();
                E::Gesture { line, col, ty }
            }
            other => match event_from_str(other) {
                Some(event) => event,
                None => {
                    eprintln!("no event for '{}'", other);
                    return;
                }
            },
        };

        let domain: EventDomain = event.into();
        let update = self.state.handle_event(domain);
        self.send_update(update);
    }

    pub fn send_update(&self, mut update: Update) {
        if let Some(newsize) = update.size.take() {
            eprintln!("newsize: {:?}", newsize);
            self.rpc_callback
                .call("content_size", json!({"width": newsize.width, "height": newsize.height}));
        }

        if let Some(styles) = update.styles.take() {
            self.rpc_callback.call("new_styles", json!({ "styles": styles }));
        }

        if let Some(range) = update.lines.take() {
            self.invalidate_callback.call(range);
        }

        if let Some(scroll) = update.scroll.take() {
            self.rpc_callback.call("scroll_to", scroll)
        }

        if let Some(pasteboard) = update.pasteboard.take() {
            self.rpc_callback.call("set_pasteboard", json!({ "text": pasteboard }))
        }
    }
}

fn event_from_str(string: &str) -> Option<EditNotification> {
    use EditNotification as E;
    match string {
        "deleteBackward:" => Some(E::DeleteBackward),
        "deleteForward:" => Some(E::DeleteForward),
        "deleteToBeginningOfLine:" => Some(E::DeleteToBeginningOfLine),
        "deleteToEndOfParagraph:" => Some(E::DeleteToEndOfParagraph),
        "deleteWordBackward:" => Some(E::DeleteWordBackward),
        "deleteWordForward:" => Some(E::DeleteForward),
        "insertNewline:" => Some(E::InsertNewline),
        "insertTab:" => Some(E::InsertTab),
        "moveBackward:" => Some(E::MoveBackward),
        "moveDown:" => Some(E::MoveDown),
        "moveDownAndModifySelection:" => Some(E::MoveDownAndModifySelection),
        "moveForward:" => Some(E::MoveForward),
        "moveLeft:" => Some(E::MoveLeft),
        "moveLeftAndModifySelection:" => Some(E::MoveLeftAndModifySelection),
        "moveRight:" => Some(E::MoveRight),
        "moveRightAndModifySelection:" => Some(E::MoveRightAndModifySelection),
        "moveToBeginningOfDocument:" => Some(E::MoveToBeginningOfDocument),
        "moveToBeginningOfDocumentAndModifySelection:" => {
            Some(E::MoveToBeginningOfDocumentAndModifySelection)
        }
        "moveToBeginningOfLine:" => Some(E::MoveToLeftEndOfLine),
        "moveToBeginningOfLineAndModifySelection:" => {
            Some(E::MoveToLeftEndOfLineAndModifySelection)
        }
        "moveToBeginningOfParagraph:" => Some(E::MoveToBeginningOfParagraph),
        "moveToBeginningOfParagraphAndModifySelection:" => {
            Some(E::MoveToBeginningOfParagraphAndModifySelection)
        }
        "moveToEndOfDocument:" => Some(E::MoveToEndOfDocument),
        "moveToEndOfDocumentAndModifySelection:" => Some(E::MoveToEndOfDocumentAndModifySelection),
        "moveToEndOfLine:" => Some(E::MoveToRightEndOfLine),
        "moveToEndOfLineAndModifySelection:" => Some(E::MoveToRightEndOfLineAndModifySelection),
        "moveToEndOfParagraph:" => Some(E::MoveToEndOfParagraph),
        "moveToEndOfParagraphAndModifySelection:" => {
            Some(E::MoveToEndOfParagraphAndModifySelection)
        }
        "moveToLeftEndOfLine:" => Some(E::MoveToLeftEndOfLine),
        "moveToLeftEndOfLineAndModifySelection:" => Some(E::MoveToLeftEndOfLineAndModifySelection),
        "moveToRightEndOfLine:" => Some(E::MoveToRightEndOfLine),
        "moveToRightEndOfLineAndModifySelection:" => {
            Some(E::MoveToRightEndOfLineAndModifySelection)
        }
        "moveUp:" => Some(E::MoveUp),
        "moveUpAndModifySelection:" => Some(E::MoveUpAndModifySelection),
        "moveWordLeft:" => Some(E::MoveWordLeft),
        "moveWordLeftAndModifySelection:" => Some(E::MoveWordLeftAndModifySelection),
        "moveWordRight:" => Some(E::MoveWordRight),
        "moveWordRightAndModifySelection:" => Some(E::MoveWordRightAndModifySelection),
        "pageDownAndModifySelection:" => Some(E::PageDownAndModifySelection),
        "pageUpAndModifySelection:" => Some(E::PageUpAndModifySelection),
        "transpose:" => Some(E::Transpose),
        "selectAll:" => Some(E::SelectAll),
        "cancelOperation:" => Some(E::CollapseSelections),
        "copy" => Some(E::CopyAsync),
        "cut" => Some(E::CutAsync),
        "undo" => Some(E::Undo),
        "redo" => Some(E::Redo),
        "toggle_comment" => Some(E::ToggleComment),
        _other => None,
        //(Some("scrollPageDown:"), None) => E::ScrollPageDown
        //(Some("scrollPageUp:"), None) =>
        //(Some("scrollToBeginningOfDocument:"), None) =>
        //(Some("scrollToEndOfDocument:"), None) =>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_smoke_test() {
        let rpc_json = json!({"method": "hello", "params": {"foo": "bar"}});
        let rpc_str = serde_json::to_string(&rpc_json).unwrap();
        let rpc: Rpc = serde_json::from_str(&rpc_str).unwrap();
        assert_eq!(rpc.method, "hello");
        assert_eq!(rpc.params.as_object().unwrap().get("foo").unwrap().as_str(), Some("bar"));
    }
}
