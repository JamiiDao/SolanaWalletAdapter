use std::borrow::Borrow;

use ed25519_dalek::Signature;
use web_sys::{js_sys::Object, Document, Window};

use crate::{
    events::InitEvents, Cluster, SendOptions, SignInOutput, SignedMessageOutput, SigninInput,
    Wallet, WalletAccount, WalletError, WalletResult, WalletStorage,
};

/// Operations on a browser window.
/// `Window` and `Document` object must be present otherwise
/// an error is thrown.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WalletAdapter {
    window: Window,
    document: Document,
    storage: WalletStorage,
    connected_wallet: Option<Wallet>,
    connected_account: Option<WalletAccount>,
}

impl WalletAdapter {
    /// Get the `Window` and `Document` object in the current browser window
    pub fn init() -> WalletResult<Self> {
        let storage = WalletStorage::default();

        let window = if let Some(window) = web_sys::window() {
            window
        } else {
            return Err(WalletError::MissingAccessToBrowserWindow);
        };

        let document = if let Some(document) = window.document() {
            document
        } else {
            return Err(WalletError::MissingAccessToBrowserDocument);
        };

        let mut new_self = Self {
            window: window.clone(),
            document,
            storage,
            connected_wallet: Option::default(),
            connected_account: Option::default(),
        };

        InitEvents::new(&window).init(&mut new_self.storage)?;

        Ok(new_self)
    }

    /// Send a connect request to the browser wallet
    pub async fn connect(&mut self, wallet: Wallet) -> WalletResult<WalletAccount> {
        let wallet_account = wallet.features.connect.call_connect().await?;

        self.set_connected_account(wallet_account.clone());
        self.set_connected_wallet(wallet);

        Ok(wallet_account)
    }

    /// Lookup a wallet entry by name from the registered wallets
    /// and then send a connect request to the browser extension wallet
    pub async fn connect_by_name(&mut self, wallet_name: &str) -> WalletResult<WalletAccount> {
        let wallet = self.get_wallet(wallet_name)?;

        let wallet_account = wallet.features.connect.call_connect().await?;

        self.set_connected_account(wallet_account.clone());
        self.set_connected_wallet(wallet);

        Ok(wallet_account)
    }

    /// Send a disconnect request to the browser wallet
    pub async fn disconnect(&mut self) -> WalletResult<()> {
        if let Some(wallet) = self.connected_wallet.take() {
            wallet.disconnect().await?;
            self.set_disconnected();

            Ok(())
        } else {
            Err(WalletError::NoConnectedWallet)
        }
    }

    /// Send a sign and send transaction request to the browser wallet
    pub async fn sign_and_send_transaction(
        &self,
        transaction_bytes: &[u8],
        cluster: Cluster,
        options: SendOptions,
    ) -> WalletResult<Signature> {
        let account = self.connected_account()?;
        let wallet = self.connected_wallet()?;
        wallet
            .sign_and_send_transaction(transaction_bytes, cluster, options, account)
            .await
    }

    /// Send a connect request to the browser wallet
    pub async fn sign_transaction(
        &self,
        transaction_bytes: &[u8],
        cluster: Option<Cluster>,
    ) -> WalletResult<Vec<Vec<u8>>> {
        let account = self.connected_account()?;
        let wallet = self.connected_wallet()?;
        wallet
            .sign_transaction(transaction_bytes, cluster, account)
            .await
    }

    /// Send a sign message request to the browser wallet
    pub async fn sign_message<'a>(
        &self,
        message: &'a [u8],
    ) -> WalletResult<SignedMessageOutput<'a>> {
        let account = self.connected_account()?;
        let wallet = self.connected_wallet()?;

        wallet.sign_message(message, account).await
    }

    /// Send a sign in request to the browser wallet to Sign In With Solana
    pub async fn sign_in(
        &self,
        signin_input: &SigninInput,
        public_key: [u8; 32],
    ) -> WalletResult<SignInOutput> {
        let wallet = self.connected_wallet()?;

        wallet.sign_in(signin_input, public_key).await
    }

    /// Set the connected account
    pub fn set_connected_account(&mut self, account_name: WalletAccount) -> &mut Self {
        self.connected_account.replace(account_name);

        self
    }

    /// Set the connected wallet
    pub fn set_connected_wallet(&mut self, wallet: Wallet) -> &mut Self {
        self.connected_wallet.replace(wallet);

        self
    }

    /// Set the disconnected account
    pub fn set_disconnected(&mut self) -> &mut Self {
        self.connected_wallet.take();
        self.connected_account.take();

        self
    }

    /// Check if an [account](WalletAccount) is connected
    pub fn is_connected(&self) -> bool {
        self.connected_account.is_some()
    }

    /// Get the connected [account](WalletAccount)
    pub fn connected_account(&self) -> WalletResult<&WalletAccount> {
        self.connected_account
            .as_ref()
            .ok_or(WalletError::NoConnectedAccount)
    }

    /// Get the connected [wallet](Wallet)
    pub fn connected_wallet(&self) -> WalletResult<&Wallet> {
        self.connected_wallet
            .as_ref()
            .ok_or(WalletError::NoConnectedWallet)
    }

    /// Get an entry in the `Window` object
    pub fn get_entry(&self, property: &str) -> Option<Object> {
        self.window.get(property)
    }

    /// Get the browser window
    pub fn window(&self) -> &Window {
        &self.window
    }

    /// Get the browser document
    pub fn document(&self) -> &Document {
        &self.document
    }

    /// Get the storage where the adapter stores the registered wallets
    pub fn storage(&self) -> &WalletStorage {
        self.storage.borrow()
    }

    /// Get the clusters supported by the connected wallet
    pub fn clusters(&self) -> WalletResult<Vec<Cluster>> {
        let mut clusters = Vec::<Cluster>::default();

        if self.mainnet()? {
            clusters.push(Cluster::MainNet);
        }
        if self.devnet()? {
            clusters.push(Cluster::DevNet);
        }
        if self.localnet()? {
            clusters.push(Cluster::LocalNet);
        }
        if self.testnet()? {
            clusters.push(Cluster::TestNet);
        }

        Ok(clusters)
    }

    /// Get the registered wallets
    pub fn wallets(&self) -> Vec<Wallet> {
        self.storage.borrow().get_wallets()
    }

    /// Get a certain wallet by its name
    pub fn get_wallet(&self, wallet_name: &str) -> WalletResult<Wallet> {
        self.storage
            .get_wallet(wallet_name)
            .ok_or(WalletError::NoConnectedWallet)
    }

    /// Check if the connected wallet supports mainnet cluster
    pub fn mainnet(&self) -> WalletResult<bool> {
        Ok(self.connected_wallet()?.mainnet())
    }

    /// Check if the connected wallet supports devnet cluster
    pub fn devnet(&self) -> WalletResult<bool> {
        Ok(self.connected_wallet()?.devnet())
    }

    /// Check if the connected wallet supports testnet cluster
    pub fn testnet(&self) -> WalletResult<bool> {
        Ok(self.connected_wallet()?.testnet())
    }

    /// Check if the connected wallet supports localnet cluster
    pub fn localnet(&self) -> WalletResult<bool> {
        Ok(self.connected_wallet()?.localnet())
    }

    /// Check if the connected wallet supports `standard:connect` feature
    pub fn standard_connect(&self) -> WalletResult<bool> {
        Ok(self.connected_wallet()?.standard_connect())
    }

    /// Check if the connected wallet supports `standard:disconnect` feature
    pub fn standard_disconnect(&self) -> WalletResult<bool> {
        Ok(self.connected_wallet()?.standard_disconnect())
    }

    /// Check if the connected wallet supports `standard:events` feature
    pub fn standard_events(&self) -> WalletResult<bool> {
        Ok(self.connected_wallet()?.standard_events())
    }

    /// Check if the connected wallet supports `solana:signIn` feature
    pub fn solana_signin(&self) -> WalletResult<bool> {
        Ok(self.connected_wallet()?.solana_signin())
    }

    /// Check if the connected wallet supports `solana:signMessage` feature
    pub fn solana_sign_message(&self) -> WalletResult<bool> {
        Ok(self.connected_wallet()?.solana_sign_message())
    }

    /// Check if the connected wallet supports `solana:signAndSendTransaction` feature
    pub fn solana_sign_and_send_transaction(&self) -> WalletResult<bool> {
        Ok(self.connected_wallet()?.solana_sign_and_send_transaction())
    }

    /// Check if the connected wallet supports `solana:signTransaction` feature
    pub fn solana_sign_transaction(&self) -> WalletResult<bool> {
        Ok(self.connected_wallet()?.solana_sign_transaction())
    }
}
