use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::{Reflection, WalletError, WalletResult};

use wallet_adapter_common::{clusters::Cluster, signin_standard::SigninInput as SigninInputLib};
use web_sys::{
    js_sys::{self, Array},
    wasm_bindgen::JsValue,
    Window,
};

/// The Sign In input used as parameters when performing
/// `SignInWithSolana (SIWS)` requests as defined by the
/// [SIWS](https://github.com/phantom/sign-in-with-solana) standard.
/// A backup fork can be found at [https://github.com/JamiiDao/sign-in-with-solana](https://github.com/JamiiDao/sign-in-with-solana)
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SigninInput(pub(crate) SigninInputLib);

impl SigninInput {
    /// Same as `Self::default()` as it initializes [Self] with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// An EIP-4361 domain requesting the sign-in.
    /// If not provided, the wallet must determine the domain to include in the message.
    /// Sets the domain name by fetching the details from [window.location().host()](web_sys::Location) .
    pub fn set_domain(&mut self, window: &Window) -> WalletResult<&mut Self> {
        let host = window.location().host()?;

        self.0.set_domain(&host);

        Ok(self)
    }

    /// An EIP-4361 domain requesting the sign-in.
    /// If not provided, the wallet must determine the domain to include in the message.
    /// Sets a custom domain name instead of fetching from
    /// [window.location().host()](web_sys::Location)
    pub fn set_custom_domain(&mut self, domain: &str) -> &mut Self {
        self.0.set_domain(domain);

        self
    }

    /// The Base58 public key address
    /// NOTE: Some wallets require this field or
    /// an error `MessageResponseMismatch` which is as
    /// a result of the sent message not corresponding with the signed message
    pub fn set_address(&mut self, address: &str) -> WalletResult<&mut Self> {
        self.0.set_address(address)?;

        Ok(self)
    }

    ///  An EIP-4361 Statement which is a human readable string and should not have new-line characters (\n).
    /// Sets the message that is shown to the user during Sign In With Solana
    pub fn set_statement(&mut self, statement: &str) -> &mut Self {
        self.0.set_statement(statement);

        self
    }

    /// An EIP-4361 URI is automatically set to the `window.location.href`
    /// since if it is not the same, the wallet will ignore it and
    /// show the user an error.
    /// This is the URL that is requesting the sign-in.
    pub fn set_uri(&mut self, window: &Window) -> WalletResult<&mut Self> {
        self.0.set_uri(&window.location().href()?);

        Ok(self)
    }

    /// An EIP-4361 version.
    /// Sets the version
    pub fn set_version(&mut self, version: &str) -> &mut Self {
        self.0.set_version(version);

        self
    }

    /// An EIP-4361 Chain ID.
    /// The chainId can be one of the following:
    /// mainnet, testnet, devnet, localnet, solana:mainnet, solana:testnet, solana:devnet.
    pub fn set_chain_id(&mut self, cluster: Cluster) -> &mut Self {
        self.0.set_chain_id(cluster);

        self
    }

    /// An EIP-4361 Nonce which is an alphanumeric string containing a minimum of 8 characters.
    /// This is generated from the Cryptographically Secure Random Number Generator
    /// and the bytes converted to hex formatted string.
    pub fn set_nonce(&mut self) -> &mut Self {
        self.0.set_nonce();
        self
    }

    ///  An EIP-4361 Nonce which is an alphanumeric string containing a minimum of 8 characters.
    /// This is generated from the Cryptographically Secure Random Number Generator
    /// and the bytes converted to hex formatted string.
    pub fn custom_nonce(&mut self, nonce: &str) -> WalletResult<&mut Self> {
        self.0.set_custom_nonce(nonce)?;

        Ok(self)
    }

    /// Fetches the time from [JavaScript Date Now](js_sys::Date::now()) .
    /// This is converted to [SystemTime]
    pub fn time_now() -> WalletResult<SystemTime> {
        let date_now = js_sys::Date::now() as u64;

        UNIX_EPOCH
            .checked_add(Duration::from_millis(date_now))
            .ok_or(WalletError::JsError {
                name: "UNIX_EPOCH.checked_add(js_sys::Date::now()".to_string(),
                message: "Unable to get the current time".to_string(),
                stack: "INTERNAL ERROR".to_string(),
            })
    }

    ///  This represents the time at which the sign-in request was issued to the wallet.
    /// Note: For Phantom, issuedAt has a threshold and it should be within +- 10 minutes
    /// from the timestamp at which verification is taking place.
    /// If not provided, the wallet does not include Issued At in the message.
    /// This also follows the ISO 8601 datetime.
    pub fn set_issued_at(&mut self) -> WalletResult<&mut Self> {
        self.0.set_issued_at(Self::time_now()?);

        Ok(self)
    }

    /// An ergonomic method for [Self::set_expiration_time()]
    /// where you can add milliseconds and [SystemTime] is automatically calculated for you
    pub fn set_expiration_time_millis(
        &mut self,
        expiration_time_milliseconds: u64,
    ) -> WalletResult<&mut Self> {
        self.0
            .set_expiration_time_millis(Self::time_now()?, expiration_time_milliseconds)?;

        Ok(self)
    }

    /// An ergonomic method for [Self::set_expiration_time()]
    /// where you can add seconds and [SystemTime] is automatically calculated for you
    pub fn set_expiration_time_seconds(
        &mut self,
        expiration_time_seconds: u64,
    ) -> WalletResult<&mut Self> {
        self.0
            .set_expiration_time_seconds(Self::time_now()?, expiration_time_seconds)?;

        Ok(self)
    }

    /// An ISO 8601 datetime string. This represents the time at which the sign-in request should expire.
    /// If not provided, the wallet does not include Expiration Time in the message.
    /// Expiration time should be in future or an error will be thrown even before a request to the wallet is sent
    pub fn set_expiration_time(&mut self, expiration_time: SystemTime) -> WalletResult<&mut Self> {
        if let Some(issued_at) = self.0.issued_at() {
            if issued_at > &expiration_time {
                return Err(WalletError::ExpiryTimeEarlierThanIssuedTime);
            }
        }

        let now = Self::time_now()?;

        if now > expiration_time {
            return Err(WalletError::ExpirationTimeIsInThePast);
        }

        self.0.set_expiration_time(now, expiration_time)?;

        Ok(self)
    }

    /// An ergonomic method for [Self::set_not_before_time()]
    /// where you can add milliseconds and [SystemTime] is automatically calculated for you
    pub fn set_not_before_time_millis(
        &mut self,
        expiration_time_milliseconds: u64,
    ) -> WalletResult<&mut Self> {
        self.0
            .set_not_before_time_millis(Self::time_now()?, expiration_time_milliseconds)?;

        Ok(self)
    }

    /// An ergonomic method for [Self::set_not_before_time()]
    /// where you can add seconds and [SystemTime] is automatically calculated for you
    pub fn set_not_before_time_seconds(
        &mut self,
        expiration_time_seconds: u64,
    ) -> WalletResult<&mut Self> {
        self.0
            .set_not_before_time_seconds(Self::time_now()?, expiration_time_seconds)?;

        Ok(self)
    }

    /// An ISO 8601 datetime string.
    /// This represents the time at which the sign-in request becomes valid.
    /// If not provided, the wallet does not include Not Before in the message.
    /// Time must be after `IssuedTime`
    pub fn set_not_before_time(&mut self, not_before: SystemTime) -> WalletResult<&mut Self> {
        self.0.set_not_before_time(Self::time_now()?, not_before)?;

        Ok(self)
    }

    /// Converts [Self] to a [JsValue] to pass to the wallet where it's internal representation
    /// is a [js_sys::Object]
    pub fn get_object(&self) -> WalletResult<JsValue> {
        let mut signin_input_object = Reflection::new_object();

        signin_input_object.set_object_string_optional("domain", self.0.domain())?;
        signin_input_object.set_object_string_optional("address", self.0.address())?;
        signin_input_object.set_object_string_optional("statement", self.0.statement())?;
        signin_input_object.set_object_string_optional("uri", self.0.uri())?;
        signin_input_object.set_object_string_optional("version", self.0.version())?;
        signin_input_object.set_object_string_optional("address", self.0.address())?;
        signin_input_object.set_object_string_optional(
            "chainId",
            self.0
                .chain_id()
                .map(|cluster| cluster.chain().to_string())
                .as_ref(),
        )?;
        signin_input_object.set_object_string_optional("nonce", self.0.nonce())?;
        signin_input_object
            .set_object_string_optional("issuedAt", self.issued_at_iso8601().as_ref())?;
        signin_input_object.set_object_string_optional(
            "expirationTime",
            self.expiration_time_iso8601().as_ref(),
        )?;
        signin_input_object
            .set_object_string_optional("notBefore", self.not_before_iso8601().as_ref())?;
        signin_input_object.set_object_string_optional("requestId", self.0.request_id())?;

        if !self.0.resources().is_empty() {
            let stringify_resources = Array::new();
            self.0.resources().iter().for_each(|resource| {
                stringify_resources.push(&resource.into());
            });
            signin_input_object.set_object(&"resources".into(), &stringify_resources)?;
        }

        Ok(signin_input_object.take())
    }

    /// An EIP-4361 Request ID.
    /// In addition to using nonce to avoid replay attacks,
    /// dapps can also choose to include a unique signature in the requestId .
    /// Once the wallet returns the signed message,
    /// dapps can then verify this signature against the state to add an additional,
    /// strong layer of security. If not provided, the wallet must not include Request ID in the message.
    pub fn set_request_id(&mut self, id: &str) -> &mut Self {
        self.0.set_request_id(id);

        self
    }

    /// An EIP-4361 Resources.
    /// Usually a list of references in the form of URIs that the dapp wants the user to be aware of.
    /// These URIs should be separated by \n-, ie, URIs in new lines starting with the character -.
    /// If not provided, the wallet must not include Resources in the message.
    pub fn add_resource(&mut self, resource: &str) -> &mut Self {
        self.0.add_resource(resource);

        self
    }

    /// Helper for [Self::add_resource()] when you want to add multiple resources at the same time
    pub fn add_resources(&mut self, resources: &[&str]) -> &mut Self {
        self.0.add_resources(resources);

        self
    }

    /// Get the `domain` field
    pub fn domain(&self) -> Option<&String> {
        self.0.domain()
    }

    /// Get the `address` field
    pub fn address(&self) -> Option<&String> {
        self.0.address()
    }

    /// Get the `statement` field
    pub fn statement(&self) -> Option<&String> {
        self.0.statement()
    }

    /// Get the `uri` field
    pub fn uri(&self) -> Option<&String> {
        self.0.uri()
    }

    /// Get the `version` field
    pub fn version(&self) -> Option<&String> {
        self.0.version()
    }

    /// Get the `chain_id` field
    pub fn chain_id(&self) -> Option<&Cluster> {
        self.0.chain_id()
    }

    /// Get the `nonce` field
    pub fn nonce(&self) -> Option<&String> {
        self.0.nonce()
    }

    /// Get the `issued_at` field
    pub fn issued_at(&self) -> Option<&SystemTime> {
        self.0.issued_at()
    }

    /// Get the `expiration_time` field
    pub fn expiration_time(&self) -> Option<&SystemTime> {
        self.0.expiration_time()
    }

    /// Get the `not_before` field
    pub fn not_before(&self) -> Option<&SystemTime> {
        self.0.not_before()
    }

    /// Get the `issued_at` field as ISO8601 date time string
    pub fn issued_at_iso8601(&self) -> Option<String> {
        self.0.issued_at_iso8601()
    }

    /// Get the `expiration_time` field as ISO8601 date time string
    pub fn expiration_time_iso8601(&self) -> Option<String> {
        self.0.expiration_time_iso8601()
    }

    /// Get the `not_before` field as ISO8601 date time string
    pub fn not_before_iso8601(&self) -> Option<String> {
        self.0.not_before_iso8601()
    }

    /// Get the `request_id` field
    pub fn request_id(&self) -> Option<&String> {
        self.0.request_id()
    }

    /// Get the `resources` field
    pub fn resources(&self) -> &[String] {
        self.0.resources()
    }
}

#[cfg(test)]
#[cfg(target_arch = "wasm32")]
mod signin_input_sanity_checks {
    use super::*;

    #[test]
    fn set_issued_at() {
        let mut signin_input = SigninInput::default();

        assert!(signin_input.issued_at().is_none());

        signin_input.set_issued_at().unwrap();

        assert!(signin_input.issued_at.unwrap() > SystemTime::UNIX_EPOCH)
    }

    #[test]
    fn set_expiration_time() {
        let mut signin_input = SigninInput::default();

        let now = SigninInput::time_now().unwrap();

        let past_time = now.checked_sub(Duration::from_secs(300)).unwrap();
        assert_eq!(
            Some(WalletError::ExpirationTimeIsInThePast),
            signin_input.set_expiration_time(past_time).err()
        );

        signin_input.set_issued_at().unwrap();
        assert_eq!(
            Some(WalletError::ExpiryTimeEarlierThanIssuedTime),
            signin_input.set_expiration_time(past_time).err()
        );

        let valid_expiry = now.checked_add(Duration::from_secs(300)).unwrap();
        assert!(signin_input.set_expiration_time(valid_expiry).is_ok());

        assert!(signin_input.issued_at.unwrap() > SystemTime::UNIX_EPOCH);

        assert!(signin_input.set_expiration_time_millis(4000).is_ok());
        assert!(signin_input.set_expiration_time_seconds(4).is_ok());
    }

    #[test]
    fn set_not_before_time() {
        let mut signin_input = SigninInput::default();

        let now = SigninInput::time_now().unwrap();

        let past_time = now.checked_sub(Duration::from_secs(300)).unwrap();
        assert_eq!(
            Some(WalletError::NotBeforeTimeIsInThePast),
            signin_input.set_not_before_time(past_time).err()
        );

        signin_input.set_issued_at().unwrap();
        let future_time = now.checked_sub(Duration::from_secs(3000000)).unwrap();
        assert_eq!(
            Some(WalletError::NotBeforeTimeEarlierThanIssuedTime),
            signin_input.set_not_before_time(future_time).err()
        );

        signin_input.set_issued_at().unwrap();
        let future_time = SigninInput::time_now()
            .unwrap()
            .checked_add(Duration::from_secs(30000))
            .unwrap();
        signin_input.set_expiration_time(future_time).unwrap();
        let future_time = now.checked_add(Duration::from_secs(3000000)).unwrap();
        assert_eq!(
            Some(WalletError::NotBeforeTimeLaterThanExpirationTime),
            signin_input.set_not_before_time(future_time).err()
        );

        let valid_expiry = now.checked_add(Duration::from_secs(300)).unwrap();
        assert!(signin_input.set_not_before_time(valid_expiry).is_ok());

        assert!(signin_input.issued_at.unwrap() > SystemTime::UNIX_EPOCH);

        assert!(signin_input.set_not_before_time_millis(4000).is_ok());
        assert!(signin_input.set_not_before_time_seconds(4).is_ok());
    }
}
