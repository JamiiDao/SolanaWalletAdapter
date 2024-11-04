use std::hash::Hash;

use js_sys::Function;
use wasm_bindgen::{JsCast, JsValue};

use crate::{Reflection, SemverVersion, WalletAccount, WalletError, WalletResult};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Connect {
    version: SemverVersion,
    callback: Function,
}
impl Connect {
    pub fn new(value: JsValue, version: SemverVersion) -> WalletResult<Self> {
        let get_connect_value = Reflection::new(value)?
            .reflect_inner(&"connect")
            .or(Err(WalletError::MissingConnectFunction))?;
        let get_connect_fn =
            get_connect_value
                .dyn_into::<Function>()
                .or(Err(WalletError::JsValueNotFunction(
                    "Namespace[`standard:connect -> connect`]".to_string(),
                )))?;

        Ok(Self {
            version,
            callback: get_connect_fn,
        })
    }

    pub(crate) async fn call_connect(&self) -> WalletResult<Vec<WalletAccount>> {
        let outcome = self.callback.call0(&JsValue::from_bool(false))?;

        let outcome = js_sys::Promise::resolve(&outcome);

        match wasm_bindgen_futures::JsFuture::from(outcome).await {
            Ok(success) => {
                let get_accounts = Reflection::new(success)?.get_js_array("accounts")?;

                get_accounts
                    .into_iter()
                    .map(|raw_account| WalletAccount::parse(&Reflection::new(raw_account)?))
                    .collect::<WalletResult<Vec<WalletAccount>>>()
            }
            Err(error) => {
                let value: WalletError = error.into();
                return Err(WalletError::WalletConnectError(value.to_string()));
            }
        }
    }
}

impl PartialOrd for Connect {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.version.cmp(&other.version))
    }
}

impl Ord for Connect {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.version.cmp(&other.version)
    }
}

impl Hash for Connect {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.version.hash(state);
    }
}
