use crate::{Reflection, WalletError, WalletResult};

/// A data URI containing a base64-encoded SVG, WebP, PNG, or GIF image.
pub(crate) struct WalletIcon;

impl WalletIcon {
    /// Parse the wallet from a [web_sys::wasm_bindgen::JsValue]
    pub(crate) fn from_jsvalue(reflection: &Reflection) -> WalletResult<Option<String>> {
        let icon = match reflection.string("icon") {
            Ok(icon) => Option::Some(icon),
            Err(error) => match error {
                WalletError::InternalError(_) => Option::None,
                _ => {
                    return Err(error);
                }
            },
        };

        Ok(icon)
    }
}
