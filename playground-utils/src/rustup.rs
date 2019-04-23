//Copyright (c) 2016 The Rust Project Developers

//Permission is hereby granted, free of charge, to any
//person obtaining a copy of this software and associated
//documentation files (the "Software"), to deal in the
//Software without restriction, including without
//limitation the rights to use, copy, modify, merge,
//publish, distribute, sublicense, and/or sell copies of
//the Software, and to permit persons to whom the Software
//is furnished to do so, subject to the following
//conditions:

//The above copyright notice and this permission notice
//shall be included in all copies or substantial portions
//of the Software.

//THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
//ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
//TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
//PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
//SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
//CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
//OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
//IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
//DEALINGS IN THE SOFTWARE.

//! The contents of this file is adapted from rustup (https://github.com/rust-lang/rustup.rs),
//! and is used under the terms of th license included above.

use std::env;
use std::fs;

use std::path::{Path, PathBuf};

use crate::error::Error;

pub(crate) fn list_toolchains() -> Result<Vec<String>, Error> {
    let toolchains_dir = get_toolchains_dir()?;
    if is_directory(&toolchains_dir) {
        let mut toolchains = fs::read_dir(&toolchains_dir)
            .map_err(|_| Error::ReadingToolchainsDir)?
            .filter_map(std::io::Result::ok)
            .filter(|e| e.file_type().map(|f| !f.is_file()).unwrap_or(false))
            .filter_map(|e| e.file_name().into_string().ok())
            .collect::<Vec<_>>();

        toolchain_sort(&mut toolchains);

        Ok(toolchains)
    } else {
        Err(Error::ReadingToolchainsDir)
    }
}

fn is_directory<P: AsRef<Path>>(path: P) -> bool {
    fs::metadata(path).ok().as_ref().map(fs::Metadata::is_dir) == Some(true)
}

fn get_toolchains_dir() -> Result<PathBuf, Error> {
    get_rustup_home().map(|p| p.join("toolchains"))
}

fn get_rustup_home() -> Result<PathBuf, Error> {
    fn get_rustup_home_path() -> Result<PathBuf, Error> {
        if let Some(home) = env::var_os("RUSTUP_HOME") {
            return Ok(home.into());
        }
        dirs::home_dir().map(|p| p.join(".rustup")).ok_or_else(|| Error::MissingRustup)
    }

    let rustup_home = get_rustup_home_path()?;
    if is_directory(&rustup_home) {
        Ok(rustup_home)
    } else {
        Err(Error::MissingRustup)
    }
}

fn toolchain_sort<T: AsRef<str>>(v: &mut Vec<T>) {
    use semver::{Identifier, Version};

    fn special_version(ord: u64, s: &str) -> Version {
        Version {
            major: 0,
            minor: 0,
            patch: 0,
            pre: vec![Identifier::Numeric(ord), Identifier::AlphaNumeric(s.into())],
            build: vec![],
        }
    }

    fn toolchain_sort_key(s: &str) -> Version {
        use crate::toolchain::NATIVE_TOOLCHAIN;

        // we want nightly-x86_etc to order before nightly-date-x86_etc
        let s = s.trim_end_matches(NATIVE_TOOLCHAIN);

        if s.starts_with("stable") {
            special_version(0, s)
        } else if s.starts_with("beta") {
            special_version(1, s)
        } else if s.starts_with("nightly") {
            special_version(2, s)
        } else {
            Version::parse(&s.replace("_", "-")).unwrap_or_else(|_| special_version(3, s))
        }
    }

    v.sort_by(|a, b| {
        let a_str: &str = a.as_ref();
        let b_str: &str = b.as_ref();
        let a_key = toolchain_sort_key(a_str);
        let b_key = toolchain_sort_key(b_str);
        a_key.cmp(&b_key)
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sort_toolchains() {
        let mut toolchains = vec![
            "nightly-2019-01-26-x86_64-apple-darwin",
            "stable-x86_64-apple-darwin",
            "nightly-x86_64-apple-darwin",
            "1.31.0-x86_64-apple-darwin",
        ];

        toolchain_sort(&mut toolchains);

        assert_eq!(
            toolchains,
            vec![
                "stable-x86_64-apple-darwin",
                "nightly-x86_64-apple-darwin",
                "nightly-2019-01-26-x86_64-apple-darwin",
                "1.31.0-x86_64-apple-darwin",
            ]
        );
    }
}
