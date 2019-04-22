use std::fmt;

use rustup::Error as RustupError;


#[derive(Debug)]
pub enum Error {
    RustupError(RustupError),
    ToolchainParseError(String),
    //MissingRustup,
    //MissingToolchainsDir,
}

impl From<RustupError> for Error {
    fn from(src: RustupError) -> Error {
        Error::RustupError(src)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            RustupError(e) => e.fmt(f),
            ToolchainParseError(e) => write!(f, "Error parsing toolchain: {}", e),
        }
    }
}

impl std::error::Error for Error {}
