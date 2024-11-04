use std::future::Future;

use async_channel::{Receiver, Sender};
use web_sys::{js_sys::Object, Document, Window};

use crate::{Wallet, WalletError, WalletResult};

pub type MessageSender = Sender<MessageType>;

/// Operations on a browser window.
/// `Window` and `Document` object must be present otherwise
/// an error is thrown.
#[derive(Debug, Clone)]
pub struct WalletAdapter {
    window: Window,
    document: Document,
    wallets: Vec<Wallet>,
    sender: MessageSender,
    receiver: Receiver<MessageType>,
}

impl WalletAdapter {
    /// Get the `Window` and `Document` object in the current browser window
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

        let (sender, receiver) = async_channel::unbounded::<MessageType>();

        // let sender = Arc::new(sender);

        let new_self = Self {
            window,
            document,
            wallets: Vec::default(),
            sender,
            receiver,
        };

        new_self.init_events()?;

        Ok(new_self)
    }

    pub fn execute<F>(mut self, runner: impl FnOnce(Sender<MessageType>) -> F + 'static)
    where
        F: Future<Output = ()> + 'static,
    {
        let first_sender = self.sender.clone();

        let listener = async move {
            while let Ok(message_type) = self.receiver.recv().await {
                match message_type {
                    MessageType::Success(wallet) => {
                        // log::info!("WALLET_ADAPTER> [SUCCESS]: {:#?}", &wallet);
                        self.wallets.push(wallet);
                    }
                    MessageType::Failure(error) => {
                        log::info!("WALLET_ADAPTER> [ERROR]: {:#?}", &error);
                    }
                    MessageType::Connect(name) => {
                        connect(first_sender.clone(), &self.wallets, name).await
                    }
                }
            }
        };

        wasm_bindgen_futures::spawn_local(async move {
            let local_sender = self.sender.clone();

            futures_lite::future::zip(listener, runner(local_sender)).await;
        });
    }

    fn init_events(&self) -> WalletResult<()> {
        self.register_wallet_event(self.sender.clone())?;
        self.dispatch_app_event(self.sender.clone());

        Ok(())
    }

    /// Get an entry in the `Window` object
    pub fn get_entry(&self, property: &str) -> Option<Object> {
        self.window.get(property)
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn document(&self) -> &Document {
        &self.document
    }

    pub fn wallets(&self) -> &[Wallet] {
        self.wallets.as_slice()
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum MessageType {
    Success(Wallet),
    Failure(WalletError),
    Connect(&'static str),
}

async fn connect(sender: Sender<MessageType>, wallets: &[Wallet], name: &str) {
    if let Some(solflare) = wallets.iter().find(|wallet| wallet.name() == name) {
        match solflare.features().connect().await {
            Ok(connection) => {
                log::info!("CONNECT OUTCOME: {:?}", connection);

                match solflare.features().disconnect().await {
                    Ok(_) => log::info!("DISCONNECTED SUCCESSFULLY"),
                    Err(error) => log::info!("WALLET DISCONNECT ERROR: {:?}", error),
                }
            }

            Err(error) => {
                if let Some(error) = sender.send(MessageType::Failure(error)).await.err() {
                    log::error!("Unable to send error message. Maybe `Receiver` already closed the channel. Sender Error `{:?}`", error);
                }
            }
        }
    } else {
        panic!("{name} Not Found")
    }
}
