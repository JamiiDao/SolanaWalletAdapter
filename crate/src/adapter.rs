use std::{borrow::Borrow, sync::Arc};

use async_channel::{bounded, Receiver};
use async_lock::RwLock;
use ed25519_dalek::Signature;
use wallet_adapter_common::{clusters::Cluster, signin_standard::SignInOutput};
use web_sys::{js_sys::Object, Document, Window};

use crate::{
    events::InitEvents, send_wallet_event, SendOptions, SignedMessageOutput, SigninInput, Wallet,
    WalletAccount, WalletError, WalletEvent, WalletEventReceiver, WalletEventSender, WalletResult,
    WalletStorage,
};

/// Contains the connected wallet and account.
/// Containing them in the same struct allows passing of this type
/// by containing it in types like [Arc] and [RwLock] when moving the type
/// out of it's scope like in background tasks or async functions (`async move`).
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConnectionInfo {
    wallet: Option<Wallet>,
    account: Option<WalletAccount>,
    previous_accounts: Vec<WalletAccount>,
}

impl ConnectionInfo {
    /// Create a default [ConnectionInfo]
    pub fn new() -> Self {
        ConnectionInfo::default()
    }

    /// Set the connected wallet
    pub fn set_wallet(&mut self, wallet: Wallet) -> &mut Self {
        self.wallet.replace(wallet);

        self
    }

    /// Set the connected account
    pub fn set_account(&mut self, account: WalletAccount) -> &mut Self {
        self.account.replace(account);

        self
    }

    /// Send a connect request to the browser wallet
    pub async fn connect(&mut self, sender: WalletEventSender) -> WalletResult<WalletAccount> {
        let wallet = self.connected_wallet()?;

        let connected_account = wallet.features.connect.call_connect().await?;

        self.set_account(connected_account.clone());

        send_wallet_event(WalletEvent::Connected(connected_account.clone()), sender).await;

        Ok(connected_account)
    }

    /// Set the disconnected account
    pub async fn set_disconnected(&mut self, sender: WalletEventSender) -> &mut Self {
        self.wallet.take();
        self.account.take();
        self.previous_accounts.clear();

        send_wallet_event(WalletEvent::Disconnected, sender).await;

        self
    }

    /// Get the connected [wallet](Wallet)
    pub fn connected_wallet(&self) -> WalletResult<&Wallet> {
        self.wallet.as_ref().ok_or(WalletError::WalletNotFound)
    }

    /// Get the connected [account](WalletAccount)
    pub fn connected_account(&self) -> WalletResult<&WalletAccount> {
        self.account.as_ref().ok_or(WalletError::AccountNotFound)
    }

    /// Get the connected [wallet](Wallet) but return an [Option]
    /// to show the wallet exists instead of a [WalletResult]
    pub fn connected_wallet_raw(&self) -> Option<&Wallet> {
        self.wallet.as_ref()
    }

    /// Get the connected [account](WalletAccount)
    /// but return an [Option] to show the account exists instead of a [WalletResult]
    pub fn connected_account_raw(&self) -> Option<&WalletAccount> {
        self.account.as_ref()
    }

    /// Emit an [event](WalletEvent) after processing the `[standard:events].on` result
    pub async fn emit_wallet_event(
        &mut self,
        wallet_name: &str,
        account_processing: Option<WalletAccount>,
        sender: WalletEventSender,
    ) {
        match self.connected_wallet() {
            Ok(wallet) => {
                let event_outcome = match account_processing {
                    Some(connected_account) => {
                        if self.account.is_none()
                            && self.wallet.is_none()
                            && self.previous_accounts.is_empty()
                        {
                            self.set_account(connected_account.clone());

                            WalletEvent::Connected(connected_account)
                        } else if self.account.is_none()
                            && self.wallet.is_some()
                            && self.previous_accounts.iter().any(|wallet_account| {
                                wallet_account.account.public_key
                                    == connected_account.account.public_key
                            })
                        {
                            self.push_previous_account();
                            self.set_account(connected_account.clone());

                            WalletEvent::Connected(connected_account)
                        } else if wallet.name().as_bytes() == wallet_name.as_bytes()
                            && self.account.is_none()
                            && self.previous_accounts.iter().any(|wallet_account| {
                                wallet_account.account.public_key
                                    == connected_account.account.public_key
                            })
                        {
                            self.push_previous_account();
                            self.set_account(connected_account.clone());

                            WalletEvent::Reconnected(connected_account)
                        } else if wallet.name().as_bytes() == wallet_name.as_bytes()
                            && self.account.is_some()
                        {
                            self.push_previous_account();
                            self.set_account(connected_account.clone());

                            WalletEvent::AccountChanged(connected_account)
                        } else {
                            WalletEvent::Skip
                        }
                    }
                    None => {
                        if wallet.name().as_bytes() == wallet_name.as_bytes() {
                            self.push_previous_account();
                            WalletEvent::Disconnected
                        } else {
                            WalletEvent::Skip
                        }
                    }
                };

                send_wallet_event(event_outcome, sender).await
            }
            Err(error) => {
                web_sys::console::log_2(
                    &"ON EVENT EMITTED BUT NO CONNECTED WALLET FOUND: ".into(),
                    &format!("{error:?}").into(),
                );
            }
        }
    }

    fn push_previous_account(&mut self) {
        let take_connected_account = self.account.take();

        if let Some(connected_account_inner) = take_connected_account {
            self.previous_accounts.push(connected_account_inner);
        }

        self.previous_accounts.dedup();
    }
}

/// The [ConnectionInfo] wrapped in an `Arc<RwLock<T>>`
pub type ConnectionInfoInner = Arc<RwLock<ConnectionInfo>>;

/// Operations on a browser window.
/// `Window` and `Document` object must be present otherwise
/// an error is thrown.
#[derive(Debug, Clone)]
pub struct WalletAdapter {
    window: Window,
    document: Document,
    storage: WalletStorage,
    connection_info: ConnectionInfoInner,
    wallet_events: WalletEventReceiver,
    wallet_events_sender: WalletEventSender,
    signal_receiver: Receiver<()>,
}

impl WalletAdapter {
    /// Get the `Window` and `Document` object in the current browser window,
    /// initialize the `AppReady` and `Register` events of the wallet standard
    /// and creates a bounded channel with capacity default of 5 messages before capacity is filled.
    /// Use [WalletAdapter::init_with_channel_capacity] to initialize with a desired channel capacity.
    pub fn init() -> WalletResult<Self> {
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

        Self::init_with_channel_capacity_window_and_document(5, window, document)
    }

    /// Get the `Window` and `Document` object in the current browser window,
    /// initialize the `AppReady` and `Register` events of the wallet standard
    /// and creates a bounded channel with user-specified capacity.
    pub fn init_with_channel_capacity(capacity: usize) -> WalletResult<Self> {
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

        Self::init_with_channel_capacity_window_and_document(capacity, window, document)
    }

    /// Same as [WalletAdapter::init] but a `capacity` value
    /// can be passed to create an channel with a desired capacity
    #[allow(clippy::arc_with_non_send_sync)]
    pub fn init_with_channel_capacity_window_and_document(
        capacity: usize,
        window: Window,
        document: Document,
    ) -> WalletResult<Self> {
        let storage = WalletStorage::default();

        let (sender, receiver) = bounded::<WalletEvent>(capacity);
        let (_, signal_receiver) = bounded::<()>(capacity);

        let mut new_self = Self {
            window: window.clone(),
            document,
            storage,
            connection_info: Arc::new(RwLock::new(ConnectionInfo::default())),
            wallet_events: receiver,
            wallet_events_sender: sender,
            signal_receiver,
        };

        InitEvents::new(&window).init(&mut new_self)?;

        Ok(new_self)
    }

    /// Initializes with a [web_sys::Window] and [web_sys::Document] that have been
    /// initialized elsewhere. For example some Rust frontend frameworks already
    /// expose the window and document objects, you could pass them here.
    pub fn init_custom(window: Window, document: Document) -> WalletResult<Self> {
        Self::init_with_channel_capacity_window_and_document(5, window, document)
    }

    /// Listen for [WalletEvent] to be notified when a wallet
    /// receives `connected`, `disconnected` and `accountChanged` events triggered
    /// when the `change` event is dispatched by a connected browser extension
    pub fn events(&self) -> WalletEventReceiver {
        self.wallet_events.clone()
    }

    /// Send a connect request to the browser wallet
    pub async fn connect(&mut self, wallet: Wallet) -> WalletResult<WalletAccount> {
        let wallet_name = wallet.name().to_string();
        let sender = self.wallet_events_sender.clone();
        let signal_receiver = self.signal_receiver.clone();

        if self.connection_info().await.connected_wallet().is_ok() {
            let capacity = signal_receiver.capacity().unwrap_or(5);
            let (_, signal_receiver) = bounded::<()>(capacity);
            self.signal_receiver = signal_receiver;
        }

        let wallet_account = self
            .connection_info
            .write()
            .await
            .set_wallet(wallet)
            .connect(sender.clone())
            .await?;

        self.connection_info()
            .await
            .connected_wallet()?
            .call_on_event(
                self.connection_info.clone(),
                wallet_name,
                sender,
                signal_receiver,
            )
            .await?;

        Ok(wallet_account)
    }

    /// Lookup a wallet entry by name from the registered wallets
    /// and then send a connect request to the browser extension wallet
    pub async fn connect_by_name(&mut self, wallet_name: &str) -> WalletResult<WalletAccount> {
        let wallet = self.get_wallet(wallet_name)?;

        self.connect(wallet).await
    }

    /// Send a disconnect request to the browser wallet
    pub async fn disconnect(&mut self) {
        let sender = self.wallet_events_sender.clone();

        self.connection_info
            .write()
            .await
            .set_disconnected(sender)
            .await;
        self.signal_receiver.close();
    }

    /// Send a sign in request to the browser wallet to Sign In With Solana
    pub async fn sign_in(
        &self,
        signin_input: &SigninInput,
        public_key: [u8; 32],
    ) -> WalletResult<SignInOutput> {
        self.connection_info()
            .await
            .connected_wallet()?
            .sign_in(signin_input, public_key)
            .await
    }

    /// Send a sign and send transaction request to the browser wallet
    pub async fn sign_and_send_transaction(
        &self,
        transaction_bytes: &[u8],
        cluster: Cluster,
        options: SendOptions,
    ) -> WalletResult<Signature> {
        let connection_info = self.connection_info();

        connection_info
            .await
            .connected_wallet()?
            .sign_and_send_transaction(
                transaction_bytes,
                cluster,
                options,
                self.connection_info().await.connected_account()?,
            )
            .await
    }

    /// Send a connect request to the browser wallet
    pub async fn sign_transaction(
        &self,
        transaction_bytes: &[u8],
        cluster: Option<Cluster>,
    ) -> WalletResult<Vec<Vec<u8>>> {
        let connection_info = self.connection_info();

        connection_info
            .await
            .connected_wallet()?
            .sign_transaction(
                transaction_bytes,
                cluster,
                self.connection_info().await.connected_account()?,
            )
            .await
    }

    /// Send a sign message request to the browser wallet
    pub async fn sign_message<'a>(
        &self,
        message: &'a [u8],
    ) -> WalletResult<SignedMessageOutput<'a>> {
        let connection_info = self.connection_info();

        self.connection_info()
            .await
            .connected_wallet()?
            .sign_message(message, connection_info.await.connected_account()?)
            .await
    }

    /// Check if an [account](WalletAccount) is connected
    pub async fn is_connected(&self) -> bool {
        self.connection_info
            .as_ref()
            .write()
            .await
            .account
            .is_some()
    }

    /// Get the connected [ConnectionInfo] containing the
    /// [account](WalletAccount) and [wallet](Wallet)
    pub async fn connection_info(&self) -> async_lock::RwLockReadGuard<'_, ConnectionInfo> {
        self.connection_info.as_ref().read().await
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
    pub async fn clusters(&self) -> WalletResult<Vec<Cluster>> {
        let mut clusters = Vec::<Cluster>::default();

        if self.mainnet().await? {
            clusters.push(Cluster::MainNet);
        }
        if self.devnet().await? {
            clusters.push(Cluster::DevNet);
        }
        if self.localnet().await? {
            clusters.push(Cluster::LocalNet);
        }
        if self.testnet().await? {
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
            .ok_or(WalletError::WalletNotFound)
    }

    /// Check if the connected wallet supports mainnet cluster
    pub async fn mainnet(&self) -> WalletResult<bool> {
        Ok(self.connection_info().await.connected_wallet()?.mainnet())
    }

    /// Check if the connected wallet supports devnet cluster
    pub async fn devnet(&self) -> WalletResult<bool> {
        Ok(self.connection_info().await.connected_wallet()?.devnet())
    }

    /// Check if the connected wallet supports testnet cluster
    pub async fn testnet(&self) -> WalletResult<bool> {
        Ok(self.connection_info().await.connected_wallet()?.testnet())
    }

    /// Check if the connected wallet supports localnet cluster
    pub async fn localnet(&self) -> WalletResult<bool> {
        Ok(self.connection_info().await.connected_wallet()?.localnet())
    }

    /// Check if the connected wallet supports `standard:connect` feature
    pub async fn standard_connect(&self) -> WalletResult<bool> {
        Ok(self
            .connection_info()
            .await
            .connected_wallet()?
            .standard_connect())
    }

    /// Check if the connected wallet supports `standard:disconnect` feature
    pub async fn standard_disconnect(&self) -> WalletResult<bool> {
        Ok(self
            .connection_info()
            .await
            .connected_wallet()?
            .standard_disconnect())
    }

    /// Check if the connected wallet supports `standard:events` feature
    pub async fn standard_events(&self) -> WalletResult<bool> {
        Ok(self
            .connection_info()
            .await
            .connected_wallet()?
            .standard_events())
    }

    /// Check if the connected wallet supports `solana:signIn` feature
    pub async fn solana_signin(&self) -> WalletResult<bool> {
        Ok(self
            .connection_info()
            .await
            .connected_wallet()?
            .solana_signin())
    }

    /// Check if the connected wallet supports `solana:signMessage` feature
    pub async fn solana_sign_message(&self) -> WalletResult<bool> {
        Ok(self
            .connection_info()
            .await
            .connected_wallet()?
            .solana_sign_message())
    }

    /// Check if the connected wallet supports `solana:signAndSendTransaction` feature
    pub async fn solana_sign_and_send_transaction(&self) -> WalletResult<bool> {
        Ok(self
            .connection_info()
            .await
            .connected_wallet()?
            .solana_sign_and_send_transaction())
    }

    /// Check if the connected wallet supports `solana:signTransaction` feature
    pub async fn solana_sign_transaction(&self) -> WalletResult<bool> {
        Ok(self
            .connection_info()
            .await
            .connected_wallet()?
            .solana_sign_transaction())
    }
}

impl PartialEq for WalletAdapter {
    fn eq(&self, other: &Self) -> bool {
        self.window.eq(&other.window)
            && self.document.eq(&other.document)
            && self.storage.eq(&other.storage)
    }
}
impl Eq for WalletAdapter {}
