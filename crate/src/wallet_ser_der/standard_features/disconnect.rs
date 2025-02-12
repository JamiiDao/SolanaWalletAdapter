use web_sys::{js_sys, wasm_bindgen::JsValue};

use crate::{Reflection, SemverVersion, StandardFunction, WalletError, WalletResult};

/// `standard:disconnect` struct containing the `version` and `callback`
/// in the field [StandardFunction]
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Disconnect(StandardFunction);

impl Disconnect {
    /// Parse the `standard:disconnect` callback from the [JsValue]
    pub(crate) fn new(reflection: &Reflection, version: SemverVersion) -> WalletResult<Self> {
        Ok(Self(StandardFunction::new(
            reflection,
            version,
            "disconnect",
            "standard",
        )?))
    }

    /// Calling this method disconnects the wallet by internally calling the
    /// callback function
    pub(crate) async fn call_disconnect(&self) -> WalletResult<()> {
        let outcome = self.0.callback.call0(&JsValue::null())?;

        let outcome = js_sys::Promise::resolve(&outcome);

        if let Some(error) = wasm_bindgen_futures::JsFuture::from(outcome).await.err() {
            let value: WalletError = error.into();
            return Err(WalletError::WalletDisconnectError(value.to_string()));
        }

        Ok(())
    }
}
