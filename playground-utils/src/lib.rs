#[macro_use]
extern crate serde_derive;

mod error;
mod rustup;
mod toolchain;

#[cfg(not(target_os = "macos"))]
compile_error!("this library is currently macOS only.");

pub use toolchain::{list_toolchains, ToolchainInfo};
