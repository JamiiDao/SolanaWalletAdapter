use std::borrow::Cow;

use crate::{Reflection, WalletError, WalletResult};

use wallet_adapter_common::SemverVersion as SemverVersionData;
/// Semver Versioning struct
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SemverVersion(pub SemverVersionData);

impl SemverVersion {
    /// The major version
    pub fn major(&self) -> u8 {
        self.0.major()
    }

    /// The minor version
    pub fn minor(&self) -> u8 {
        self.0.minor()
    }

    /// The patch version
    pub fn patch(&self) -> u8 {
        self.0.patch()
    }

    /// Parse the version from a [web_sys::wasm_bindgen::JsValue]
    pub(crate) fn from_jsvalue(reflection: &Reflection) -> WalletResult<Self> {
        let version = reflection
            .reflect_inner("version")
            .or(Err(WalletError::VersionNotFound))?
            .as_string()
            .ok_or(WalletError::InternalError(
                "Expected `version` JsValue to be a String".to_string(),
            ))?;

        SemverVersion::parse(&version)
    }

    /// Parse a semver versioned string  into [Self]
    pub fn parse(version: &str) -> WalletResult<Self> {
        let chunks = version.split(".").collect::<Vec<&str>>();

        if chunks.len() != 3 {
            return Err(WalletError::InvalidWalletVersion(version.to_string()));
        }

        let version_chunks = chunks
            .iter()
            .map(|chunk| {
                chunk
                    .parse::<u8>()
                    .map_err(|_| WalletError::InvalidSemVerNumber(chunk.to_string()))
            })
            .collect::<WalletResult<Vec<u8>>>()?;

        Ok(Self(
            SemverVersionData::new()
                .set_major(version_chunks[0])
                .set_minor(version_chunks[1])
                .set_patch(version_chunks[2]),
        ))
    }

    /// Get the string version of [Self] in the format `major.minor.patch`
    pub fn stringify_version<'a>(&'a self) -> Cow<'a, str> {
        self.0.stringify_version()
    }
}

impl core::fmt::Display for SemverVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
