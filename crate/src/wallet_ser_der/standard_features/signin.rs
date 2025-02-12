use web_sys::{js_sys, wasm_bindgen::JsValue};

use crate::{
    Reflection, SemverVersion, SignInOutput, SigninInput, StandardFunction, WalletAccount,
    WalletError, WalletResult,
};

/// A `solana:signin` struct containing the `version` and `callback`
/// within [StandardFunction]
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SignIn(StandardFunction);

impl SignIn {
    /// Parse the `solana:signin` callback function from the [JsValue]
    pub(crate) fn new(reflection: &Reflection, version: SemverVersion) -> WalletResult<Self> {
        Ok(Self(StandardFunction::new(
            reflection, version, "signIn", "solana",
        )?))
    }

    pub(crate) async fn call_signin(
        &self,
        signin_input: &SigninInput,
        public_key: [u8; 32],
    ) -> WalletResult<SignInOutput> {
        let outcome = self
            .0
            .callback
            .call1(&JsValue::null(), &signin_input.get_object()?)?;

        let outcome = js_sys::Promise::resolve(&outcome);

        let value = wasm_bindgen_futures::JsFuture::from(outcome).await?;
        let output_array = Reflection::new(value)?.get_array()?;

        let first_index = Reflection::new(output_array.get(0))?;
        let account = first_index.reflect_inner("account")?;
        let account = WalletAccount::parse(Reflection::new(account)?)?;

        let message_value = first_index.reflect_inner("signedMessage")?;
        let message_bytes = Reflection::new(message_value)?.as_bytes()?;
        let message =
            core::str::from_utf8(&message_bytes).map_err(|error| WalletError::JsError {
                name: "Invalid UTF-8 Message".to_string(),
                message: error.to_string(),
                stack: "INTERNAL_ERROR".to_string(),
            })?;

        signin_input.check_eq(message)?;

        let signature_value = first_index.reflect_inner("signature")?;
        let signature_bytes: [u8; 64] = Reflection::new(signature_value)?
            .as_bytes()?
            .try_into()
            .or(Err(WalletError::InvalidEd25519SignatureBytes))?;

        SigninInput::verify(public_key, &message_bytes, signature_bytes)?;

        Ok(SignInOutput {
            account,
            message: message.to_string(),
            signature: signature_bytes,
            public_key,
        })
    }
}
