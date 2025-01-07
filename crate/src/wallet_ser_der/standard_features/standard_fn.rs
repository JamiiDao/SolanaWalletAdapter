use core::hash::Hash;

use js_sys::Function;
use wasm_bindgen::JsValue;

use crate::{Reflection, SemverVersion, WalletError, WalletResult};

/// A struct containing the [semver version](SemverVersion)
/// and [callback function](js_sys::Function) within the `standard:` namespace as
/// defined by the wallet standard
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StandardFunction {
    pub(crate) version: SemverVersion,
    pub(crate) callback: Function,
}

impl StandardFunction {
    /// Parse the [semver version](SemverVersion) and the [callback function](js_sys::Function)
    /// given a [JsValue], a [key](str) and a [namespace](str) . The namespace is either
    /// `standard:` or `solana:` as defined by the wallet standard
    pub fn new(
        value: JsValue,
        version: SemverVersion,
        key: &str,
        namespace: &str,
        error: WalletError,
    ) -> WalletResult<Self> {
        let fn_value = Reflection::new(value)?.reflect_inner(key).or(Err(error))?;
        let js_typeof = Reflection::js_typeof(&fn_value);
        let get_fn = Reflection::as_function_owned(fn_value).or(Err(WalletError::JsCast(
            namespace.to_string() + " is not a function but `" + &js_typeof + "`.",
        )))?;

        Ok(Self {
            version,
            callback: get_fn,
        })
    }
}

impl PartialOrd for StandardFunction {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.version.cmp(&other.version))
    }
}

impl Ord for StandardFunction {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.version.cmp(&other.version)
    }
}

impl Hash for StandardFunction {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.version.hash(state);
    }
}
