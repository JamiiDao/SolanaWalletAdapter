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

    pub fn execute(self) {
        wasm_bindgen_futures::spawn_local(async move {
            while let Ok(message_type) = self.receiver.recv().await {
                log::info!("WALLET_ADAPTER> : {:#?}", &message_type);
            }
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
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum MessageType {
    Success(Wallet),
    Failure(WalletError),
}
