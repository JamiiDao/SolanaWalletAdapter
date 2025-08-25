use std::borrow::Cow;

use crate::{
    chains::ChainSupport, feature_support::FeatureSupport, WalletCommonUtils, WalletUtilsError,
    WalletUtilsResult,
};

/// A data URI containing a base64-encoded SVG, WebP, PNG, or GIF image.
/// **NOTE** that this does not contain the browser functions that would be
/// called to perform operations
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WalletAccountData {
    /// Address of the account, corresponding with a public key.
    pub address: String,
    /// Public key of the account, corresponding with a secret key to use.
    pub public_key: [u8; 32],
    /// Chains supported by the account.
    /// This must be a subset of the {@link Wallet.chains | chains} of the Wallet.
    pub chains: Vec<String>,
    /// Feature names supported by the account.
    /// This must be a subset of the names of {@link Wallet.features | features} of the Wallet.
    pub features: Vec<String>,
    /// Optional user-friendly descriptive label or name for the account. This may be displayed by the app.
    pub label: Option<String>,
    /// Optional user-friendly icon for the account. This may be displayed by the app. */
    /// Format `data:image/${'svg+xml' | 'webp' | 'png' | 'gif'};base64,${string}`
    pub icon: Option<String>,
    /// Convenience field, instead of going through the `features` field
    pub supported_features: FeatureSupport,
    /// Convenience field, instead of iteration through the `chains` field
    pub supported_chains: ChainSupport,
}

impl WalletAccountData {
    /// Address of the account, corresponding with a public key.
    pub fn address(&self) -> &str {
        self.address.as_str()
    }

    /// Public key of the account, corresponding with a secret key to use.
    pub fn public_key(&self) -> [u8; 32] {
        self.public_key
    }

    /// Chains supported by the account.
    /// This must be a subset of the {@link Wallet.chains | chains} of the Wallet.
    pub fn chains(&self) -> &[String] {
        self.chains.as_slice()
    }

    /// Feature names supported by the account.
    /// This must be a subset of the names of {@link Wallet.features | features} of the Wallet.
    pub fn features(&self) -> &[String] {
        self.features.as_slice()
    }

    /// Optional user-friendly descriptive label or name for the account. This may be displayed by the app.
    pub fn label(&self) -> Option<&String> {
        self.label.as_ref()
    }

    /// An optional Wallet Icon
    pub fn icon(&self) -> Option<&String> {
        self.icon.as_ref()
    }

    /// Get the shortened address of the `Base58 address` .
    /// It displays the first 4 characters and the last for characters
    /// separated by ellipsis eg `FXdl...RGd4` .
    /// If the address is less than 8 characters, an error is thrown
    pub fn shorten_address<'a>(&'a self) -> WalletUtilsResult<Cow<'a, str>> {
        WalletCommonUtils::shorten_base58(&self.address)
    }

    /// Same as [Self::shorten_address] but with a custom range
    /// instead of taking the first 4 character and the last 4 characters
    /// it uses a custom range.
    pub fn custom_shorten_address<'a>(&'a self, take: usize) -> WalletUtilsResult<Cow<'a, str>> {
        WalletCommonUtils::custom_shorten_base58(&self.address, take)
    }

    /// Same as [Self::shorten_address] but with a custom range
    /// instead of taking the first 4 character and the last 4 characters
    /// it uses a custom range for first characters before ellipsis and last characters after ellipsis.
    pub fn custom_shorten_address_rl<'a>(
        &'a self,
        left: usize,
        right: usize,
    ) -> WalletUtilsResult<Cow<'a, str>> {
        if self.address.len() < left + right {
            return Err(WalletUtilsError::InvalidBase58Address);
        }

        let first_part = &self.address[..left];
        let last_part = &self.address[self.address.len() - right..];

        Ok(Cow::Borrowed(first_part) + "..." + last_part)
    }

    /// Checks if MainNet is supported
    pub fn mainnet(&self) -> bool {
        self.supported_chains.mainnet
    }

    /// Checks if DevNet is supported
    pub fn devnet(&self) -> bool {
        self.supported_chains.devnet
    }

    /// Checks if TestNet is supported
    pub fn testnet(&self) -> bool {
        self.supported_chains.testnet
    }

    /// Checks if LocalNet is supported
    pub fn localnet(&self) -> bool {
        self.supported_chains.localnet
    }

    /// Checks if `standard:connect` is supported
    pub fn standard_connect(&self) -> bool {
        self.supported_features.connect
    }

    /// Checks if `standard:disconnect` is supported
    pub fn standard_disconnect(&self) -> bool {
        self.supported_features.disconnect
    }

    /// Checks if `standard:events` is supported
    pub fn standard_events(&self) -> bool {
        self.supported_features.events
    }

    /// Checks if `solana:signIn` is supported
    pub fn solana_signin(&self) -> bool {
        self.supported_features.sign_in
    }

    /// Checks if `solana:signMessage` is supported
    pub fn solana_sign_message(&self) -> bool {
        self.supported_features.sign_message
    }

    /// Checks if `solana:signAndSendTransaction` is supported
    pub fn solana_sign_and_send_transaction(&self) -> bool {
        self.supported_features.sign_and_send_tx
    }

    /// Checks if `solana:signTransaction` is supported
    pub fn solana_sign_transaction(&self) -> bool {
        self.supported_features.sign_tx
    }
}
