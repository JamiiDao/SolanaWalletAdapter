use std::borrow::Cow;

use wasm_bindgen::JsValue;
use web_sys::js_sys::Reflect;

use crate::{WalletError, WalletResult};

#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SemverVersion {
    major: u8,
    minor: u8,
    patch: u8,
}

impl SemverVersion {
    pub fn from_jsvalue(value: &JsValue) -> WalletResult<Self> {
        let reflect_value =
            Reflect::get(value, &"version".into()).or(Err(WalletError::VersionNotFound))?;

        let version = reflect_value
            .as_string()
            .ok_or(WalletError::ExpectedString(
                "Expected `version` JsValue to be a String".to_string(),
            ))?;

        SemverVersion::parse(&version)
    }
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

        Ok(Self {
            major: version_chunks[0],
            minor: version_chunks[1],
            patch: version_chunks[2],
        })
    }

    pub fn get_version(&self) -> &Self {
        self
    }

    pub fn stringify_version(&self) -> Cow<str> {
        Cow::Borrowed("")
            + Cow::Owned(self.major.to_string())
            + "."
            + Cow::Owned(self.minor.to_string())
            + "."
            + Cow::Owned(self.minor.to_string())
    }
}

impl core::fmt::Debug for SemverVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SemverVersion({}.{}.{})",
            self.major, self.minor, self.patch
        )
    }
}

impl core::fmt::Display for SemverVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}
