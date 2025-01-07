use ed25519_dalek::{Signature, VerifyingKey};
use wasm_bindgen::JsValue;

use core::str;

use crate::{
    Reflection, SemverVersion, StandardFunction, Utils, WalletAccount, WalletError, WalletResult,
};

/// `solana:signMessage` containing the `version` and `callback` within
/// the [StandardFunction] field
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SignMessage(pub(crate) StandardFunction);

impl SignMessage {
    /// Parse the callback for `solana:signMessage` from the [JsValue]
    pub fn new(value: JsValue, version: SemverVersion) -> WalletResult<Self> {
        Ok(Self(StandardFunction::new(
            value,
            version,
            "signMessage",
            "solana",
            WalletError::MissingSignMessageFunction,
        )?))
    }

    /// Internal callback to request a browser wallet to sign a message
    pub(crate) async fn call_sign_message<'a>(
        &self,
        wallet_account: &WalletAccount,
        message: &'a [u8],
    ) -> WalletResult<SignedMessageOutput<'a>> {
        let message_value: js_sys::Uint8Array = message.into();

        let mut message_object = Reflection::new_object();
        message_object.set_object(&"account".into(), &wallet_account.js_value)?;
        message_object.set_object(&"message".into(), &message_value)?;

        // Call the callback with message and account
        let outcome = self
            .0
            .callback
            .call1(&JsValue::null(), message_object.get_inner())?;

        let outcome = js_sys::Promise::resolve(&outcome);
        let signed_message_result = wasm_bindgen_futures::JsFuture::from(outcome).await?;
        let signed_message_result = Reflection::as_array_owned(signed_message_result)
            .or(Err(WalletError::JsCast(
                "solana:signedMessage -> SignedMessageOutput".to_string(),
            )))?
            .to_vec();

        if let Some(inner) = signed_message_result.first() {
            let reflect_outcome = Reflection::new(inner.clone())?;
            let signed_message = reflect_outcome.reflect_inner("signedMessage")?;
            let signature_value = reflect_outcome.reflect_inner("signature")?;

            let signed_message = Reflection::as_bytes(&signed_message)
                .or(Err(WalletError::JsCast(
                    "solana:signedMessage -> SignedMessageOutput::signedMessage".to_string(),
                )))?
                .to_vec();

            if signed_message != message {
                return Err(WalletError::SignedMessageMismatch);
            }

            let signature = Utils::jsvalue_to_signature(&signature_value)?;

            let public_key = Utils::public_key(wallet_account.public_key)?;

            Utils::verify_signature(public_key, message, signature)?;

            Ok(SignedMessageOutput {
                message,
                public_key: wallet_account.public_key,
                signature: signature.to_bytes(),
            })
        } else {
            Err(WalletError::ReceivedAnEmptySignedMessagesArray)
        }
    }
}

/// The output of a signed message
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Clone)]
pub struct SignedMessageOutput<'a> {
    message: &'a [u8],
    public_key: [u8; 32],
    signature: [u8; 64],
}

impl SignedMessageOutput<'_> {
    /// Get the message as a UTF-8 [str]
    pub fn message(&self) -> &str {
        //Should never fail since verified message is always UTF-8 Format hence `.unwrap()` is used.
        // This is verified to be the input message where the input message is always UTF-8 encoded
        str::from_utf8(self.message).unwrap()
    }

    /// Get the public key as an [Ed25519 Public Key](VerifyingKey)
    pub fn public_key(&self) -> WalletResult<VerifyingKey> {
        Utils::public_key(self.public_key)
    }

    /// Get the Base58 address of the  [Ed25519 Public Key](VerifyingKey) that signed the message
    pub fn address(&self) -> WalletResult<String> {
        Ok(Utils::address(self.public_key()?))
    }

    /// Get the [Ed25519 Signature](Signature) that was generated when
    /// the [Ed25519 Public Key](VerifyingKey) signed the UTF-8 encoded message
    pub fn signature(&self) -> Signature {
        Utils::signature(self.signature)
    }

    /// Get the  [Ed25519 Signature](Signature) encoded in Base58 format
    pub fn base58_signature(&self) -> WalletResult<String> {
        Ok(Utils::base58_signature(self.signature()))
    }
}

impl Default for SignedMessageOutput<'_> {
    fn default() -> Self {
        Self {
            message: &[],
            public_key: [0u8; 32],
            signature: [0u8; 64],
        }
    }
}
