use thiserror::Error;
use web_sys::js_sys::{wasm_bindgen::JsValue, Reflect};

use crate::WalletEvent;

/// A Result<T, WalletError>
pub type WalletResult<T> = Result<T, WalletError>;

impl From<async_channel::SendError<WalletEvent>> for WalletError {
    fn from(value: async_channel::SendError<WalletEvent>) -> Self {
        match value {
            async_channel::SendError(_) => Self::ChannelError,
        }
    }
}

/// Error handling enum
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Error)]
pub enum WalletError {
    /// Unable to send the a [WalletEvent] via the [crate::WalletEventSender]
    #[error("Unable to send the a `WalletEvent` variant via the WalletEventSender channel")]
    ChannelError,
    /// An JavaScript Error corresponding to a [wasm_bindgen::JsValue] .
    /// It contains the error type represented by `name`,
    /// the error message `message`
    /// and the `stack` message which offers a trace of which functions were called.
    /// Learn about this error type from [Error - Mozilla Developer Network](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Error)
    #[error("JsError{{ name: {name}, message: {message}, stack: {stack} }}")]
    JsError {
        /// The name of the error from a Javascript error message
        name: String,
        /// The message contained in the error
        message: String,
        /// The stack from the JavaScript error message
        stack: String,
    },
    /// An internal error that occurs when casting a JavaScript types or DOM types to a Rust type using [js_sys::JsCast] trait
    /// and `foo.dyn_ref::<T>()` or `foo.dyn_into::<T>()` where `foo`
    /// is a variable of type [wasm_bindgen::JsValue]. These error can also occur when trying to parse a browser error from JavaScript value
    /// into a `WalletError::JsError`. Open an issue describing this error at - https://github.com/JamiiDao/SolanaWalletAdapter/issues
    #[error("Internal Error `{0}` occurred, this is a bug in the library please open an issue at https://github.com/JamiiDao/SolanaWalletAdapter/issues")]
    InternalError(String),
    /// A value of `undefined` or `null` was encountered
    #[error("A value of `undefined` or `null` was encountered")]
    ValueNotFound,
    /// A value was expected but it doesn't exist in the `JsValue`
    #[error("A value of `{0}` was expected but it dosent't exist in the `JsValue`")]
    ExpectedValueNotFound(String),
    /// Unable to access browser window
    #[error("Unable to access browser window")]
    MissingAccessToBrowserWindow,
    /// Unable to access browser document
    #[error("Unable to access browser document")]
    MissingAccessToBrowserDocument,
    /// Only `processed`, `confirmed` and `finalized` commitments are supported by Solana clusters
    #[error("Unsupported Commitment level `{0}`. Only `processed`, `confirmed` and `finalized` commitments are supported by Solana clusters")]
    UnsupportedCommitment(String),
    /// The wallet version is invalid, expected SemVer version
    #[error("The wallet version `{0}` is invalid, expected SemVer version")]
    InvalidWalletVersion(String),
    /// Unexpected SemVer number to parse to a `u8`
    #[error("Unexpected SemVer number `{0}` to parse to a `u8`")]
    InvalidSemVerNumber(String),
    /// The byte length should be equal to 32 bytes in length
    #[error("The byte length should be equal to 32 bytes in length")]
    Expected32ByteLength,
    /// The byte length should be equal to 64 bytes in length
    #[error("The byte length should be equal to 64 bytes in length")]
    Expected64ByteLength,
    /// The version was not found
    #[error("The version was not found")]
    VersionNotFound,
    /// This feature is not supported as a standard  or solana namespace feature
    #[error("The feature `{0}` is not supported as a standard  or solana namespace feature")]
    UnsupportedWalletFeature(String),
    /// Encountered an unsupported transaction version.
    /// Only `legacy` and `version zero` transactions are supported.
    #[error("Encountered an unsupported transaction version. Only `legacy` and `version zero` transactions are supported.")]
    UnsupportedTransactionVersion,
    /// Legacy transaction versions need to be supported yet the encountered wallet does not do this.
    #[error("Legacy transaction versions need to be supported yet the encountered wallet does not do this.")]
    LegacyTransactionSupportRequired,
    /// The blockchain encountered is not supported.
    #[error("The blockchain `{0}` is not supported")]
    UnsupportedChain(String),
    /// The `connect` function of the `standard:connect` namespace was not found while parsing a wallet
    #[error("The `connect` function of the `standard:connect` namespace was not found while parsing a wallet")]
    MissingConnectFunction,
    /// Attempted to connect to a wallet that does not exist or is yet to be registered
    #[error("Attempted to connect to a wallet that does not exist or is yet to be registered")]
    WalletNotFound,
    /// Attempted to connect to an account that does not exist or might have been disconnected
    #[error(
        "Attempted to connect to an account that does not exist or might have been disconnected"
    )]
    AccountNotFound,
    /// Unable to connect to a wallet. The user may have rejected the request
    #[error("Unable to connect to a wallet. Error `{0}` request")]
    WalletConnectError(String),
    /// The connect method did not return any accounts
    #[error("The connect method did not return any accounts")]
    ConnectHasNoAccounts,
    /// The wallet `standard:disconnect` feature is missing
    #[error("The wallet `standard:disconnect` feature is missing")]
    MissingDisconnectFunction,
    /// The `accounts` method to get the accounts connected to a wallet is missing from wallet
    #[error("The `accounts` method to get the accounts connected to a wallet is missing from wallet `{0}`")]
    MissingGetAccountsFunction(String),
    /// Unable to disconnect wallet.
    #[error("Wallet Disconnect error - `{0}`")]
    WalletDisconnectError(String),
    /// Encountered an error while calling `standard:events` function
    #[error("Encountered `standard:events` error `{0}`")]
    StandardEventsError(String),
    /// Called The Function for `standard:events` yet the wallet does not provide it
    #[error("Called The Function for `standard:events` yet the wallet does not provide it")]
    MissingStandardEventsFunction,
    /// The wallet did not register a signIn function for `solana:signIn` namespece
    #[error("The wallet did not register a signIn function for `solana:signIn` namespece")]
    MissingSignInFunction,
    /// This token expires earlier than it was issued. Make sure to set the expiry time to be a later date than the issued time
    #[error("This token expires earlier than it was issued. Make sure to set the expiry time to be a later date than the issued time")]
    ExpiryTimeEarlierThanIssuedTime,
    /// This token becomes valid earlier than it was issued. Make sure to set the not_before time to be equal to or a later date than the issued time
    #[error("This token becomes valid earlier than it was issued. Make sure to set the not_before time to be equal to or a later date than the issued time")]
    NotBeforeTimeEarlierThanIssuedTime,
    /// This token becomes valid after it has already expired. Make sure to set the not_before time to be equal to or a date before expiry time
    #[error("This token becomes valid after it has already expired. Make sure to set the not_before time to be equal to or a date before expiry time")]
    NotBeforeTimeLaterThanExpirationTime,
    /// The expiration time is set to expire in the past
    #[error("The expiration time is set to expire in the past")]
    ExpirationTimeIsInThePast,
    /// NotBefore time is set in the past
    #[error("NotBefore time is set in the past")]
    NotBeforeTimeIsInThePast,
    /// Invalid Base58 Address
    #[error("Invalid Base58 Address")]
    InvalidBase58Address,
    /// The nonce is required to be at least 8 characters long
    #[error("The nonce is required to be at least 8 characters long")]
    NonceMustBeAtLeast8Characters,
    ///Expected a timestamp in the format specified by ISO8601
    #[error("Invalid ISO 8601 timestamp `{0}. Only timestamps in the format specified by ISO8601 are supported.")]
    InvalidISO8601Timestamp(String),
    /// The message signed by the wallet is not the same as the message sent to the wallet for signing
    #[error("The message signed by the wallet is not the same as the message sent to the wallet for signing")]
    MessageResponseMismatch,
    /// The Ed25519 Signature is invalid for the signed message and public key")]
    #[error("The Ed25519 Signature is invalid for the signed message and public key")]
    InvalidSignature,
    /// The bytes provided for the Ed25519 Signature are invalid
    #[error("The bytes provided for the Ed25519 Signature are invalid")]
    InvalidEd25519SignatureBytes,
    /// The bytes provided for the Ed25519 Public Key are invalid
    #[error("The bytes provided for the Ed25519 Public Key are invalid")]
    InvalidEd25519PublicKeyBytes,
    /// The function call to Sign A Message Is Missing
    #[error("The function call to Sign A Message Is Missing")]
    MissingSignMessageFunction,
    /// The message sent to the wallet to be signed is different from the message the wallet responded with
    #[error("The message sent to the wallet to be signed is different from the message the wallet responded with")]
    SignedMessageMismatch,
    /// The Wallet returned an empty array of  signed messages
    #[error("The Wallet returned an empty array of  signed messages")]
    ReceivedAnEmptySignedMessagesArray,
    /// The `solana:signTransaction` function is missing in the provided wallet
    #[error("The `solana:signTransaction` function is missing in the provided wallet")]
    MissingSignTransactionFunction,
    /// The `sendAndSignTransaction` method did not return any signature
    #[error("The `sendAndSignTransaction` method did not return any signature")]
    SendAndSignTransactionSignatureEmpty,
    /// An operation resulted in an error. This is a convenience error that you can use to return any error
    /// that was not caused by the wallet adapter, example, parsing a recipient address or the result of parsing
    /// the body of a HTTP response using serde resulted in an error. Remember, this error is not from the [crate::WalletAdapter]
    /// but instead an external error.
    #[error("An operation resulted in an error `{0}`.")]
    Op(String),
}

impl From<JsValue> for WalletError {
    fn from(value: JsValue) -> Self {
        let reflect = |key: &str| -> Result<String, Self> {
            Reflect::get(&value, &key.into())
                .map_err(|error: JsValue| WalletError::InternalError(format!("{:?}", &error)))?
                .as_string()
                .ok_or(WalletError::InternalError(format!(
                    "Reflecting `{key}` in `{value:?}` did not yield a JsString"
                )))
        };

        let name = match reflect("name") {
            Ok(inner) => inner,
            Err(error) => return error,
        };

        let stack = match reflect("stack") {
            Ok(inner) => inner,
            Err(error) => return error,
        };
        let message = match reflect("message") {
            Ok(inner) => inner,
            Err(error) => return error,
        };

        Self::JsError {
            message,
            name,
            stack,
        }
    }
}
