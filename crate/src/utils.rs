use wallet_adapter_common::WalletCommonUtils;
use web_sys::{
    js_sys::{self, Array, Function, Object, Reflect},
    wasm_bindgen::{JsCast, JsValue},
};

use crate::{WalletError, WalletResult};

/// Helper utilities
pub struct InnerUtils;

impl InnerUtils {
    /// Convert a [JsValue] error to a [WalletError]
    pub fn jsvalue_to_error<T: core::fmt::Debug>(
        value: Result<T, JsValue>,
    ) -> Result<(), WalletError> {
        if let Err(error) = value {
            Err(error.into())
        } else {
            Ok(())
        }
    }

    /// Convert a [JsValue] to a [ed25519_dalek::Signature]
    pub fn jsvalue_to_signature(
        value: JsValue,
        namespace: &str,
    ) -> WalletResult<ed25519_dalek::Signature> {
        let in_case_of_error = Err(WalletError::InternalError(format!(
            "{namespace}: `{value:?}` cannot be cast to a Uint8Array, only a JsValue of bytes can be cast."
        )));

        let signature_bytes: [u8; 64] = value
            .dyn_into::<js_sys::Uint8Array>()
            .or(in_case_of_error)?
            .to_vec()
            .try_into()
            .or(Err(WalletError::InvalidEd25519PublicKeyBytes))?;

        Ok(WalletCommonUtils::signature(&signature_bytes))
    }
}

/// Perform reflection on a [JsValue]
#[derive(Debug)]
pub struct Reflection(JsValue);

impl Reflection {
    /// Initialize [Reflection] and check if the value is null or undefined
    pub fn new(value: JsValue) -> WalletResult<Self> {
        Reflection::check_is_undefined(&value)?;

        Ok(Self(value))
    }

    /// Initialize [Reflection] from the reflection ([js_sys::Reflect]) of a value
    /// by getting the value described by `key` argument
    pub fn new_from_str(value: &JsValue, key: &str) -> WalletResult<Self> {
        let inner = Reflect::get(value, &key.into())?;

        Reflection::new(inner)
    }

    /// Initialize [Reflection] from a [js_sys::Object]
    pub fn new_object() -> Self {
        Self(Object::new().into())
    }

    /// Consumes [Self](Reflection) and returns a [JsValue]
    pub fn take(self) -> JsValue {
        self.0
    }

    /// Adds the `key` `value` arguments to the object within [Self](Reflection)
    pub fn set_object_str(&mut self, key: &str, value: &str) -> WalletResult<&Self> {
        self.set_object(&key.into(), &value.into())
    }

    /// Adds the `key` `value` arguments to the object within [Self](Reflection)
    pub fn set_object_string_optional(
        &mut self,
        key: &str,
        value: Option<&String>,
    ) -> WalletResult<&Self> {
        if let Some(inner_value) = value {
            self.set_object(&key.into(), &inner_value.into())
        } else {
            Ok(self)
        }
    }

    /// Adds the `key` `value` arguments to the object within [Self](Reflection)
    pub fn set_object(&mut self, key: &JsValue, value: &JsValue) -> WalletResult<&Self> {
        if !self.0.is_object() {
            return Err(WalletError::InternalError(format!(
                "Attempted to set the key `{key:?} in type `{value:?} which is not a JS object"
            )));
        }

        let target = self.0.dyn_ref::<Object>().unwrap(); // check above ensure it is an object hence unwrapping should never fail

        Reflect::set(target, key, value)?;

        self.0 = target.into();

        Ok(self)
    }

    /// Reflect the `key` from the value of [Self](Reflection) and return the
    /// reflected value
    pub fn reflect_inner(&self, key: &str) -> WalletResult<JsValue> {
        let inner = Reflect::get(&self.0, &key.into())?;

        Reflection::check_is_undefined(&inner)?;

        Ok(inner)
    }

    /// Reflect the `key` from the value of [Self](Reflection) and return the
    /// reflected value as a [String]
    pub fn string(&self, key: &str) -> WalletResult<String> {
        let name = Reflect::get(&self.0, &key.into())?;

        let parsed = name.as_string().ok_or(WalletError::InternalError(format!(
            "Reflecting {key:?} did not yield a JsString"
        )))?;

        Ok(parsed)
    }

    /// Reflect the `key` from the value of [Self](Reflection) and return the
    /// reflected value as a [Vec of Vec of bytes](Vec<Vec<u8>>)
    pub fn get_bytes_from_vec(&self, key: &str) -> WalletResult<Vec<Vec<u8>>> {
        let js_array = self.get_array()?;

        js_array
            .iter()
            .map(|value| Reflection::new(value)?.reflect_bytes(key))
            .collect::<WalletResult<Vec<Vec<u8>>>>()
    }

    /// Consume from the value of [Self](Reflection) and return it
    /// as a [Vec of bytes](Vec<u8>)
    pub fn into_bytes(self) -> WalletResult<Vec<u8>> {
        let js_typeof = Self::js_typeof(&self.0);

        Ok(self
            .0
            .dyn_into::<js_sys::Uint8Array>()
            .or(Err(Self::concat_error("Uint8Array", &js_typeof)))?
            .to_vec())
    }

    /// Reflect the `key` from the value of [Self](Reflection) and return the
    /// reflected value as a [Vec of bytes](Vec<Vec<u8>>)
    pub fn reflect_bytes(&self, key: &str) -> WalletResult<Vec<u8>> {
        let js_value = Reflect::get(&self.0, &key.into())?;

        let incase_of_error = Err(WalletError::InternalError(format!(
            "`{js_value:?}` reflected from key `{key}` of JsValue `{:?}` cannot be cast to a Uint8Array, only a JsValue of bytes can be cast.", self.0
        )));

        let to_uint8array = js_value
            .dyn_into::<js_sys::Uint8Array>()
            .or(incase_of_error)?;

        Ok(to_uint8array.to_vec())
    }

    /// Reflect the `key` from the value of [Self](Reflection) and return the
    /// reflected value as a 32 byte array
    pub fn byte32array(&self, key: &str) -> WalletResult<[u8; 32]> {
        let js_value = Reflect::get(&self.0, &key.into())?;

        let to_js_array: js_sys::Uint8Array = js_value.unchecked_into();

        let byte32array: [u8; 32] = to_js_array
            .to_vec()
            .try_into()
            .or(Err(WalletError::Expected32ByteLength))?;

        Ok(byte32array)
    }

    /// Return the value of [Self](Reflection) as a [js_sys::Array]
    /// without consuming `Self`
    pub fn get_array(&self) -> WalletResult<Array> {
        Ok(self.0.clone().dyn_into::<js_sys::Array>()?)
    }

    /// Return a [JsValue] as a [String]
    pub fn get_string(value: &JsValue) -> WalletResult<String> {
        value.as_string().ok_or(WalletError::InternalError(format!(
            "{value:?} is not a JsString"
        )))
    }

    /// Reflect the `key` from the value of [Self](Reflection) and return the
    /// reflected value as a [Vec of String](Vec<String>)
    pub fn vec_string(&self, key: &str) -> WalletResult<Vec<String>> {
        let to_js_array = self.reflect_js_array(key)?;

        to_js_array
            .iter()
            .map(|value| Self::get_string(&value))
            .collect::<WalletResult<Vec<String>>>()
    }

    /// Reflect the `key` from the value of [Self](Reflection) and return the
    /// reflected value as a [Array]
    pub fn reflect_js_array(&self, key: &str) -> WalletResult<Array> {
        let js_value = self.reflect_inner(key)?;

        Self::new(js_value)?.into_array()
    }

    pub(crate) fn vec_string_and_filter(
        &self,
        key: &str,
        filter: &str,
    ) -> WalletResult<Vec<String>> {
        let js_value = self.reflect_inner(key)?;

        let to_js_array = Reflection::new(js_value)?.into_array()?;

        to_js_array
            .iter()
            .map(|value| {
                value.as_string().ok_or(WalletError::InternalError(format!(
                    "{value:?} is not a JsString"
                )))
            })
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

    pub(crate) fn object_to_vec_string(&self, key: &str) -> WalletResult<Vec<String>> {
        let features_value = self.reflect_inner(key)?;

        let js_typeof = Self::js_typeof(&self.0);

        let features_object = features_value
            .dyn_ref::<Object>()
            .ok_or(Self::concat_error("JS Object", &js_typeof))?;

        Object::keys(features_object)
            .iter()
            .map(|value| {
                value.as_string().ok_or(WalletError::InternalError(format!(
                    "{value:?} is not a JsString"
                )))
            })
            .collect::<WalletResult<Vec<String>>>()
    }

    /// Check if [Self](Reflection) is null or undefined
    pub fn check_is_undefined(value: &JsValue) -> WalletResult<()> {
        if value.is_undefined() || value.is_null() {
            Err(WalletError::ValueNotFound)
        } else {
            Ok(())
        }
    }

    /// Reflect a `key` from value of [Self](Reflection) into a [Function]
    pub fn get_function(&self, key: &str) -> WalletResult<Function> {
        let js_value = Reflect::get(&self.0, &key.into())?;

        let incase_of_error = Err(WalletError::InternalError(format!(
            "`{js_value:?}` reflected from key `{key}` of JsValue `{:?}` cannot be cast to a js_sys::Function, only a JsValue of bytes can be cast.", self.0
        )));

        js_value.dyn_into::<Function>().or(incase_of_error)
    }

    /// Get the value of [Self](Reflection) without consuming Self
    pub fn get_inner(&self) -> &JsValue {
        &self.0
    }

    /// Check the `JS typeof` from a [JsValue]
    pub fn js_typeof(value: &JsValue) -> String {
        // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/typeof
        // The `typeof` in Js should always be a string hence unwrapping
        value.js_typeof().as_string().unwrap()
    }

    /// Consume [Self](Reflection) and return it's value as a [Function]
    pub fn into_function(self) -> WalletResult<Function> {
        let js_typeof = Self::js_typeof(&self.0);

        self.0
            .dyn_into::<Function>()
            .or(Err(Self::concat_error("Function", &js_typeof)))
    }

    /// Consume [Self](Reflection) and return it's value as an [Array]
    pub fn into_array(self) -> WalletResult<Array> {
        let js_typeof = Self::js_typeof(&self.0);

        self.0
            .dyn_into::<Array>()
            .or(Err(Self::concat_error("Array", &js_typeof)))
    }

    fn concat_error(expected: &str, encountered: &str) -> WalletError {
        WalletError::InternalError(
            String::new()
                + "Expected a typeof JS "
                + expected
                + "but encountered a typeof Js `"
                + encountered
                + "`.",
        )
    }
}

impl Default for Reflection {
    fn default() -> Self {
        Reflection(JsValue::undefined())
    }
}

impl Clone for Reflection {
    fn clone(&self) -> Self {
        Reflection(self.0.clone())
    }
}
