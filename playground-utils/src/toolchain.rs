use crate::error::Error;
use crate::rustup;

pub(crate) const NATIVE_TOOLCHAIN: &str = "-apple-darwin";

#[derive(Debug, Clone, Serialize)]
pub struct ToolchainInfo {
    /// The full name, e.g. `nightly-2019-01-26-x86_64-apple-darwin`
    name: String,
    /// The channel; stable / nightly / beta
    channel: String,
    /// The date, if present
    date: Option<String>,
}

impl ToolchainInfo {
    pub fn from_name(name: String) -> Result<Self, Error> {
        let trimmed = name.trim_end_matches(NATIVE_TOOLCHAIN).trim_end_matches('-');
        let mut split = trimmed.splitn(2, '-');
        let channel =
            split.next().ok_or_else(|| Error::ToolchainParseError(name.clone()))?.to_owned();
        let date = split.next().map(String::from);

        Ok(ToolchainInfo { name, channel, date })
    }
}

/// Lists the installed toolchains for this target (macos)
pub fn list_toolchains() -> Result<Vec<ToolchainInfo>, Error> {
    let toolchains = rustup::list_toolchains()?;
    let toolchains = toolchains
        .into_iter()
        .filter(|t| t.ends_with(NATIVE_TOOLCHAIN))
        .map(ToolchainInfo::from_name)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(toolchains)
}
