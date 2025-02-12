use core::hash::Hash;

use web_sys::js_sys::Function;

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
    pub(crate) fn new(
        reflection: &Reflection,
        version: SemverVersion,
        key: &str,
        namespace: &str,
    ) -> WalletResult<Self> {
        let incase_of_error = Err(WalletError::InternalError(format!(
            "Namespace[`{namespace}: {key} -> {key}]: Reflect `{key}` in JsValue `{:?}` did not yield a JS Function", reflection.get_inner()
        )));

        let fn_value = reflection
            .reflect_inner(key)
            .or(Err(WalletError::MissingConnectFunction))?;
        let get_fn = Reflection::new(fn_value)?
            .as_function_owned()
            .or(incase_of_error)?;

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
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.version.hash(state);
    }
}
