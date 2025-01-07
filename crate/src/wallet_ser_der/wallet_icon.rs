use std::borrow::Cow;

use crate::{Reflection, WalletResult};

/// A data URI containing a base64-encoded SVG, WebP, PNG, or GIF image.
#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WalletIcon(
    /// Format `data:image/${'svg+xml' | 'webp' | 'png' | 'gif'};base64,${string}`
    pub Cow<'static, str>,
);

impl WalletIcon {
    /// Parse the wallet from a [wasm_bindgen::JsValue]
    pub(crate) fn from_jsvalue(reflection: &Reflection) -> WalletResult<Option<WalletIcon>> {
        reflection
            .reflect_string("icon")
            .map(|icon| Option::Some(WalletIcon(Cow::Owned(icon))))
    }
}

impl core::fmt::Display for WalletIcon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
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
