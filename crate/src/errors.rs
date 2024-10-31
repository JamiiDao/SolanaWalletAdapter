use js_sys::{wasm_bindgen::JsValue, Reflect};
use thiserror::Error;

/// A Result<T, WalletError>
pub type WalletResult<T> = Result<T, WalletError>;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Error)]
pub enum WalletError {
    /// An JavaScript Error corresponding to a [wasm_bindgen::JsValue] .
    /// It contains the error type represented by `name`,
    /// the error message `message`
    /// and the `stack` message which offers a trace of which functions were called.
    /// Learn about this error type from [Error - Mozilla Developer Network](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Error)
    #[error("JsError{{ name: {name}, message: {message}, stack: {stack} }}")]
    JsError {
        name: String,
        message: String,
        stack: String,
    },
    /// Unable to parse an `Err(JsValue)` to get the `name`, `message` or `stack`. One, some or all of these values might be missing
    #[error("Unable to parse an `Err(JsValue)` to get the `name`, `message` or `stack`. One, some or all of these values might be missing")]
    UnableToParseJsError,
    /// Attempted to convert a JsValue to a String where a String was expected
    #[error(" Attempted to convert a JsValue to a String where a String was expected")]
    JsValueNotString,
    /// Attempted to convert a JsError to a String
    #[error("Attempted to convert a JsError to a String")]
    JsErrorNotString,
    /// A value of `undefined` or `null` was encountered
    #[error("A value of `undefined` or `null` was encountered")]
    ValueNotFound,
    /// A value was expected but it dosen't exist in the `JsValue`
    #[error("A value of `{0}` was expected but it dosen't exist in the `JsValue`")]
    ExpectedValueNotFound(String),
    /// Unable to access browser window
    #[error("Unable to access browser window")]
    MissingAccessToBrowserWindow,
    /// Unable to access browser document
    #[error("Unable to access browser document")]
    MissingAccessToBrowserDocument,
    /// Only `processed`, `confirmed` and `finalized` commitments are supported by Solana clusters
    #[error("Unsupported Commmitment level `{0}`. Only `processed`, `confirmed` and `finalized` commitments are supported by Solana clusters")]
    UnsupportedCommitment(String),
    /// Unable to cast a wasm_bindgen closure to Function
    #[error("Unable to cast a wasm_bindgen closure to Function")]
    CastClosureToFunction,
    /// The wallet version is invalid, expected SemVer version
    #[error("The wallet version `{0}` is invalid, expected SemVer version")]
    InvalidWalletVersion(String),
    /// Unexpected SemVer number to parse to a `u8`
    #[error("Unexpected SemVer number `{0}` to parse to a `u8`")]
    InvalidSemVerNumber(String),
    /// Expected an array JsValue.
    #[error("Expected an array `{0}` JsValue.")]
    ExpectedArray(String),
    /// Expected an `String` JsValue.
    #[error("Expected an `{0}` String JsValue.")]
    ExpectedString(String),
    /// The byte length should be equal to 32 bytes in length
    #[error("The byte length should be equal to 32 bytes in length")]
    Expected32ByteLength,
    /// Expected the JsValue to be an Object
    #[error("Expected the `{0}` JsValue to be an Object")]
    ExpectedObject(String),
    /// The version was not found
    #[error("The version was not found")]
    VersionNotFound,
    /// This feature is not supported as a standard feature
    #[error("The feature `{0}` is not supported as a standard feature")]
    UnsupportedStandardFeature(String),
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
}

impl WalletError {
    pub fn not_found(&self) -> bool {
        self == &WalletError::ValueNotFound
    }
}

impl From<JsValue> for WalletError {
    fn from(value: JsValue) -> Self {
        let reflect = |key: &str| -> Result<String, Self> {
            Reflect::get(&value, &key.into())
                .map_err(|_: JsValue| WalletError::UnableToParseJsError)?
                .as_string()
                .map_or(Err(WalletError::JsErrorNotString), |inner| Ok(inner))
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
