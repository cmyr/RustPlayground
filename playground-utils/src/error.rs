use std::fmt;
use std::io;
use std::path::PathBuf;
use std::process::Output;

#[derive(Debug)]
pub enum Error {
    ToolchainParseError(String),
    MissingRustup,
    ReadingToolchainsDir,
    CompileFailed(io::Error),
    ToolchainSelectFailed(io::Error),
    CreateOutputFailed(PathBuf),
    MalformedDependency(String),
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

    /// An error code included in the json if this is sent to core.
    /// This is not systematic. We just want to be able to easily identify
    /// certain cases, such as when Rustup is not installed.
    pub fn error_code(&self) -> u32 {
        use Error::*;
        match self {
            BadExit(_) => 1,
            MissingRustup => 10,
            MalformedDependency(_) => 30,
            _ => 2, // catchall; we can add these as we need them.
        }
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
            CompileFailed(s) => write!(f, "Compiler command failed: '{}'.", s),
            ToolchainSelectFailed(s) => write!(f, "Toolchain select failed: '{}'.", s),
            BadExit(msg) => write!(f, "{}", msg),
            MalformedDependency(s) => write!(
                f,
                "Malformed dependency '{}'. Inline dependencies must \n\
                 be in the form, 'use crate_name [= x[.y.z]]'.",
                s
            ),
        }
    }
}

impl std::error::Error for Error {}
