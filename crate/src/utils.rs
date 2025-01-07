use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use js_sys::{Array, Function, Object, Reflect, Uint8Array};
use wasm_bindgen::{JsCast, JsValue};

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
        bytes.try_into().or(Err(WalletError::SliceTo32ByteArray))
    }

    /// Convert a slice of bytes into a 64 byte array. This is useful especially if a [Signature] is
    /// given as a slice instead of 64 byte array
    pub fn to64byte_array(bytes: &[u8]) -> WalletResult<[u8; 64]> {
        bytes.try_into().or(Err(WalletError::SliceTo64ByteArray))
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
    pub fn jsvalue_to_signature(value: &JsValue) -> WalletResult<Signature> {
        let signature_bytes: [u8; 64] = Reflection::as_bytes(value)?
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
}

#[derive(Debug)]
pub(crate) struct Reflection(JsValue);

impl<'lulu> Reflection {
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
        let target = Self::as_object(&self.0)?;

        Reflect::set(target, key, value)?;

        self.0 = target.into();

        Ok(self)
    }

    pub(crate) fn reflect_inner(&self, key: &str) -> WalletResult<JsValue> {
        let inner = Reflect::get(&self.0, &key.into())?;

        Reflection::check_is_undefined(&inner)?;

        Ok(inner)
    }

    pub(crate) fn reflect_string(&self, key: &str) -> WalletResult<String> {
        let name = Reflect::get(&self.0, &key.into())?;

        Self::as_string(&name)
    }

    pub(crate) fn reflect_string_raw(js_value: &JsValue, key: &str) -> WalletResult<String> {
        let name = Reflect::get(js_value, &key.into())?;

        Self::as_string(&name)
    }

    pub(crate) fn as_vec_of_bytes(&self, key: &str) -> WalletResult<Vec<Vec<u8>>> {
        let js_array = Self::as_array(&self.0)?;

        js_array
            .iter()
            .map(|value| Reflection::new(value)?.reflect_bytes(key))
            .collect::<WalletResult<Vec<Vec<u8>>>>()
    }

    pub(crate) fn reflect_bytes(&self, key: &str) -> WalletResult<Vec<u8>> {
        let js_value = Reflect::get(&self.0, &key.into())?;

        Self::as_bytes(&js_value)
    }

    pub(crate) fn reflect_byte32array(&self, key: &str) -> WalletResult<[u8; 32]> {
        let js_value = Reflect::get(&self.0, &key.into())?;

        Self::as_bytes(&js_value).map(|value| {
            let byte32array: [u8; 32] =
                value.try_into().or(Err(WalletError::SliceTo32ByteArray))?;

            Ok(byte32array)
        })?
    }

    pub(crate) fn vec_string(&self, key: &str) -> WalletResult<Vec<String>> {
        let to_js_array = self.reflect_js_array(key)?;

        to_js_array
            .iter()
            .map(|value| Self::as_string(&value))
            .collect::<WalletResult<Vec<String>>>()
    }

    pub(crate) fn reflect_js_array(&self, key: &str) -> WalletResult<Array> {
        let js_value = Reflect::get(&self.0, &key.into())?;

        Self::as_array_owned(js_value)
    }

    pub(crate) fn vec_string_and_filter(
        &self,
        key: &str,
        filter: &str,
    ) -> WalletResult<Vec<String>> {
        let js_value = self.reflect_js_array(key)?;

        let to_js_array = Self::as_array(&js_value)?;

        to_js_array
            .iter()
            .map(|value| Self::as_string(&value))
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
        let features_value = Reflect::get(&self.0, &key.into())?;

        let features_object = Self::as_object(&features_value)?;

        Object::keys(features_object)
            .iter()
            .map(|value| Self::as_string(&value))
            .collect::<WalletResult<Vec<String>>>()
    }

    pub(crate) fn check_is_undefined(value: &JsValue) -> WalletResult<()> {
        if value.is_undefined() || value.is_null() {
            Err(WalletError::JsValueNotFound)
        } else {
            Ok(())
        }
    }

    pub(crate) fn get_function(&self, key: &str) -> WalletResult<Function> {
        let js_value = Reflect::get(&self.0, &key.into())?;

        Self::as_function_owned(js_value)
    }

    pub(crate) fn get_inner(&self) -> &JsValue {
        &self.0
    }

    pub(crate) fn js_typeof(value: &JsValue) -> String {
        // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/typeof
        // The `typeof` in Js should always be a string hence unwrapping
        value.js_typeof().as_string().unwrap()
    }

    pub(crate) fn as_string(value: &JsValue) -> WalletResult<String> {
        value
            .as_string()
            .ok_or(Self::concat_error("String", &Self::js_typeof(value)))
    }

    pub(crate) fn as_function_owned(value: JsValue) -> WalletResult<Function> {
        let js_typeof = Self::js_typeof(&value);

        value
            .dyn_into::<Function>()
            .or(Err(Self::concat_error("Function", &js_typeof)))
    }

    pub(crate) fn as_object(value: &'lulu JsValue) -> WalletResult<&'lulu Object> {
        let js_typeof = Self::js_typeof(&value);

        value
            .dyn_ref::<Object>()
            .ok_or(Self::concat_error("Object", &js_typeof))
    }

    pub(crate) fn as_array(value: &'lulu JsValue) -> WalletResult<&'lulu Array> {
        let js_typeof = Self::js_typeof(&value);

        value
            .dyn_ref::<Array>()
            .ok_or(Self::concat_error("Array", &js_typeof))
    }

    pub(crate) fn as_array_owned(value: JsValue) -> WalletResult<Array> {
        let js_typeof = Self::js_typeof(&value);

        value
            .dyn_into::<Array>()
            .or(Err(Self::concat_error("Array", &js_typeof)))
    }

    pub(crate) fn as_bytes(value: &JsValue) -> WalletResult<Vec<u8>> {
        let js_typeof = Self::js_typeof(&value);

        value
            .dyn_ref::<Uint8Array>()
            .ok_or(Self::concat_error("Uint8Array", &js_typeof))
            .map(|value| value.to_vec())
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

    fn concat_error(expected: &str, encountered: &str) -> WalletError {
        WalletError::JsCast(
            String::new()
                + "Expected a typeof JS "
                + expected
                + "but encountered a typeof Js `"
                + encountered
                + "`.",
        )
    }
}
