use std::borrow::Cow;

use async_channel::Receiver;
use ed25519_dalek::Signature;
use wallet_adapter_common::{
    chains::ChainSupport, clusters::Cluster, signin_standard::SignInOutput, WalletData,
};
use web_sys::wasm_bindgen::JsValue;

use crate::{
    ConnectionInfoInner, Features, Reflection, SemverVersion, SigninInput, WalletAccount,
    WalletError, WalletEventSender, WalletIcon, WalletResult,
};

use super::{SendOptions, SignedMessageOutput};

/// A wallet implementing wallet standard
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Wallet {
    pub(crate) data: WalletData,
    pub(crate) accounts: Vec<WalletAccount>,
    pub(crate) features: Features,
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

    /// Get the standard events [Function](web_sys::js_sys::Function) `[standard:events].on`
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
                let cluster: Cluster = chain_raw.as_str().into();
                if cluster == Cluster::MainNet {
                    supported_chains.mainnet = true;
                } else if cluster == Cluster::DevNet {
                    supported_chains.devnet = true;
                } else if cluster == Cluster::TestNet {
                    supported_chains.testnet = true;
                } else if cluster == Cluster::LocalNet {
                    supported_chains.localnet = true;
                }

                cluster
            })
            .collect::<Vec<Cluster>>();

        let name = reflection.string("name")?;
        let version = SemverVersion::parse(&reflection.string("version")?)?;
        let icon = WalletIcon::from_jsvalue(&reflection)?;
        let accounts = Self::get_accounts(&reflection, "accounts")?;
        let (features, supported_features) = Features::parse(&reflection)?;

        let data = WalletData::new()
            .set_name(&name)
            .set_version(
                wallet_adapter_common::SemverVersion::new()
                    .set_major(version.major())
                    .set_minor(version.minor())
                    .set_patch(version.patch()),
            )
            .set_icon(icon.as_ref())
            .replace_accounts(
                accounts
                    .iter()
                    .map(|wallet_account| wallet_account.account.clone())
                    .collect(),
            )
            .replace_chains(chains)
            .set_supported_features(supported_features)
            .set_supported_chains(supported_chains);

        Ok(Self {
            data,
            accounts,
            features,
        })
    }

    fn get_accounts(reflection: &Reflection, key: &str) -> WalletResult<Vec<WalletAccount>> {
        let accounts_raw = reflection.reflect_inner(key)?;

        let accounts_array = Reflection::new(accounts_raw)?.into_array()?;

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
        self.data.chains()
    }

    /// Check whether the wallet supports mainnet cluster
    pub fn mainnet(&self) -> bool {
        self.data.mainnet()
    }

    /// Check whether the wallet supports devnet cluster
    pub fn devnet(&self) -> bool {
        self.data.devnet()
    }

    /// Check whether the wallet supports testnet cluster
    pub fn testnet(&self) -> bool {
        self.data.testnet()
    }

    /// Check whether the wallet supports localnet cluster
    pub fn localnet(&self) -> bool {
        self.data.localnet()
    }

    /// Check whether the wallet supports `standard:connect` feature
    pub fn standard_connect(&self) -> bool {
        self.data.standard_connect()
    }

    /// Check whether the wallet supports `standard:disconnect` feature
    pub fn standard_disconnect(&self) -> bool {
        self.data.standard_disconnect()
    }

    /// Check whether the wallet supports `standard:events` feature
    pub fn standard_events(&self) -> bool {
        self.data.standard_events()
    }

    /// Check whether the wallet supports `solana:signIn` feature
    pub fn solana_signin(&self) -> bool {
        self.data.solana_signin()
    }

    /// Check whether the wallet supports `solana:signMessage` feature
    pub fn solana_sign_message(&self) -> bool {
        self.data.solana_sign_message()
    }

    /// Check whether the wallet supports `solana:signAndSendTransaction` feature
    pub fn solana_sign_and_send_transaction(&self) -> bool {
        self.data.solana_sign_and_send_transaction()
    }

    /// Check whether the wallet supports `solana:signTransaction` feature
    pub fn solana_sign_transaction(&self) -> bool {
        self.data.solana_sign_transaction()
    }

    /// Get the optional wallet icon
    pub fn icon(&self) -> Option<&Cow<'static, str>> {
        self.data.icon()
    }

    /// Get the name of the wallet
    pub fn name(&self) -> &str {
        self.data.name()
    }

    /// Get the version of the wallet standard that the wallet supports
    pub fn version(&self) -> SemverVersion {
        let version = self.data.version();

        SemverVersion(version.clone())
    }
}
