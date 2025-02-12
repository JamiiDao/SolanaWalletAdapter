use async_channel::Receiver;
use ed25519_dalek::Signature;
use web_sys::wasm_bindgen::JsValue;

use crate::{
    Cluster, ConnectionInfoInner, Features, Reflection, SemverVersion, WalletAccount, WalletError,
    WalletEventSender, WalletIcon, WalletResult,
};

use super::{
    ChainSupport, FeatureSupport, SendOptions, SignInOutput, SignedMessageOutput, SigninInput,
};

/// A wallet implementing wallet standard
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Wallet {
    name: String,
    version: SemverVersion,
    icon: Option<WalletIcon>,
    accounts: Vec<WalletAccount>,
    chains: Vec<Cluster>,
    pub(crate) features: Features,
    // Convenience field, instead of going through the `features` field
    supported_features: FeatureSupport,
    // Convenience field, instead of iteration through the `chains` field
    supported_chains: ChainSupport,
}

impl Wallet {
    /// Send a request to connect to a browser wallet
    pub async fn connect(&self) -> WalletResult<WalletAccount> {
        self.features.connect.call_connect().await
    }

    /// Send a request to the browser wallet to disconnect
    pub async fn disconnect(&self) -> WalletResult<()> {
        self.features.disconnect.call_disconnect().await
    }

    /// Send a signin request to the browser wallet
    pub async fn sign_in(
        &self,
        signin_input: &SigninInput,
        public_key: [u8; 32],
    ) -> WalletResult<SignInOutput> {
        if let Some(fn_exists) = self.features.sign_in.as_ref() {
            fn_exists.call_signin(signin_input, public_key).await
        } else {
            Err(WalletError::MissingSignInFunction)
        }
    }

    /// Send a sign message request to the browser wallet.
    /// Message must be UTF-8 encoded
    pub async fn sign_message<'a>(
        &self,
        message: &'a [u8],
        account: &WalletAccount,
    ) -> WalletResult<SignedMessageOutput<'a>> {
        self.features
            .sign_message
            .call_sign_message(account, message)
            .await
    }

    /// Send a sign transaction request to the browser wallet.
    /// The transaction bytes expected are encoded using serde in byte form.
    pub async fn sign_transaction(
        &self,
        transaction_bytes: &[u8],
        cluster: Option<Cluster>,
        account: &WalletAccount,
    ) -> WalletResult<Vec<Vec<u8>>> {
        self.features
            .sign_tx
            .call_sign_tx(account, transaction_bytes, cluster)
            .await
    }

    /// Send a sign and send transaction request to the browser wallet.
    pub async fn sign_and_send_transaction(
        &self,
        transaction_bytes: &[u8],
        cluster: Cluster,
        options: SendOptions,
        account: &WalletAccount,
    ) -> WalletResult<Signature> {
        self.features
            .sign_and_send_tx
            .call_sign_and_send_transaction(account, transaction_bytes, cluster, options)
            .await
    }

    /// Get the standard events [Function] `[standard:events].on`
    pub async fn call_on_event(
        &self,
        connection_info: ConnectionInfoInner,
        wallet_name: String,
        sender: WalletEventSender,
        signal_receiver: Receiver<()>,
    ) -> WalletResult<()> {
        self.features
            .events
            .call_on_event(connection_info, wallet_name, sender, signal_receiver)
            .await
    }

    /// Parse the Wallet details from a [JsValue]
    pub fn from_jsvalue(value: JsValue) -> WalletResult<Self> {
        let reflection = Reflection::new(value)?;

        let mut supported_chains = ChainSupport::default();

        let chains_raw = reflection.vec_string_and_filter("chains", "solana:")?;
        let chains = chains_raw
            .into_iter()
            .map(|chain_raw| {
                let cluster = chain_raw.as_str().try_into();
                if let Ok(cluster_inner) = &cluster {
                    if cluster_inner == &Cluster::MainNet {
                        supported_chains.mainnet = true;
                    } else if cluster_inner == &Cluster::DevNet {
                        supported_chains.devnet = true;
                    } else if cluster_inner == &Cluster::TestNet {
                        supported_chains.testnet = true;
                    } else if cluster_inner == &Cluster::LocalNet {
                        supported_chains.localnet = true;
                    }
                }

                cluster
            })
            .collect::<WalletResult<Vec<Cluster>>>()?;

        let name = reflection.string("name")?;
        let version = SemverVersion::parse(&reflection.string("version")?)?;
        let icon = WalletIcon::from_jsvalue(&reflection)?;
        let accounts = Self::get_accounts(&reflection, "accounts")?;
        let (features, supported_features) = Features::parse(&reflection)?;

        Ok(Wallet {
            name,
            version,
            icon,
            accounts,
            chains,
            features,
            supported_features,
            supported_chains,
        })
    }

    fn get_accounts(reflection: &Reflection, key: &str) -> WalletResult<Vec<WalletAccount>> {
        let accounts_raw = reflection.reflect_inner(key)?;

        let accounts_array = Reflection::new(accounts_raw)?.as_array()?;

        accounts_array
            .iter()
            .map(|account| WalletAccount::parse(Reflection::new(account)?))
            .collect::<WalletResult<Vec<WalletAccount>>>()
    }

    /// Get the features of the wallet
    pub fn features(&self) -> &Features {
        &self.features
    }

    /// Get the accounts provided by the wallet
    pub fn accounts(&self) -> &[WalletAccount] {
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

    /// Get the optional [wallet icon](WalletIcon)
    pub fn icon(&self) -> Option<&WalletIcon> {
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

impl core::fmt::Debug for Wallet {
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
            .field("features", &self.features)
            .finish()
    }
}

impl PartialOrd for Wallet {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Wallet {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name
            .as_bytes()
            .cmp(&other.name.as_bytes())
            .then(self.version.cmp(&other.version))
    }
}

impl core::hash::Hash for Wallet {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.as_bytes().hash(state);
        self.version.hash(state);
    }
}
