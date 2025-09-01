use crate::{SemverVersion, StandardFeatures, WalletStandardIcon};

pub trait Wallet: StandardFeatures {
    fn label(&self) -> &str;

    fn version(&self) -> SemverVersion;

    fn icon(&self) -> Option<WalletStandardIcon>;
}
