#[macro_use]
extern crate serde_derive;

mod compile;
mod error;
mod rustup;
mod toolchain;

#[cfg(not(target_os = "macos"))]
compile_error!("this library is currently macOS only.");

pub use compile::{do_compile_task, Task};
pub use toolchain::{list_toolchains, ToolchainInfo};
