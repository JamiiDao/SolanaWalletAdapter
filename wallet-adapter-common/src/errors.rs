/// Convenience type for `Result<T, WalletUtilsError>`
pub type WalletUtilsResult<T> = Result<T, WalletUtilsError>;

/// Errors for this crate
#[derive(Debug, PartialEq, Eq, Clone, thiserror::Error)]
pub enum WalletUtilsError {
    /// Overflow during SystemTime::checked_add(expiration_time_milliseconds) overflow
    #[error("SystemTime::checked_add(expiration_time_milliseconds) overflow")]
    SystemTimeCheckedAddOverflow,
    /// This token expires earlier than it was issued. Make sure to set the expiry time to be a later date than the issued time
    #[error("This token expires earlier than it was issued. Make sure to set the expiry time to be a later date than the issued time")]
    ExpiryTimeEarlierThanIssuedTime,
    /// The expiration time is set to expire in the past
    #[error("The expiration time is set to expire in the past")]
    ExpirationTimeIsInThePast,
    /// This token becomes valid earlier than it was issued. Make sure to set the not_before time to be equal to or a later date than the issued time
    #[error("This token becomes valid earlier than it was issued. Make sure to set the not_before time to be equal to or a later date than the issued time")]
    NotBeforeTimeEarlierThanIssuedTime,
    /// NotBefore time is set in the past
    #[error("NotBefore time is set in the past")]
    NotBeforeTimeIsInThePast,
    /// This token becomes valid after it has already expired. Make sure to set the not_before time to be equal to or a date before expiry time
    #[error("This token becomes valid after it has already expired. Make sure to set the not_before time to be equal to or a date before expiry time")]
    NotBeforeTimeLaterThanExpirationTime,
    ///Expected a timestamp in the format specified by ISO8601
    #[error("Invalid ISO 8601 timestamp `{0}. Only timestamps in the format specified by ISO8601 are supported.")]
    InvalidISO8601Timestamp(String),
    /// Invalid Base58 Address
    #[error("Invalid Base58 Address")]
    InvalidBase58Address,
    /// The bytes provided for the Ed25519 Public Key are invalid
    #[error("The bytes provided for the Ed25519 Public Key are invalid")]
    InvalidEd25519PublicKeyBytes,
    /// The Ed25519 Signature is invalid for the signed message and public key")]
    #[error("The Ed25519 Signature is invalid for the signed message and public key")]
    InvalidSignature,
    /// The byte length should be equal to 64 bytes in length
    #[error("The byte length should be equal to 64 bytes in length")]
    Expected64ByteLength,
    /// The byte length should be equal to 32 bytes in length
    #[error("The byte length should be equal to 32 bytes in length")]
    Expected32ByteLength,
    /// The nonce is required to be at least 8 characters long
    #[error("The nonce is required to be at least 8 characters long")]
    NonceMustBeAtLeast8Characters,
    /// The message signed by the wallet is not the same as the message sent to the wallet for signing
    #[error("The message signed by the wallet is not the same as the message sent to the wallet for signing")]
    MessageResponseMismatch,
}
