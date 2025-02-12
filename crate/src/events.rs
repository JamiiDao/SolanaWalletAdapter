use std::rc::Rc;

use async_channel::{Receiver, Sender};
use web_sys::{
    js_sys::{Object, Reflect},
    wasm_bindgen::{prelude::Closure, JsValue},
    CustomEvent, CustomEventInit, Window,
};

use crate::{
    Reflection, StorageType, Utils, Wallet, WalletAccount, WalletAdapter, WalletError,
    WalletResult, WINDOW_APP_READY_EVENT_TYPE,
};

/// The `Sender` part of an [async_channel::bounded] channel
pub type WalletEventSender = Sender<WalletEvent>;

/// The `Receiver` part of an [async_channel::bounded] channel
pub type WalletEventReceiver = Receiver<WalletEvent>;

/// Used to initialize the `Register` and `AppReady` events to the browser window
#[derive(Debug, PartialEq, Eq)]
pub struct InitEvents<'a> {
    window: &'a Window,
}

impl<'a> InitEvents<'a> {
    /// Instantiate [Self](self)
    pub fn new(window: &'a Window) -> Self {
        Self { window }
    }

    /// Register events by providing a [WalletStorage] that is used to store
    /// all registered wallets
    pub fn init(&self, adapter: &mut WalletAdapter) -> WalletResult<()> {
        let storage = adapter.storage();
        self.register_wallet_event(storage.clone_inner())?;
        self.dispatch_app_event(storage.clone_inner());

        Ok(())
    }

    /// An App Ready event registered to the browser window
    pub fn dispatch_app_event(&self, storage: StorageType) {
        let app_ready_init = CustomEventInit::new();
        app_ready_init.set_bubbles(false);
        app_ready_init.set_cancelable(false);
        app_ready_init.set_composed(false);
        app_ready_init.set_detail(&Self::register_object(storage));

        let app_ready_ev =
            CustomEvent::new_with_event_init_dict(WINDOW_APP_READY_EVENT_TYPE, &app_ready_init)
                .unwrap();

        self.window.dispatch_event(&app_ready_ev).unwrap();
    }

    /// The register wallet event registered to the browser window
    pub fn register_wallet_event(&self, storage: StorageType) -> WalletResult<()> {
        let inner_storage = Rc::clone(&storage);

        let listener_closure = Closure::wrap(Box::new(move |custom_event: CustomEvent| {
            let detail = Reflection::new(custom_event
                .detail()).unwrap().as_function_owned()
                .expect("Unable to get the `detail` function from the `Event` object. This is a fatal error as the register handler won't execute.");

            Utils::jsvalue_to_error(detail.call1(
                &JsValue::null(),
                &Self::register_object(inner_storage.clone()),
            ))
            .unwrap()
        }) as Box<dyn Fn(_)>);

        let listener_fn = Reflection::new(listener_closure.into_js_value())
            .unwrap()
            .as_function_owned()
            .unwrap();

        self.window.add_event_listener_with_callback(
            crate::WINDOW_REGISTER_WALLET_EVENT_TYPE,
            &listener_fn,
        )?;

        Ok(())
    }

    pub(crate) fn register_object(storage: StorageType) -> Object {
        // The `register` function that logs and returns a closure like in your JS code
        let register =
            Closure::wrap(
                Box::new(move |value: JsValue| match Wallet::from_jsvalue(value) {
                    Ok(wallet) => {
                        let inner_outcome = storage.clone();

                        inner_outcome.borrow_mut().insert(
                            blake3::hash(wallet.name().to_lowercase().as_bytes()),
                            wallet,
                        );
                    }
                    Err(error) => {
                        let error = error.to_string();
                        if error.contains("is not supported") {
                        } else {
                            web_sys::console::error_2(
                                &"REGISTER EVENT ERROR".into(),
                                &error.into(),
                            );
                        }
                    }
                }) as Box<dyn Fn(_)>,
            );

        // Create an object and set the `register` property
        let register_object = Object::new();

        if let Err(error) = Reflect::set(
            &register_object,
            &JsValue::from("register"),
            &register.into_js_value(),
        ) {
            web_sys::console::error_2(&"REGISTER EVENT ERROR".into(), &error);
        }

        register_object
    }
}

/// Events emitted by connected browser extensions
/// when an account is connected, disconnected or changed.
/// Wallets implementing the wallet standard emit these events
/// from the `standard:events` events namespace specifically,
/// `wallet.features[standard:events].on`
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Clone)]
pub enum WalletEvent {
    /// An account has been connected and an event `change` emitted.
    Connected(WalletAccount),
    /// An account has been reconnected and an event `change` emitted.
    Reconnected(WalletAccount),
    /// An account has been disconnected and an event `change` emitted.
    Disconnected,
    /// An account has been connected and an event `change` emitted.
    /// The wallet adapter then updates the connected [WalletAccount].
    AccountChanged(WalletAccount),
    /// An error occured when a background task was executed.
    /// This type of event is encountered mostly from the
    /// `on` method from the `[standard:events]` namespace
    /// (when an account is connected, changed or disconnected)
    BackgroundTaskError(WalletError),
    /// An event was emitted by a wallet that is not connected.
    #[default]
    Skip,
}

impl core::fmt::Display for WalletEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let as_str = match self {
            Self::Connected(_) => "Connected",
            Self::Reconnected(_) => "Reconnected",
            Self::Disconnected => "Disconnected",
            Self::AccountChanged(_) => "Account Changed",
            Self::BackgroundTaskError(error) => &format!("Task error: {error:?}"),
            Self::Skip => "Skipped",
        };
        write!(f, "{}", as_str)
    }
}
