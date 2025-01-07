use wasm_bindgen::{JsCast, JsValue};

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
    pub fn new(value: JsValue, version: SemverVersion) -> Option<Self> {
        StandardFunction::new(
            value,
            version,
            "signIn",
            "solana",
            WalletError::MissingSignInFunction(
                "This wallet does not support SignIn with Solana.".to_string(),
            ),
        )
        .map(|value| Self(value))
        .ok()
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
        let output_array = Reflection::as_array(&value)?;

        let first_index = Reflection::new(output_array.get(0))?;
        let account = first_index.reflect_inner("account")?;
        let account = WalletAccount::parse(Reflection::new(account)?)?;

        let message_value = first_index.reflect_inner("signedMessage")?;
        let message_bytes = message_value.dyn_into::<js_sys::Uint8Array>()?.to_vec();
        let message =
            core::str::from_utf8(&message_bytes).map_err(|error| WalletError::JsError {
                name: "Invalid UTF-8 Message".to_string(),
                message: error.to_string(),
                stack: "INTERNAL_ERROR".to_string(),
            })?;

        signin_input.check_eq(message)?;

        let signature_value = first_index.reflect_inner("signature")?;
        let signature_bytes: [u8; 64] = Reflection::as_bytes(&signature_value)?
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
