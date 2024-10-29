use std::borrow::Cow;

use wasm_bindgen::{JsCast, JsValue};
use web_sys::js_sys::Reflect;

use crate::{WalletAccount, WalletError, WalletResult};

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Wallet {
    accounts: Vec<WalletAccount>,
    chains: Vec<String>,
    // features: Feature,
    icon: Option<WalletIcon>,
    name: String,
    version: SemverVersion,
}

impl Wallet {
    pub fn from_jsvalue(value: JsValue) -> WalletResult<Self> {
        let reflection = Reflection::new(value);

        let wallet_name = reflection.string("name")?;

        let wallet_version = reflection.string("version")?;

        let icon = WalletIcon::from_jsvalue(&reflection)?;

        let accounts = Self::get_accounts(&reflection, "accounts")?;

        let chains = reflection.vec_string("chains")?;

        let mut wallet = Self::default();
        wallet.name = wallet_name;
        wallet.version = SemverVersion::parse(&wallet_version)?;
        wallet.icon = icon;
        wallet.accounts = accounts;
        wallet.chains = chains;

        Ok(wallet)
    }

    pub fn get_accounts(reflection: &Reflection, key: &str) -> WalletResult<Vec<WalletAccount>> {
        let accounts_raw = Reflect::get(&reflection.0, &key.into())?;

        Reflection::check_is_undefined(&accounts_raw)?;

        if !accounts_raw.is_array() {
            return Err(WalletError::ExpectedArray(
                "Reflection for `accounts` key".to_string(),
            ));
        }

        let accounts_array: js_sys::Array = accounts_raw.unchecked_into();

        accounts_array
            .iter()
            .map(|account| WalletAccount::parse(&Reflection(account)))
            .collect::<WalletResult<Vec<WalletAccount>>>()
    }

    pub fn accounts(&self) -> &[WalletAccount] {
        &self.accounts
    }

    pub fn chains(&self) -> &[String] {
        &self.chains
    }

    pub fn icon(&self) -> Option<&WalletIcon> {
        self.icon.as_ref()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &SemverVersion {
        &self.version
    }
}

#[derive(Debug)]
pub struct Reflection(JsValue);

impl Reflection {
    pub fn new(value: JsValue) -> Self {
        Self(value)
    }

    pub fn string(&self, key: &str) -> WalletResult<String> {
        let name = Reflect::get(&self.0, &key.into())?;

        Reflection::check_is_undefined(&name)?;

        let parsed = name.as_string().ok_or(WalletError::JsValueNotString)?;

        Ok(parsed)
    }

    pub fn byte32array(&self, key: &str) -> WalletResult<[u8; 32]> {
        let js_value = Reflect::get(&self.0, &key.into())?;

        Reflection::check_is_undefined(&js_value)?;

        if !js_value.is_array() {
            return Err(WalletError::ExpectedArray(key.to_string()));
        }

        let to_js_array: js_sys::Uint8Array = js_value.unchecked_into();

        let byte32array: [u8; 32] = to_js_array
            .to_vec()
            .try_into()
            .or(Err(WalletError::Expected32ByteLength))?;

        Ok(byte32array)
    }

    pub fn vec_string(&self, key: &str) -> WalletResult<Vec<String>> {
        let js_value = Reflect::get(&self.0, &key.into())?;

        Reflection::check_is_undefined(&js_value)?;

        if !js_value.is_array() {
            return Err(WalletError::ExpectedArray(key.to_string()));
        }

        let to_js_array: js_sys::Array = js_value.unchecked_into();

        to_js_array
            .iter()
            .map(|value| value.as_string().ok_or(WalletError::JsValueNotString))
            .collect::<WalletResult<Vec<String>>>()
    }

    pub fn check_is_undefined(value: &JsValue) -> WalletResult<()> {
        if value.is_undefined() || value.is_null() {
            return Err(WalletError::ValueNotFound);
        } else {
            Ok(())
        }
    }
}

/// A data URI containing a base64-encoded SVG, WebP, PNG, or GIF image.
#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WalletIcon(
    /// Format `data:image/${'svg+xml' | 'webp' | 'png' | 'gif'};base64,${string}`
    pub Cow<'static, str>,
);

impl WalletIcon {
    pub fn from_jsvalue(reflection: &Reflection) -> WalletResult<Option<WalletIcon>> {
        let icon = match reflection.string("icon") {
            Ok(icon) => Option::Some(WalletIcon(Cow::Owned(icon))),
            Err(error) => {
                if error == WalletError::JsValueNotString {
                    Option::None
                } else {
                    return Err(error);
                }
            }
        };

        Ok(icon)
    }
}

impl core::fmt::Debug for WalletIcon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = if let Some((first, _)) = self.0.split_once(",") {
            first
        } else {
            &self.0
        };

        write!(f, "{value}",)
    }
}

#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SemverVersion {
    major: u8,
    minor: u8,
    patch: u8,
}

impl SemverVersion {
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
