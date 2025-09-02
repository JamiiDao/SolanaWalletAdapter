use core::fmt::Debug;

use crate::Byte64Array;

pub trait SignTransactionInput {
    fn sign_transaction_input<T: Debug + PartialEq>(&self, bytes: T) -> &[u8];
}

/// Represents a `signedTransaction` type in JSON that can be deserialized
pub trait SignTransactionOutput {
    fn signed_transaction(&self) -> &[u8];

    fn verify_signed_transaction<OutputError: core::error::Error>(&self)
        -> Result<(), OutputError>;
}

/// Input should be the same as [SignTransactionInput]
pub trait SignAndSendTransactionOutput {
    fn signature(&self) -> &Byte64Array;
}
