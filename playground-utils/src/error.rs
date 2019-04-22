use rustup::Error as RustupError;

pub enum Error {
    RustupError(RustupError),
    ToolchainParseError(String),
    MissingRustup,
    MissingToolchainsDir,
}

impl From<RustupError> for Error {
    fn from(src: RustupError) -> Error {
        Error::RustupError(src)
    }
}
