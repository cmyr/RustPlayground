use std::fmt;

#[derive(Debug)]
pub enum Error {
    ToolchainParseError(String),
    MissingRustup,
    ReadingToolchainsDir,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            MissingRustup => write!(f, "Cannot find rustup."),
            ReadingToolchainsDir => write!(f, "Cannot read toolchains directory."),
            ToolchainParseError(e) => write!(f, "Error parsing toolchain: '{}'.", e),
        }
    }
}

impl std::error::Error for Error {}
