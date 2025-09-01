use core::{fmt::Debug, hash::Hash};
use std::{
    borrow::Cow,
    time::{Duration, SystemTime},
};

use crate::{BaseUtils, Cluster, RandomBytes, WalletAccount, WalletBaseError, WalletBaseResult};

/// The Sign In input used as parameters when performing
/// `SignInWithSolana (SIWS)` requests as defined by the
/// [SIWS](https://github.com/phantom/sign-in-with-solana) standard.
/// A backup fork can be found at [https://github.com/JamiiDao/sign-in-with-solana](https://github.com/JamiiDao/sign-in-with-solana)
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SignInInput<'wa> {
    /// Optional EIP-4361 domain requesting the sign-in.
    /// If not provided, the wallet must determine the domain to include in the message.
    domain: Option<Cow<'wa, str>>,
    /// Optional Solana Base58 address performing the sign-in.
    /// The address is case-sensitive.
    /// If not provided, the wallet must determine the Address to include in the message.
    address: Option<Cow<'wa, str>>,
    /// Optional EIP-4361 Statement.
    /// The statement is a human readable string and should not have new-line characters (\n).
    /// If not provided, the wallet does not include Statement in the message.
    statement: Option<Cow<'wa, str>>,
    /// Optional EIP-4361 URI.
    /// The URL that is requesting the sign-in.
    /// If not provided, the wallet does not include URI in the message.
    uri: Option<Cow<'wa, str>>,
    /// Optional EIP-4361 version.
    /// If not provided, the wallet does not include Version in the message.
    version: Option<Cow<'wa, str>>,
    /// Optional EIP-4361 Chain ID.
    /// The chainId can be one of the following:
    /// mainnet, testnet, devnet, localnet, solana:mainnet, solana:testnet, solana:devnet.
    /// If not provided, the wallet does not include Chain ID in the message.
    chain_id: Option<Cow<'wa, str>>,
    /// Optional EIP-4361 Nonce.
    /// It should be an alphanumeric string containing a minimum of 8 characters.
    /// If not provided, the wallet does not include Nonce in the message.
    nonce: Option<Cow<'wa, str>>,
    /// Optional ISO 8601 datetime string.
    /// This represents the time at which the sign-in request was issued to the wallet.
    /// Note: For Phantom, issuedAt has a threshold and it should be
    /// within +- 10 minutes from the timestamp at which verification is taking place.
    /// If not provided, the wallet does not include Issued At in the message.
    issued_at: Option<SystemTime>,
    /// Optional ISO 8601 datetime string.
    /// This represents the time at which the sign-in request should expire.
    /// If not provided, the wallet does not include Expiration Time in the message.
    expiration_time: Option<SystemTime>,
    /// Optional ISO 8601 datetime string.
    /// This represents the time at which the sign-in request becomes valid.
    /// If not provided, the wallet does not include Not Before in the message.
    not_before: Option<SystemTime>,
    /// Optional EIP-4361 Request ID.
    /// In addition to using nonce to avoid replay attacks,
    /// dapps can also choose to include a unique signature in the requestId .
    /// Once the wallet returns the signed message,
    /// dapps can then verify this signature against the state to add an additional,
    /// strong layer of security. If not provided, the wallet does not include Request ID in the message.
    request_id: Option<Cow<'wa, str>>,
    /// Optional EIP-4361 Resources.
    /// Usually a list of references in the form of URIs that the
    /// dapp wants the user to be aware of.
    /// These URIs should be separated by \n-, ie,
    /// URIs in new lines starting with the character -.
    /// If not provided, the wallet does not include Resources in the message.
    resources: Cow<'wa, [&'wa str]>,
}

impl<'wa> SignInInput<'wa> {
    /// Same as `Self::default()` as it initializes [Self] with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// An EIP-4361 domain requesting the sign-in.
    /// If not provided, the wallet must determine the domain to include in the message.
    pub fn set_domain(&mut self, domain: &'wa str) -> &mut Self {
        self.domain.replace(Cow::Borrowed(domain));

        self
    }

    /// The Base58 public key address
    /// NOTE: Some wallets require this field or
    /// an error `MessageResponseMismatch` is returned which is as
    /// a result of the sent message not corresponding with the signed message
    pub fn set_address(&'wa mut self, address: &'wa str) -> WalletBaseResult<'wa, &'wa mut Self> {
        let mut buffer = [0u8; 32];
        let buffer_written_len = bs58::decode(address).onto(&mut buffer).or(Err(
            WalletBaseError::InvalidBase58Address(Cow::Borrowed(address)),
        ))?;

        if buffer_written_len != 32 {
            return Err(WalletBaseError::InvalidEd25519PublicKeyLen(
                buffer_written_len as u8,
            ));
        }

        self.address.replace(Cow::Borrowed(address));

        Ok(self)
    }
    ///  An EIP-4361 Statement which is a human readable string and should not have new-line characters (\n).
    /// Sets the message that is shown to the user during Sign In With Solana
    pub fn set_statement(&mut self, statement: &'wa str) -> &mut Self {
        self.statement.replace(Cow::Borrowed(statement));

        self
    }

    /// An EIP-4361 URI is automatically set to the `window.location.href`
    /// since if it is not the same, the wallet will ignore it and
    /// show the user an error.
    /// This is the URL that is requesting the sign-in.
    pub fn set_uri(&mut self, uri: &'wa str) -> &mut Self {
        self.uri.replace(Cow::Borrowed(uri));

        self
    }

    /// An EIP-4361 version.
    /// Sets the version
    pub fn set_version(&mut self, version: &'wa str) -> &mut Self {
        self.version.replace(Cow::Borrowed(version));

        self
    }

    /// An EIP-4361 Chain ID.
    /// The chainId can be one of the following:
    /// mainnet, testnet, devnet, localnet, solana:mainnet, solana:testnet, solana:devnet.
    pub fn set_chain_id(&mut self, cluster: impl Cluster) -> &mut Self {
        self.chain_id
            .replace(Cow::Owned(cluster.chain().to_string()));

        self
    }

    /// An EIP-4361 Nonce which is an alphanumeric string containing a minimum of 8 characters.
    /// This is generated from the Cryptographically Secure Random Number Generator
    /// and the bytes converted to hex formatted string.
    pub fn set_nonce(&mut self) -> &mut Self {
        let random_bytes = RandomBytes::<32>::generate();

        self.nonce
            .replace(Cow::Owned(blake3::hash(random_bytes.expose()).to_string()));

        self
    }

    /// An EIP-4361 Nonce which is an alphanumeric string containing a minimum of 8 characters.
    /// This is generated from the Cryptographically Secure Random Number Generator
    /// and the bytes converted to hex formatted string.
    pub fn set_custom_nonce(
        &'wa mut self,
        nonce: &'wa str,
    ) -> WalletBaseResult<'wa, &'wa mut Self> {
        let nonce_length = nonce.len();
        if nonce_length < 8 {
            return Err(WalletBaseError::NonceMustBeAtLeast8Characters(
                nonce_length as u8,
            ));
        }

        self.nonce.replace(Cow::Borrowed(nonce));

        Ok(self)
    }

    ///  This represents the time at which the sign-in request was issued to the wallet.
    /// Note: For Phantom, issuedAt has a threshold and it should be within +- 10 minutes
    /// from the timestamp at which verification is taking place.
    /// If not provided, the wallet does not include Issued At in the message.
    /// This also follows the ISO 8601 datetime.
    pub fn set_issued_at(&mut self, time: SystemTime) -> &mut Self {
        self.issued_at.replace(time);

        self
    }

    /// An ergonomic method for [Self::set_expiration_time()]
    /// where you can add milliseconds and [SystemTime] is automatically calculated for you
    pub fn set_expiration_time_millis(
        &'wa mut self,
        now: SystemTime,
        expiration_time_milliseconds: u64,
    ) -> WalletBaseResult<'wa, &'wa mut Self> {
        let duration = Duration::from_millis(expiration_time_milliseconds);

        self.set_expiry_internal(now, duration)
    }

    /// An ergonomic method for [Self::set_expiration_time()]
    /// where you can add seconds and [SystemTime] is automatically calculated for you
    pub fn set_expiration_time_seconds(
        &'wa mut self,
        now: SystemTime,
        expiration_time_seconds: u64,
    ) -> WalletBaseResult<'wa, &'wa mut Self> {
        let duration = Duration::from_secs(expiration_time_seconds);

        self.set_expiry_internal(now, duration)
    }

    fn set_expiry_internal(
        &'wa mut self,
        now: SystemTime,
        duration: Duration,
    ) -> WalletBaseResult<'wa, &'wa mut Self> {
        let expiry_time = if let Some(issued_time) = self.issued_at {
            issued_time
                .checked_add(duration)
                .ok_or(WalletBaseError::SystemTimeCheckedAddOverflow)?
        } else {
            now
        };

        self.set_expiration_time(now, expiry_time)
    }

    /// An ISO 8601 datetime string. This represents the time at which the sign-in request should expire.
    /// If not provided, the wallet does not include Expiration Time in the message.
    /// Expiration time should be in future or an error will be thrown even before a request to the wallet is sent
    pub fn set_expiration_time(
        &'wa mut self,
        now: SystemTime,
        expiration_time: SystemTime,
    ) -> WalletBaseResult<'wa, &'wa mut Self> {
        if let Some(issued_at) = self.issued_at {
            if issued_at > expiration_time {
                let issued = BaseUtils::to_iso860(issued_at).to_string();
                let expiry = BaseUtils::to_iso860(expiration_time).to_string();

                return Err(WalletBaseError::ExpiryTimeEarlierThanIssuedTime {
                    issued: issued.into(),
                    expiry: expiry.into(),
                });
            }
        }

        if now > expiration_time {
            let now = BaseUtils::to_iso860(now).to_string();
            let expiry = BaseUtils::to_iso860(expiration_time).to_string();
            return Err(WalletBaseError::ExpirationTimeIsInThePast {
                now: now.into(),
                expiry: expiry.into(),
            });
        }

        self.expiration_time.replace(expiration_time);

        Ok(self)
    }

    fn set_not_before_internal(
        &'wa mut self,
        now: SystemTime,
        duration: Duration,
    ) -> WalletBaseResult<'wa, &'wa mut Self> {
        let not_before = if let Some(issued_time) = self.issued_at {
            issued_time
                .checked_add(duration)
                .ok_or(WalletBaseError::SystemTimeCheckedAddOverflow)?
        } else {
            now
        };

        self.set_not_before_time(now, not_before)
    }

    /// An ergonomic method for [Self::set_not_before_time()]
    /// where you can add milliseconds and [SystemTime] is automatically calculated for you
    pub fn set_not_before_time_millis(
        &'wa mut self,
        now: SystemTime,
        expiration_time_milliseconds: u64,
    ) -> WalletBaseResult<'wa, &'wa mut Self> {
        let duration = Duration::from_millis(expiration_time_milliseconds);

        self.set_not_before_internal(now, duration)
    }

    /// An ergonomic method for [Self::set_not_before_time()]
    /// where you can add seconds and [SystemTime] is automatically calculated for you
    pub fn set_not_before_time_seconds(
        &'wa mut self,
        now: SystemTime,
        expiration_time_seconds: u64,
    ) -> WalletBaseResult<'wa, &'wa mut Self> {
        let duration = Duration::from_secs(expiration_time_seconds);

        self.set_not_before_internal(now, duration)
    }

    /// An ISO 8601 datetime string.
    /// This represents the time at which the sign-in request becomes valid.
    /// If not provided, the wallet does not include Not Before in the message.
    /// Time must be after `IssuedTime`
    pub fn set_not_before_time(
        &mut self,
        now: SystemTime,
        not_before: SystemTime,
    ) -> WalletBaseResult<'wa, &mut Self> {
        if let Some(issued_at) = self.issued_at {
            if issued_at > not_before {
                let issued = BaseUtils::to_iso860(issued_at).to_string();
                let not_before = BaseUtils::to_iso860(not_before).to_string();
                return Err(WalletBaseError::NotBeforeTimeEarlierThanIssuedTime {
                    issued_at: issued.into(),
                    not_before: not_before.into(),
                });
            }
        }

        if now > not_before {
            let now = BaseUtils::to_iso860(now).to_string();
            let not_before = BaseUtils::to_iso860(not_before).to_string();

            return Err(WalletBaseError::NotBeforeTimeIsInThePast {
                now: now.into(),
                not_before: not_before.into(),
            });
        }

        if let Some(expiration_time) = self.expiration_time {
            if not_before > expiration_time {
                let expiry = BaseUtils::to_iso860(expiration_time).to_string();
                let not_before = BaseUtils::to_iso860(not_before).to_string();
                // TODO add a check for this in the build()
                return Err(WalletBaseError::NotBeforeTimeLaterThanExpirationTime {
                    not_before: not_before.into(),
                    expiry: expiry.into(),
                });
            }
        }

        self.not_before.replace(not_before);

        Ok(self)
    }

    /// Parses the Sign In With Solana (SIWS) result of the Response from a wallet
    pub fn parser(input: &'wa str) -> WalletBaseResult<'wa, Self> {
        let mut signin_input = Self::default();

        input
            .split_once(" ")
            .map(|(left, _right)| signin_input.domain.replace(left.trim().into()));

        let split_colon = |value: &str| {
            value
                .split_once(":")
                .map(|(_left, right)| Cow::Owned(right.trim().into()))
        };

        let split_colon_system_time = |value: &'wa str| -> WalletBaseResult<Option<SystemTime>> {
            value
                .split_once(":")
                .map(|(_left, right)| {
                    humantime::parse_rfc3339(right.trim())
                        .or(Err(WalletBaseError::InvalidISO8601Timestamp(right.into())))
                })
                .transpose()
        };

        input
            .split("\n")
            .enumerate()
            .try_for_each(|(index, input)| {
                if index == 1 {
                    signin_input.address.replace(input.trim().into());
                }

                if index == 3 {
                    signin_input.statement.replace(input.trim().into());
                }

                if input.contains("URI") {
                    signin_input.uri = split_colon(input);
                }

                if input.contains("Version") {
                    signin_input.version = split_colon(input);
                }

                if input.contains("Chain ID") {
                    if let Some((_left, right)) = input.split_once(":") {
                        let cluster = right.trim().into();

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
                        signin_input.resources.to_mut().push(value.trim());
                    }
                }

                Ok::<(), WalletBaseError>(())
            })?;

        Ok(signin_input)
    }

    /// Checks if the response of a Sign In With Solana (SIWS) from the Wallet is the same as the
    /// request data sent to the wallet to be signed
    pub fn check_eq(&'wa self, other: &'wa str) -> WalletBaseResult<'wa, ()> {
        let other = Self::parser(other)?;

        if self.eq(&other) {
            Ok(())
        } else {
            Err(WalletBaseError::MessageResponseMismatch)
        }
    }

    /// An EIP-4361 Request ID.
    /// In addition to using nonce to avoid replay attacks,
    /// dapps can also choose to include a unique signature in the requestId .
    /// Once the wallet returns the signed message,
    /// dapps can then verify this signature against the state to add an additional,
    /// strong layer of security. If not provided, the wallet must not include Request ID in the message.
    pub fn set_request_id(&mut self, id: &'wa str) -> &mut Self {
        self.request_id.replace(id.into());

        self
    }

    /// An EIP-4361 Resources.
    /// Usually a list of references in the form of URIs that the dapp wants the user to be aware of.
    /// These URIs should be separated by \n-, ie, URIs in new lines starting with the character -.
    /// If not provided, the wallet must not include Resources in the message.
    pub fn add_resource(&mut self, resource: &'wa str) -> &mut Self {
        self.resources.to_mut().push(resource);

        self
    }

    /// Helper for [Self::add_resource()] when you want to add multiple resources at the same time
    pub fn add_resources(&mut self, resources: &'wa [&'wa str]) -> &mut Self {
        self.resources.to_mut().extend_from_slice(resources);

        self
    }

    /// Get the `domain` field
    pub fn domain(&self) -> Option<&str> {
        self.domain.as_deref()
    }

    /// Get the `address` field
    pub fn address(&self) -> Option<&str> {
        self.address.as_deref()
    }

    /// Get the `statement` field
    pub fn statement(&self) -> Option<&str> {
        self.statement.as_deref()
    }

    /// Get the `uri` field
    pub fn uri(&self) -> Option<&str> {
        self.uri.as_deref()
    }

    /// Get the `version` field
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    /// Get the `chain_id` field
    pub fn chain_id(&self) -> Option<&str> {
        self.chain_id.as_deref()
    }

    /// Get the `nonce` field
    pub fn nonce(&self) -> Option<&str> {
        self.nonce.as_deref()
    }

    /// Get the `issued_at` field
    pub fn issued_at(&self) -> Option<&SystemTime> {
        self.issued_at.as_ref()
    }

    /// Get the `expiration_time` field
    pub fn expiration_time(&self) -> Option<&SystemTime> {
        self.expiration_time.as_ref()
    }

    /// Get the `not_before` field
    pub fn not_before(&self) -> Option<&SystemTime> {
        self.not_before.as_ref()
    }

    /// Get the `issued_at` field as ISO8601 date time string
    pub fn issued_at_iso8601(&self) -> Option<String> {
        self.issued_at
            .map(|time_exists| BaseUtils::to_iso860(time_exists).to_string())
    }

    /// Get the `expiration_time` field as ISO8601 date time string
    pub fn expiration_time_iso8601(&self) -> Option<String> {
        self.expiration_time
            .map(|time_exists| BaseUtils::to_iso860(time_exists).to_string())
    }

    /// Get the `not_before` field as ISO8601 date time string
    pub fn not_before_iso8601(&self) -> Option<String> {
        self.not_before
            .map(|time_exists| BaseUtils::to_iso860(time_exists).to_string())
    }

    /// Get the `request_id` field
    pub fn request_id(&self) -> Option<&str> {
        self.request_id.as_deref()
    }

    /// Get the `resources` field
    pub fn resources(&'wa self) -> &'wa [&'wa str] {
        &self.resources
    }
}

/// The output of Sign In With Solana (SIWS) response from a wallet
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct SignInOutput<T: WalletAccount + Debug + Clone + Hash + Ord + PartialEq + Eq + Default> {
    /// A [An Account](WalletAccountData)
    pub account: T,
    /// The UTF-8 encoded message
    pub message: String,
    /// The signature as a  byte array of 64 bytes in length corresponding to a
    /// [Ed25519 Signature](ed25519_dalek::Signature)
    pub signature: [u8; 64],
    /// The public key as a  byte array of 32 bytes in length corresponding to a
    /// [Ed25519 Public Key](ed25519_dalek::VerifyingKey)
    pub public_key: [u8; 32],
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
