use js_sys::{Object, Reflect};
use wasm_bindgen::{JsCast, JsValue};

use crate::{WalletError, WalletResult};

/// A 32 byte array representing a Public Key
pub type PublicKey = [u8; 32];

/// A 64 byte array represnting a Signature
pub type Signature = [u8; 64];

/// The Version of the Wallet Standard currently implemented.
/// This may be used by the app to determine compatibility and feature detect.
pub const WALLET_STANDARD_VERSION: &str = "1.0.0";

pub struct Utils;

impl Utils {
    pub fn jsvalue_to_error<T: core::fmt::Debug>(
        value: Result<T, JsValue>,
    ) -> Result<(), WalletError> {
        if let Err(error) = value {
            Err(error.into())
        } else {
            Ok(())
        }
    }
}

#[derive(Debug)]
pub struct Reflection(JsValue);

impl Reflection {
    pub fn new(value: JsValue) -> WalletResult<Self> {
        Reflection::check_is_undefined(&value)?;

        Ok(Self(value))
    }

    pub fn new_from_str(value: &JsValue, key: &str) -> WalletResult<Self> {
        let inner = Reflect::get(&value, &key.into())?;

        Reflection::new(inner)
    }

    pub fn reflect_inner(&self, key: &str) -> WalletResult<JsValue> {
        let inner = Reflect::get(&self.0, &key.into())?;

        Reflection::check_is_undefined(&inner)?;

        Ok(inner)
    }

    pub fn string(&self, key: &str) -> WalletResult<String> {
        let name = Reflect::get(&self.0, &key.into())?;

        let parsed = name.as_string().ok_or(WalletError::JsValueNotString)?;

        Ok(parsed)
    }

    pub fn byte32array(&self, key: &str) -> WalletResult<[u8; 32]> {
        let js_value = Reflect::get(&self.0, &key.into())?;

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

        if !js_value.is_array() {
            return Err(WalletError::ExpectedArray(key.to_string()));
        }

        let to_js_array: js_sys::Array = js_value.unchecked_into();

        to_js_array
            .iter()
            .map(|value| value.as_string().ok_or(WalletError::JsValueNotString))
            .collect::<WalletResult<Vec<String>>>()
    }

    pub fn vec_string_and_filter(&self, key: &str, filter: &str) -> WalletResult<Vec<String>> {
        let js_value = Reflect::get(&self.0, &key.into())?;

        if !js_value.is_array() {
            return Err(WalletError::ExpectedArray(key.to_string()));
        }

        let to_js_array: js_sys::Array = js_value.unchecked_into();

        to_js_array
            .iter()
            .map(|value| value.as_string().ok_or(WalletError::JsValueNotString))
            .map(|value| {
                let value = value?;

                if value.starts_with(filter) {
                    Ok(value)
                } else {
                    Err(WalletError::UnsupportedChain(value.to_string()))
                }
            })
            .collect::<WalletResult<Vec<String>>>()
    }

    pub fn object_to_vec_string(&self, key: &str) -> WalletResult<Vec<String>> {
        let features_value = Reflect::get(&self.0, &key.into())?;

        let features_object = features_value
            .dyn_ref::<Object>()
            .ok_or(WalletError::ExpectedObject(key.to_string()))?;

        Object::keys(features_object)
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

    pub fn get_inner(&self) -> &JsValue {
        &self.0
    }
}
