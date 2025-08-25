use std::borrow::Cow;

use crate::{
    chains::ChainSupport, clusters::Cluster, feature_support::FeatureSupport, SemverVersion,
    WalletAccountData,
};

/// Wallet information without any browser function calls for `wallet-adapter` standard operations
#[derive(Clone, Default, PartialEq, Eq)]
pub struct WalletData {
    name: Cow<'static, str>,
    version: SemverVersion,
    icon: Option<Cow<'static, str>>,
    accounts: Vec<WalletAccountData>,
    chains: Vec<Cluster>,
    // Convenience field, instead of going through the `features` field
    supported_features: FeatureSupport,
    // Convenience field, instead of iteration through the `chains` field
    supported_chains: ChainSupport,
}

impl WalletData {
    /// Instantiate a new [WalletData]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the name of the wallet
    pub fn set_name(mut self, name: &str) -> Self {
        self.name = Cow::Owned(name.to_string());

        self
    }

    /// Set the [Semver version](SemverVersion) of the wallet
    pub fn set_version(mut self, version: SemverVersion) -> Self {
        self.version = version;

        self
    }

    /// Set the icon of the wallet.
    /// Should be in Base64 URL web format `data:image/${'svg+xml' | 'webp' | 'png' | 'gif'};base64,${string}`
    pub fn set_icon(mut self, icon: Option<impl ToString>) -> Self {
        icon.map(|value| self.icon.replace(Cow::Owned(value.to_string())));

        self
    }

    /// Add a [Wallet account](WalletAccountData) data
    pub fn add_account(mut self, account: WalletAccountData) -> Self {
        self.accounts.push(account);

        self
    }

    /// Add multiple [Wallet account](WalletAccountData) data(s)
    pub fn add_accounts(mut self, accounts: &[WalletAccountData]) -> Self {
        self.accounts.extend_from_slice(accounts);

        self
    }

    /// Replace all [Wallet account](WalletAccountData) data
    pub fn replace_accounts(mut self, accounts: Vec<WalletAccountData>) -> Self {
        self.accounts = accounts;

        self
    }

    /// Add a [Cluster]
    pub fn add_chain(mut self, chain: Cluster) -> Self {
        self.chains.push(chain);

        self
    }

    /// Add multiple [Cluster]s
    pub fn add_chains(mut self, chains: &[Cluster]) -> Self {
        self.chains.extend_from_slice(chains);

        self
    }

    /// Replace existing [Cluster]s
    pub fn replace_chains(mut self, chains: Vec<Cluster>) -> Self {
        self.chains = chains;

        self
    }

    /// Set the supported wallet [features](FeatureSupport)
    pub fn set_supported_features(mut self, supported_features: FeatureSupport) -> Self {
        self.supported_features = supported_features;

        self
    }

    /// Set the [chains](ChainSupport) supported by the wallet
    pub fn set_supported_chains(mut self, supported_chains: ChainSupport) -> Self {
        self.supported_chains = supported_chains;

        self
    }

    /// Get the accounts provided by the wallet
    pub fn accounts(&self) -> &[WalletAccountData] {
        &self.accounts
    }

    /// Get the chains supported by the wallet
    pub fn chains(&self) -> &[Cluster] {
        &self.chains
    }

    /// Check whether the wallet supports mainnet cluster
    pub fn mainnet(&self) -> bool {
        self.supported_chains.mainnet
    }

    /// Check whether the wallet supports devnet cluster
    pub fn devnet(&self) -> bool {
        self.supported_chains.devnet
    }

    /// Check whether the wallet supports testnet cluster
    pub fn testnet(&self) -> bool {
        self.supported_chains.testnet
    }

    /// Check whether the wallet supports localnet cluster
    pub fn localnet(&self) -> bool {
        self.supported_chains.localnet
    }

    /// Check whether the wallet supports `standard:connect` feature
    pub fn standard_connect(&self) -> bool {
        self.supported_features.connect
    }

    /// Check whether the wallet supports `standard:disconnect` feature
    pub fn standard_disconnect(&self) -> bool {
        self.supported_features.disconnect
    }

    /// Check whether the wallet supports `standard:events` feature
    pub fn standard_events(&self) -> bool {
        self.supported_features.events
    }

    /// Check whether the wallet supports `solana:signIn` feature
    pub fn solana_signin(&self) -> bool {
        self.supported_features.sign_in
    }

    /// Check whether the wallet supports `solana:signMessage` feature
    pub fn solana_sign_message(&self) -> bool {
        self.supported_features.sign_message
    }

    /// Check whether the wallet supports `solana:signAndSendTransaction` feature
    pub fn solana_sign_and_send_transaction(&self) -> bool {
        self.supported_features.sign_and_send_tx
    }

    /// Check whether the wallet supports `solana:signTransaction` feature
    pub fn solana_sign_transaction(&self) -> bool {
        self.supported_features.sign_tx
    }

    /// Get the optional Wallet Icon
    pub fn icon(&self) -> Option<&Cow<'static, str>> {
        self.icon.as_ref()
    }

    /// Get the name of the wallet
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the version of the wallet standard that the wallet supports
    pub fn version(&self) -> &SemverVersion {
        &self.version
    }
}

impl core::fmt::Debug for WalletData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let chains = self
            .chains
            .iter()
            .map(|cluster| cluster.chain())
            .collect::<Vec<&str>>();

        f.debug_struct("Wallet")
            .field("name", &self.name)
            .field("version", &self.version)
            .field("icon", &self.icon)
            .field("accounts", &self.accounts)
            .field("chains", &chains)
            .finish()
    }
}

impl PartialOrd for WalletData {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for WalletData {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name
            .as_bytes()
            .cmp(other.name.as_bytes())
            .then(self.version.cmp(&other.version))
    }
}

impl core::hash::Hash for WalletData {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.as_bytes().hash(state);
        self.version.hash(state);
    }
}
