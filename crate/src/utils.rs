use std::borrow::Cow;

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use web_sys::{
    js_sys::{self, Array, Function, Object, Reflect},
    wasm_bindgen::{JsCast, JsValue},
};

use crate::{WalletError, WalletResult};

/// A 32 byte array representing a Public Key
pub type PublicKeyBytes = [u8; 32];

/// A 64 byte array representing a Signature
pub type SignatureBytes = [u8; 64];

/// The Version of the Wallet Standard currently implemented.
/// This may be used by the app to determine compatibility and feature detect.
pub const WALLET_STANDARD_VERSION: &str = "1.0.0";

/// Helper utilities
pub struct Utils;

impl Utils {
    /// Generate a public key from random bytes. This is useful for testing
    pub fn public_key_rand() -> [u8; 32] {
        Self::rand_32bytes()
    }

    /// Generate a 32 byte array from random bytes
    pub fn rand_32bytes() -> [u8; 32] {
        use rand_chacha::ChaCha20Rng;
        use rand_core::{RngCore, SeedableRng};

        let mut rng = ChaCha20Rng::from_entropy();

        let mut buffer = [0u8; 32];

        rng.fill_bytes(&mut buffer);

        buffer
    }

    pub(crate) fn jsvalue_to_error<T: core::fmt::Debug>(
        value: Result<T, JsValue>,
    ) -> Result<(), WalletError> {
        if let Err(error) = value {
            Err(error.into())
        } else {
            Ok(())
        }
    }

    /// Parse a [PublicKey] from an array of 32 bytes
    pub fn public_key(public_key_bytes: [u8; 32]) -> WalletResult<VerifyingKey> {
        VerifyingKey::from_bytes(&public_key_bytes)
            .or(Err(WalletError::InvalidEd25519PublicKeyBytes))
    }

    /// Parse a [Signature] from an array of 64 bytes
    pub fn signature(signature_bytes: [u8; 64]) -> Signature {
        Signature::from_bytes(&signature_bytes)
    }

    /// Convert a slice of bytes into a 32 byte array. This is useful especially if a [PublicKey] is
    /// given as a slice instead of 32 byte array
    pub fn to32byte_array(bytes: &[u8]) -> WalletResult<[u8; 32]> {
        bytes.try_into().or(Err(WalletError::Expected32ByteLength))
    }

    /// Convert a slice of bytes into a 64 byte array. This is useful especially if a [Signature] is
    /// given as a slice instead of 64 byte array
    pub fn to64byte_array(bytes: &[u8]) -> WalletResult<[u8; 64]> {
        bytes.try_into().or(Err(WalletError::Expected64ByteLength))
    }

    /// Verify a [message](str) using a [PublicKey] and [Signature]
    pub fn verify_signature(
        public_key: VerifyingKey,
        message: &[u8],
        signature: Signature,
    ) -> WalletResult<()> {
        public_key
            .verify(message, &signature)
            .or(Err(WalletError::InvalidSignature))
    }

    /// Convert a [JsValue] to a [Signature]
    pub fn jsvalue_to_signature(value: JsValue, namespace: &str) -> WalletResult<Signature> {
        let in_case_of_error = Err(WalletError::InternalError(format!(
            "{namespace}: `{value:?}` cannot be cast to a Uint8Array, only a JsValue of bytes can be cast."
        )));

        let signature_bytes: [u8; 64] = value
            .dyn_into::<js_sys::Uint8Array>()
            .or(in_case_of_error)?
            .to_vec()
            .try_into()
            .or(Err(WalletError::InvalidEd25519PublicKeyBytes))?;

        Ok(Self::signature(signature_bytes))
    }

    /// Generate the Base58 address from a [PublicKey]
    pub fn address(public_key: VerifyingKey) -> String {
        bs58::encode(public_key.as_ref()).into_string()
    }

    /// Generate a Base58 encoded string from a [Signature]
    pub fn base58_signature(signature: Signature) -> String {
        bs58::encode(signature.to_bytes()).into_string()
    }

    /// Get the shortened string of the `Base58 string` .
    /// It displays the first 4 characters and the last for characters
    /// separated by ellipsis eg `FXdl...RGd4` .
    /// If the string is less than 8 characters, an error is thrown
    pub fn shorten_base58(base58_str: &str) -> WalletResult<Cow<str>> {
        if base58_str.len() < 8 {
            return Err(WalletError::InvalidBase58Address);
        }

        let first_part = &base58_str[..4];
        let last_part = &base58_str[base58_str.len() - 4..];

        Ok(Cow::Borrowed(first_part) + "..." + last_part)
    }

    /// Same as [Self::shorten_base58] but with a custom range
    /// instead of taking the first 4 character and the last 4 characters
    /// it uses a custom range.
    pub fn custom_shorten_base58(base58_str: &str, take: usize) -> WalletResult<Cow<str>> {
        if base58_str.len() < take + take {
            return Err(WalletError::InvalidBase58Address);
        }

        let first_part = &base58_str[..take];
        let last_part = &base58_str[base58_str.len() - take..];

        Ok(Cow::Borrowed(first_part) + "..." + last_part)
    }
}

#[derive(Debug)]
pub(crate) struct Reflection(JsValue);

impl<'wa> Reflection {
    pub(crate) fn new(value: JsValue) -> WalletResult<Self> {
        Reflection::check_is_undefined(&value)?;

        Ok(Self(value))
    }

    pub(crate) fn new_from_str(value: &JsValue, key: &str) -> WalletResult<Self> {
        let inner = Reflect::get(value, &key.into())?;

        Reflection::new(inner)
    }

    pub(crate) fn new_object() -> Self {
        Self(Object::new().into())
    }

    pub(crate) fn take(self) -> JsValue {
        self.0
    }

    pub(crate) fn set_object_str(&mut self, key: &str, value: &str) -> WalletResult<&Self> {
        self.set_object(&key.into(), &value.into())
    }

    pub(crate) fn set_object_string_optional(
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

    pub(crate) fn set_object(&mut self, key: &JsValue, value: &JsValue) -> WalletResult<&Self> {
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

    pub(crate) fn reflect_inner(&self, key: &str) -> WalletResult<JsValue> {
        let inner = Reflect::get(&self.0, &key.into())?;

        Reflection::check_is_undefined(&inner)?;

        Ok(inner)
    }

    pub(crate) fn string(&self, key: &str) -> WalletResult<String> {
        let name = Reflect::get(&self.0, &key.into())?;

        let parsed = name.as_string().ok_or(WalletError::InternalError(format!(
            "Reflecting {key:?} did not yield a JsString"
        )))?;

        Ok(parsed)
    }

    pub(crate) fn get_bytes_from_vec(&self, key: &str) -> WalletResult<Vec<Vec<u8>>> {
        let js_array = self.get_array()?;

        js_array
            .iter()
            .map(|value| Reflection::new(value)?.reflect_bytes(key))
            .collect::<WalletResult<Vec<Vec<u8>>>>()
    }

    pub(crate) fn as_bytes(self) -> WalletResult<Vec<u8>> {
        let js_typeof = Self::js_typeof(&self.0);

        Ok(self
            .0
            .dyn_into::<js_sys::Uint8Array>()
            .or(Err(Self::concat_error("Uint8Array", &js_typeof)))?
            .to_vec())
    }

    pub(crate) fn reflect_bytes(&self, key: &str) -> WalletResult<Vec<u8>> {
        let js_value = Reflect::get(&self.0, &key.into())?;

        let incase_of_error = Err(WalletError::InternalError(format!(
            "`{js_value:?}` reflected from key `{key}` of JsValue `{:?}` cannot be cast to a Uint8Array, only a JsValue of bytes can be cast.", self.0
        )));

        let to_uint8array = js_value
            .dyn_into::<js_sys::Uint8Array>()
            .or(incase_of_error)?;

        Ok(to_uint8array.to_vec())
    }

    pub(crate) fn byte32array(&self, key: &str) -> WalletResult<[u8; 32]> {
        let js_value = Reflect::get(&self.0, &key.into())?;

        let to_js_array: js_sys::Uint8Array = js_value.unchecked_into();

        let byte32array: [u8; 32] = to_js_array
            .to_vec()
            .try_into()
            .or(Err(WalletError::Expected32ByteLength))?;

        Ok(byte32array)
    }

    pub(crate) fn get_array(&self) -> WalletResult<Array> {
        Ok(self.0.clone().dyn_into::<js_sys::Array>()?)
    }

    pub(crate) fn get_string(value: &JsValue) -> WalletResult<String> {
        value.as_string().ok_or(WalletError::InternalError(format!(
            "{value:?} is not a JsString"
        )))
    }

    pub(crate) fn vec_string(&self, key: &str) -> WalletResult<Vec<String>> {
        let to_js_array = self.reflect_js_array(key)?;

        to_js_array
            .iter()
            .map(|value| Self::get_string(&value))
            .collect::<WalletResult<Vec<String>>>()
    }

    pub(crate) fn reflect_js_array(&self, key: &str) -> WalletResult<Array> {
        let js_value = self.reflect_inner(key)?;

        Self::new(js_value)?.as_array()
    }

    pub(crate) fn vec_string_and_filter(
        &self,
        key: &str,
        filter: &str,
    ) -> WalletResult<Vec<String>> {
        let js_value = self.reflect_inner(key)?;

        let to_js_array = Reflection::new(js_value)?.as_array()?;

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

    pub(crate) fn check_is_undefined(value: &JsValue) -> WalletResult<()> {
        if value.is_undefined() || value.is_null() {
            Err(WalletError::ValueNotFound)
        } else {
            Ok(())
        }
    }

    pub(crate) fn get_function(&self, key: &str) -> WalletResult<Function> {
        let js_value = Reflect::get(&self.0, &key.into())?;

        let incase_of_error = Err(WalletError::InternalError(format!(
            "`{js_value:?}` reflected from key `{key}` of JsValue `{:?}` cannot be cast to a js_sys::Function, only a JsValue of bytes can be cast.", self.0
        )));

        js_value.dyn_into::<Function>().or(incase_of_error)
    }

    pub(crate) fn get_inner(&self) -> &JsValue {
        &self.0
    }

    pub(crate) fn js_typeof(value: &JsValue) -> String {
        // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/typeof
        // The `typeof` in Js should always be a string hence unwrapping
        value.js_typeof().as_string().unwrap()
    }

    pub(crate) fn as_function_owned(self) -> WalletResult<Function> {
        let js_typeof = Self::js_typeof(&self.0);

        self.0
            .dyn_into::<Function>()
            .or(Err(Self::concat_error("Function", &js_typeof)))
    }

    pub(crate) fn as_array(self) -> WalletResult<Array> {
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
