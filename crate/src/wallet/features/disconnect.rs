use std::hash::Hash;

use js_sys::Function;
use wasm_bindgen::{JsCast, JsValue};

use crate::{Reflection, SemverVersion, WalletError, WalletResult};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Disconnect {
    version: SemverVersion,
    callback: Function,
}

impl Disconnect {
    pub fn new(value: JsValue, version: SemverVersion) -> WalletResult<Self> {
        let get_disconnect_value = Reflection::new(value)?
            .reflect_inner(&"disconnect")
            .or(Err(WalletError::MissingDisconnectFunction))?;
        let get_disconnect_fn = get_disconnect_value.dyn_into::<Function>().or(Err(
            WalletError::JsValueNotFunction(
                "Namespace[`standard:disconnect -> disconnect`]".to_string(),
            ),
        ))?;

        Ok(Self {
            version,
            callback: get_disconnect_fn,
        })
    }

    pub(crate) async fn call_diconnect(&self) -> WalletResult<()> {
        let outcome = self.callback.call0(&JsValue::null())?;

        let outcome = js_sys::Promise::resolve(&outcome);

        if let Some(error) = wasm_bindgen_futures::JsFuture::from(outcome).await.err() {
            let value: WalletError = error.into();
            return Err(WalletError::WalletDisonnectError(value.to_string()));
        }

        Ok(())
    }
}

impl PartialOrd for Disconnect {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.version.cmp(&other.version))
    }
}

impl Ord for Disconnect {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.version.cmp(&other.version)
    }
}

impl Hash for Disconnect {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.version.hash(state);
    }
}
