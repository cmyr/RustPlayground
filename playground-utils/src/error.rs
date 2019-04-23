use std::fmt;
use std::io;
use std::path::PathBuf;
use std::process::Output;

#[derive(Debug)]
pub enum Error {
    ToolchainParseError(String),
    MissingRustup,
    ReadingToolchainsDir,
    CreateOutputFailed(PathBuf),
    CommandFailed(io::Error),
    BadExit(String),
}

impl Error {
    /// Generates an error from the result of process::Command::output.
    /// Prelude is a short message describing the particular failure.
    pub fn bad_output(prelude: &str, output: &Output) -> Self {
        debug_assert!(!output.status.success());

        let mut err_string = String::from(prelude);
        if let Some(code) = output.status.code() {
            err_string.push_str(&format!(" Exit code {}.", code));
        }
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.is_empty() {
            err_string.pop();
            err_string.push_str(&format!(", '{}'.", stderr));
        }
        Error::BadExit(err_string)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            MissingRustup => write!(f, "Cannot find rustup."),
            ReadingToolchainsDir => write!(f, "Cannot read toolchains directory."),
            ToolchainParseError(e) => write!(f, "Error parsing toolchain: '{}'.", e),
            CreateOutputFailed(p) => {
                write!(f, "Failed to create output path at '{}'.", p.to_string_lossy())
            }
            CommandFailed(s) => write!(f, "Failed to execute command '{}'.", s),
            BadExit(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for Error {}
