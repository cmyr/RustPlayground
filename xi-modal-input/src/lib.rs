extern crate libc;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate syntect;

mod callbacks;
mod core;
mod gesture;
mod highlighting;
mod input_handler;
mod lines;
mod rpc;
mod style;
mod update;
mod view;
mod vim;

pub use crate::core::XiCore;
pub use input_handler::{EventCtx, EventPayload, Handler, KeyEvent, Plumber};
pub use lines::Size;
pub use view::{Line, OneView};
pub use vim::Machine as Vim;
