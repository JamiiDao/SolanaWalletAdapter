use std::time::{Duration, SystemTime, UNIX_EPOCH};

use js_sys::Array;
use wasm_bindgen::JsValue;
use web_sys::Window;

use crate::{Cluster, Reflection, Utils, WalletError, WalletResult};

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SigninInput {
    /// Optional EIP-4361 domain requesting the sign-in.
    /// If not provided, the wallet must determine the domain to include in the message.
    domain: Option<String>,
    /// Optional Solana Base58 address performing the sign-in.
    /// The address is case-sensitive.
    /// If not provided, the wallet must determine the Address to include in the message.
    address: Option<String>,
    /// Optional EIP-4361 Statement.
    /// The statement is a human readable string and should not have new-line characters (\n).
    /// If not provided, the wallet must not include Statement in the message.
    statement: Option<String>,
    /// Optional EIP-4361 URI.
    /// The URL that is requesting the sign-in.
    /// If not provided, the wallet must not include URI in the message.
    uri: Option<String>,
    /// Optional EIP-4361 version.
    /// If not provided, the wallet must not include Version in the message.
    version: Option<String>,
    /// Optional EIP-4361 Chain ID.
    /// The chainId can be one of the following:
    /// mainnet, testnet, devnet, localnet, solana:mainnet, solana:testnet, solana:devnet.
    /// If not provided, the wallet must not include Chain ID in the message.
    chain_id: Option<Cluster>,
    /// Optional EIP-4361 Nonce.
    /// It should be an alphanumeric string containing a minimum of 8 characters.
    /// If not provided, the wallet must not include Nonce in the message.
    nonce: Option<String>,
    /// Optional ISO 8601 datetime string.
    /// This represents the time at which the sign-in request was issued to the wallet.
    /// Note: For Phantom, issuedAt has a threshold and it should be
    /// within +- 10 minutes from the timestamp at which verification is taking place.
    /// If not provided, the wallet must not include Issued At in the message.
    issued_at: Option<SystemTime>,
    /// Optional ISO 8601 datetime string.
    /// This represents the time at which the sign-in request should expire.
    /// If not provided, the wallet must not include Expiration Time in the message.
    expiration_time: Option<SystemTime>,
    /// Optional ISO 8601 datetime string.
    /// This represents the time at which the sign-in request becomes valid.
    /// If not provided, the wallet must not include Not Before in the message.
    not_before: Option<SystemTime>,
    /// Optional EIP-4361 Request ID.
    /// In addition to using nonce to avoid replay attacks,
    /// dapps can also choose to include a unique signature in the requestId .
    /// Once the wallet returns the signed message,
    /// dapps can then verify this signature against the state to add an additional,
    /// strong layer of security. If not provided, the wallet must not include Request ID in the message.
    request_id: Option<String>,
    /// Optional EIP-4361 Resources.
    /// Usually a list of references in the form of URIs that the
    /// dapp wants the user to be aware of.
    /// These URIs should be separated by \n-, ie,
    /// URIs in new lines starting with the character -.
    /// If not provided, the wallet must not include Resources in the message.
    resources: Vec<String>,
}

impl SigninInput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_domain(&mut self, window: &Window) -> WalletResult<&mut Self> {
        let host = window.location().host()?;

        self.domain.replace(host);

        Ok(self)
    }

    pub fn set_custom_domain(&mut self, domain: &str) -> &mut Self {
        self.domain.replace(domain.to_string());

        self
    }

    pub fn set_address(&mut self, address: &str) -> WalletResult<&mut Self> {
        let mut buffer = [0u8; 32];
        bs58::decode(address)
            .onto(&mut buffer)
            .or(Err(WalletError::InvalidBase58Address))?;

        self.address.replace(address.to_string());

        Ok(self)
    }

    pub fn set_statement(&mut self, statement: &str) -> &mut Self {
        self.statement.replace(statement.to_string());

        self
    }

    /// URI is automatically set to the `window.location.href`
    /// since if it is not the same, the wallet will ignore it and
    /// show the user an error
    pub fn set_uri(&mut self, window: &Window) -> WalletResult<&mut Self> {
        self.uri.replace(window.location().href()?);

        Ok(self)
    }

    pub fn set_version(&mut self, version: &str) -> &mut Self {
        self.version.replace(version.to_string());

        self
    }

    pub fn set_chain_id(&mut self, cluster: Cluster) -> &mut Self {
        self.chain_id.replace(cluster);

        self
    }

    /// Sets a 32 byte custom nonce
    pub fn set_nonce(&mut self) -> &mut Self {
        use rand_chacha::ChaCha12Rng;
        use rand_core::{RngCore, SeedableRng};

        let mut rng = ChaCha12Rng::from_entropy();

        let mut buffer = [0u8; 32];

        rng.fill_bytes(&mut buffer);

        self.nonce.replace(blake3::hash(&buffer).to_string());
        buffer.fill(0);

        self
    }

    pub fn custom_nonce(&mut self, nonce: &str) -> WalletResult<&mut Self> {
        if nonce.len() < 8 {
            return Err(WalletError::NonceMustBeAtLeast8Characters);
        }
        self.nonce.replace(nonce.to_string());

        Ok(self)
    }

    pub fn time_now() -> WalletResult<SystemTime> {
        let date_now = js_sys::Date::now() as u64;

        Ok(SystemTime::from(
            UNIX_EPOCH
                .checked_add(Duration::from_millis(date_now))
                .ok_or(WalletError::JsError {
                    name: "UNIX_EPOCH.checked_add(js_sys::Date::now()".to_string(),
                    message: "Unable to get the current time".to_string(),
                    stack: "INTERNAL ERROR".to_string(),
                })?,
        ))
    }

    pub fn to_iso860(system_time: SystemTime) -> humantime::Rfc3339Timestamp {
        humantime::format_rfc3339_millis(system_time)
    }

    pub fn set_issued_at(&mut self) -> WalletResult<&mut Self> {
        self.issued_at.replace(Self::time_now()?);

        Ok(self)
    }

    pub fn set_expiration_time_millis(
        &mut self,
        expiration_time_milliseconds: u64,
    ) -> WalletResult<&mut Self> {
        let duration = Duration::from_millis(expiration_time_milliseconds);

        self.set_expiry_internal(duration)
    }

    pub fn set_expiration_time_seconds(
        &mut self,
        expiration_time_seconds: u64,
    ) -> WalletResult<&mut Self> {
        let duration = Duration::from_secs(expiration_time_seconds);

        self.set_expiry_internal(duration)
    }

    fn set_expiry_internal(&mut self, duration: Duration) -> WalletResult<&mut Self> {
        let expiry_time = if let Some(issued_time) = self.issued_at {
            issued_time
                .checked_add(duration)
                .ok_or(WalletError::JsError {
                    name: "SystemTime".to_string(),
                    message: "SystemTime::checked_add(expiration_time_milliseconds) overflow"
                        .to_string(),
                    stack: "INTERNAL ERROR".to_string(),
                })?
        } else {
            Self::time_now()?
        };

        self.set_expiration_time(expiry_time)
    }

    pub fn set_expiration_time(&mut self, expiration_time: SystemTime) -> WalletResult<&mut Self> {
        if let Some(issued_at) = self.issued_at {
            if issued_at > expiration_time {
                return Err(WalletError::ExpiryTimeEarlierThanIssuedTime);
            }
        }

        if Self::time_now()? > expiration_time {
            return Err(WalletError::ExpirationTimeIsInThePast);
        }

        self.expiration_time.replace(expiration_time);

        Ok(self)
    }

    fn set_not_before_internal(&mut self, duration: Duration) -> WalletResult<&mut Self> {
        let not_before = if let Some(issued_time) = self.issued_at {
            issued_time
                .checked_add(duration)
                .ok_or(WalletError::JsError {
                    name: "SystemTime".to_string(),
                    message: "SystemTime::checked_add(not_before_time_milliseconds) overflow"
                        .to_string(),
                    stack: "INTERNAL ERROR".to_string(),
                })?
        } else {
            Self::time_now()?
        };

        self.set_not_before_time(not_before)
    }

    pub fn set_not_before_time_millis(
        &mut self,
        expiration_time_milliseconds: u64,
    ) -> WalletResult<&mut Self> {
        let duration = Duration::from_millis(expiration_time_milliseconds);

        self.set_not_before_internal(duration)
    }

    pub fn set_not_before_time_seconds(
        &mut self,
        expiration_time_seconds: u64,
    ) -> WalletResult<&mut Self> {
        let duration = Duration::from_secs(expiration_time_seconds);

        self.set_not_before_internal(duration)
    }

    fn set_not_before_time(&mut self, not_before: SystemTime) -> WalletResult<&mut Self> {
        if let Some(issued_at) = self.issued_at {
            if issued_at > not_before {
                return Err(WalletError::NotBeforeTimeEarlierThanIssuedTime);
            }
        }

        if Self::time_now()? > not_before {
            return Err(WalletError::NotBeforeTimeIsInThePast);
        }

        if let Some(expiration_time) = self.expiration_time {
            if not_before > expiration_time {
                return Err(WalletError::NotBeforeTimeLaterThanExpirationTime);
            }
        }

        self.not_before.replace(not_before);

        Ok(self)
    }

    pub fn get_object(&self) -> WalletResult<JsValue> {
        let mut signin_input_object = Reflection::new_object();

        signin_input_object.set_object_string_optional("domain", self.domain.as_ref())?;
        signin_input_object.set_object_string_optional("address", self.address.as_ref())?;
        signin_input_object.set_object_string_optional("statement", self.statement.as_ref())?;
        signin_input_object.set_object_string_optional("uri", self.uri.as_ref())?;
        signin_input_object.set_object_string_optional("version", self.version.as_ref())?;
        signin_input_object.set_object_string_optional("address", self.address.as_ref())?;
        signin_input_object.set_object_string_optional(
            "chainId",
            self.chain_id
                .as_ref()
                .map(|cluster| cluster.chain().to_string())
                .as_ref(),
        )?;
        signin_input_object.set_object_string_optional("nonce", self.nonce.as_ref())?;
        signin_input_object
            .set_object_string_optional("issuedAt", self.issued_at_iso8601().as_ref())?;
        signin_input_object.set_object_string_optional(
            "expirationTime",
            self.expiration_time_iso8601().as_ref(),
        )?;
        signin_input_object
            .set_object_string_optional("notBefore", self.not_before_iso8601().as_ref())?;
        signin_input_object.set_object_string_optional("requestId", self.request_id.as_ref())?;

        if !self.resources.is_empty() {
            let stringify_resources = Array::new();
            self.resources.iter().for_each(|resource| {
                stringify_resources.push(&resource.into());
            });
            signin_input_object.set_object(&"resources".into(), &stringify_resources)?;
        }

        Ok(signin_input_object.take())
    }

    pub fn parser(input: &str) -> WalletResult<Self> {
        let mut signin_input = Self::default();

        input
            .split_once(" ")
            .map(|(left, _right)| signin_input.domain.replace(left.trim().to_string()));

        let split_colon = |value: &str| {
            value
                .split_once(":")
                .map(|(_left, right)| right.trim().to_string())
        };

        let split_colon_system_time = |value: &str| -> WalletResult<Option<SystemTime>> {
            value
                .split_once(":")
                .map(|(_left, right)| {
                    humantime::parse_rfc3339(right.trim())
                        .or(Err(WalletError::InvalidISO8601Timestamp(right.to_string())))
                })
                .transpose()
        };

        input
            .split("\n")
            .into_iter()
            .enumerate()
            .try_for_each(|(index, input)| {
                if index == 1 {
                    signin_input.address.replace(input.trim().to_string());
                }

                if index == 3 {
                    signin_input.statement.replace(input.trim().to_string());
                }

                if input.contains("URI") {
                    signin_input.uri = split_colon(input);
                }

                if input.contains("Version") {
                    signin_input.version = split_colon(input);
                }

                if input.contains("Chain ID") {
                    if let Some((_left, right)) = input.split_once(":") {
                        let cluster: Cluster = right.trim().try_into()?;

                        signin_input.chain_id.replace(cluster);
                    }
                }
                if input.contains("Nonce") {
                    signin_input.nonce = split_colon(input);
                }

                if input.contains("Issued At") {
                    signin_input.issued_at = split_colon_system_time(input)?;
                }

                if input.contains("Expiration") {
                    signin_input.expiration_time = split_colon_system_time(input)?;
                }

                if input.contains("Not Before") {
                    signin_input.not_before = split_colon_system_time(input)?;
                }

                if input.contains("Request ID") {
                    signin_input.request_id = split_colon(input);
                }

                if input.starts_with("-") {
                    if let Some(value) = input.split("-").nth(1) {
                        signin_input.resources.push(value.trim().to_string());
                    }
                }

                Ok::<(), WalletError>(())
            })?;

        Ok(signin_input)
    }

    pub fn check_eq(&self, other: &str) -> WalletResult<()> {
        let other = Self::parser(other)?;
        if self.eq(&other) {
            Ok(())
        } else {
            Err(WalletError::MessageReponseMismatch)
        }
    }

    pub fn verify(
        public_key_bytes: [u8; 32],
        message: &[u8],
        signature_bytes: [u8; 64],
    ) -> WalletResult<()> {
        let public_key = Utils::public_key(public_key_bytes)?;
        let signature = Utils::signature(signature_bytes)?;

        Utils::verify_signature(public_key, message, signature)
    }

    pub fn set_request_id(&mut self, id: &str) -> &mut Self {
        self.request_id.replace(id.to_string());

        self
    }

    pub fn add_resource(&mut self, resource: &str) -> &mut Self {
        self.resources.push(resource.to_string());

        self
    }

    pub fn add_resources(&mut self, resources: &[&str]) -> &mut Self {
        resources.iter().for_each(|resource| {
            self.resources.push(resource.to_string());
        });

        self
    }

    pub fn domain(&self) -> Option<&String> {
        self.domain.as_ref()
    }

    pub fn address(&self) -> Option<&String> {
        self.address.as_ref()
    }

    pub fn statement(&self) -> Option<&String> {
        self.statement.as_ref()
    }

    pub fn uri(&self) -> Option<&String> {
        self.uri.as_ref()
    }

    pub fn version(&self) -> Option<&String> {
        self.version.as_ref()
    }

    pub fn chain_id(&self) -> Option<&Cluster> {
        self.chain_id.as_ref()
    }

    pub fn nonce(&self) -> Option<&String> {
        self.nonce.as_ref()
    }

    pub fn issued_at(&self) -> Option<&SystemTime> {
        self.issued_at.as_ref()
    }

    pub fn expiration_time(&self) -> Option<&SystemTime> {
        self.expiration_time.as_ref()
    }

    pub fn not_before(&self) -> Option<&SystemTime> {
        self.not_before.as_ref()
    }

    pub fn issued_at_iso8601(&self) -> Option<String> {
        self.issued_at
            .map(|time_exists| Self::to_iso860(time_exists).to_string())
    }

    pub fn expiration_time_iso8601(&self) -> Option<String> {
        self.expiration_time
            .map(|time_exists| Self::to_iso860(time_exists).to_string())
    }

    pub fn not_before_iso8601(&self) -> Option<String> {
        self.not_before
            .map(|time_exists| Self::to_iso860(time_exists).to_string())
    }

    pub fn request_id(&self) -> Option<&String> {
        self.request_id.as_ref()
    }

    pub fn resources(&self) -> &[String] {
        self.resources.as_slice()
    }
}

#[cfg(test)]
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