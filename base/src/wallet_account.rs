use crate::{Byte32Array, StandardFeatures, WalletStandardIcon};

pub trait WalletAccount: StandardFeatures {
    fn address(&self) -> &str;

    fn public_key(&self) -> &Byte32Array;

    fn icon(&self) -> Option<WalletStandardIcon>;

    fn label(&self) -> Option<&str>;
}
