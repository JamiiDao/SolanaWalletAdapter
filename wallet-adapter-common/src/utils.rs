use std::{borrow::Cow, time::SystemTime};

use ed25519_dalek::{Signature, Verifier, VerifyingKey};

use crate::{WalletUtilsError, WalletUtilsResult};

/// A 32 byte array representing a Public Key
pub type PublicKeyBytes = [u8; 32];

/// A 64 byte array representing a Signature
pub type SignatureBytes = [u8; 64];

/// Helper utilities
pub struct WalletCommonUtils;

impl WalletCommonUtils {
    /// Generate a public key from random bytes. This is useful for testing
    pub fn public_key_rand() -> [u8; 32] {
        Self::rand_32bytes()
    }

    /// Generate a 32 byte array from random bytes
    pub fn rand_32bytes() -> [u8; 32] {
        use rand_chacha::ChaCha20Rng;
        use rand_core::{RngCore, SeedableRng};

        let mut rng = ChaCha20Rng::from_os_rng();

        let mut buffer = [0u8; 32];

        rng.fill_bytes(&mut buffer);

        buffer
    }

    /// Parse a [PublicKey](VerifyingKey) from an array of 32 bytes
    pub fn public_key(public_key_bytes: &[u8; 32]) -> WalletUtilsResult<VerifyingKey> {
        VerifyingKey::from_bytes(public_key_bytes)
            .or(Err(WalletUtilsError::InvalidEd25519PublicKeyBytes))
    }

    /// Parse a [Signature] from an array of 64 bytes
    pub fn signature(signature_bytes: &[u8; 64]) -> Signature {
        Signature::from_bytes(signature_bytes)
    }

    /// Convert a slice of bytes into a 32 byte array. This is useful especially if a [PublicKey](VerifyingKey) is
    /// given as a slice instead of 32 byte array
    pub fn to32byte_array(bytes: &[u8]) -> WalletUtilsResult<[u8; 32]> {
        bytes
            .try_into()
            .or(Err(WalletUtilsError::Expected32ByteLength))
    }

    /// Convert a slice of bytes into a 64 byte array. This is useful especially if a [Signature] is
    /// given as a slice instead of 64 byte array
    pub fn to64byte_array(bytes: &[u8]) -> WalletUtilsResult<[u8; 64]> {
        bytes
            .try_into()
            .or(Err(WalletUtilsError::Expected64ByteLength))
    }

    /// Verify a [message](str) using a [PublicKey](VerifyingKey) and [Signature]
    pub fn verify_signature(
        public_key: VerifyingKey,
        message: &[u8],
        signature: Signature,
    ) -> WalletUtilsResult<()> {
        public_key
            .verify(message, &signature)
            .or(Err(WalletUtilsError::InvalidSignature))
    }

    /// Verify a [message](str) using a [PublicKey](VerifyingKey) and [Signature]
    pub fn verify(
        public_key_bytes: &[u8; 32],
        message_bytes: &[u8],
        signature_bytes: &[u8; 64],
    ) -> WalletUtilsResult<()> {
        let public_key = Self::public_key(public_key_bytes)?;
        let signature = Self::signature(signature_bytes);

        public_key
            .verify(message_bytes, &signature)
            .or(Err(WalletUtilsError::InvalidSignature))
    }

    /// Generate the Base58 address from a [PublicKey](VerifyingKey)
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
    pub fn shorten_base58<'a>(base58_str: &'a str) -> WalletUtilsResult<Cow<'a, str>> {
        if base58_str.len() < 8 {
            return Err(WalletUtilsError::InvalidBase58Address);
        }

        let first_part = &base58_str[..4];
        let last_part = &base58_str[base58_str.len() - 4..];

        Ok(Cow::Borrowed(first_part) + "..." + last_part)
    }

    /// Same as [Self::shorten_base58] but with a custom range
    /// instead of taking the first 4 character and the last 4 characters
    /// it uses a custom range.
    pub fn custom_shorten_base58<'a>(
        base58_str: &'a str,
        take: usize,
    ) -> WalletUtilsResult<Cow<'a, str>> {
        if base58_str.len() < take + take {
            return Err(WalletUtilsError::InvalidBase58Address);
        }

        let first_part = &base58_str[..take];
        let last_part = &base58_str[base58_str.len() - take..];

        Ok(Cow::Borrowed(first_part) + "..." + last_part)
    }

    /// Same as [Self::shorten_base58] but with a custom range
    /// instead of taking the first 4 character and the last 4 characters
    /// it uses a custom range for first characters before ellipsis and last characters after ellipsis.
    pub fn custom_shorten_address_rl<'a>(
        base58_address: &'a str,
        left: usize,
        right: usize,
    ) -> WalletUtilsResult<Cow<'a, str>> {
        if base58_address.len() < left + right {
            return Err(WalletUtilsError::InvalidBase58Address);
        }

        let first_part = &base58_address[..left];
        let last_part = &base58_address[base58_address.len() - right..];

        Ok(Cow::Borrowed(first_part) + "..." + last_part)
    }

    /// Converts [SystemTime] to ISO 8601 datetime string as required by
    /// Sign In With Solana standard
    pub fn to_iso860(system_time: SystemTime) -> humantime::Rfc3339Timestamp {
        humantime::format_rfc3339_millis(system_time)
    }
}
