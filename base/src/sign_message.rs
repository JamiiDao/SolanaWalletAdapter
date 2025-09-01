use crate::{Byte32Array, Byte64Array};

pub trait SignMessageInput: Utf8Message {
    fn message(&self) -> &[u8];
}

pub trait SignedMessageOutput<'a> {
    fn message(&self) -> &[u8];

    fn public_key(&self) -> &Byte32Array;

    fn signature(&self) -> &Byte64Array;

    fn verify_message<OutputError: core::error::Error>(&self) -> Result<(), OutputError>;
}

pub trait Utf8Message: AsRef<str> + AsRef<[u8]> {}
