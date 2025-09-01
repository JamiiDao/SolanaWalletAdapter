use std::borrow::Cow;

pub type WalletBaseResult<'wa, T> = Result<T, WalletBaseError<'wa>>;

#[derive(Debug, PartialEq, Eq, Clone, thiserror::Error)]
pub enum WalletBaseError<'wa> {
    #[error("Invalid Base58 string: `{0}`")]
    InvalidBase58Address(Cow<'wa, str>),
    #[error(
        "The public key length is expected to be 32 bytes but encountered public key of a length `{0}` bytes"
    )]
    InvalidEd25519PublicKeyLen(u8),
    #[error(
        "Sign In nonce is required to be at least 8 characters long but given nonce is `{0}` characters long!"
    )]
    NonceMustBeAtLeast8Characters(u8),
    #[error("Adding one system time to the other caused overflow")]
    SystemTimeCheckedAddOverflow,
    #[error(
        "The issued time should be earlier than the expiry time; Issued at: `{issued}`, Expires at: `{expiry}`"
    )]
    ExpiryTimeEarlierThanIssuedTime {
        issued: Cow<'wa, str>,
        expiry: Cow<'wa, str>,
    },
    #[error(
        "The expiry time should be later than the current time; Expires at: `{expiry}`, Current tine at: `{now}`"
    )]
    ExpirationTimeIsInThePast {
        now: Cow<'wa, str>,
        expiry: Cow<'wa, str>,
    },
    #[error(
        "The not before time be later than the issued time; Issued at: `{issued_at}`, Not Before Time at: `{not_before}`"
    )]
    NotBeforeTimeEarlierThanIssuedTime {
        issued_at: Cow<'wa, str>,
        not_before: Cow<'wa, str>,
    },
    #[error(
        "The not before set is earlier than current time; Current time at: `{now}`, Not Before Time at: `{not_before}`"
    )]
    NotBeforeTimeIsInThePast {
        now: Cow<'wa, str>,
        not_before: Cow<'wa, str>,
    },
    #[error(
        "The expiration time is earlier that not before time. A token cannot expire before it's valid; Expiry time at: `{expiry}`, Not Before Time at: `{not_before}`"
    )]
    NotBeforeTimeLaterThanExpirationTime {
        not_before: Cow<'wa, str>,
        expiry: Cow<'wa, str>,
    },
    #[error("Encountered an invalid ISO8601 timestamp")]
    InvalidISO8601Timestamp(Cow<'wa, str>),
    #[error("The message that was sent to be signed did not match the message that was received")]
    MessageResponseMismatch,
}
