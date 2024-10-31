use std::future::Future;

use js_sys::{Function, Object, Reflect};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{CustomEvent, CustomEventInit};

use crate::{
    MessageSender, MessageType, Utils, Wallet, WalletAdapter, WalletError, WalletResult,
    WINDOW_APP_READY_EVENT_TYPE,
};

impl WalletAdapter {
    pub fn dispatch_app_event(&self, sender: MessageSender) {
        let app_ready_init = CustomEventInit::new();
        app_ready_init.set_bubbles(false);
        app_ready_init.set_cancelable(false);
        app_ready_init.set_composed(false);
        app_ready_init.set_detail(&Self::register_object(sender));

        let app_ready_ev =
            CustomEvent::new_with_event_init_dict(WINDOW_APP_READY_EVENT_TYPE, &app_ready_init)
                .unwrap();

        self.window().dispatch_event(&app_ready_ev).unwrap();
    }

    pub async fn dispatch_error_event(error: WalletError, sender: MessageSender) {
        if let Err(sender_error) = sender.send(MessageType::Failure(error)).await {
            panic!("Error `{sender_error:?}`. Unable to send message via channel. Maybe the `Receiver` was dropped closing the channel.")
        }
    }

    pub fn register_wallet_event(&self, sender: MessageSender) -> WalletResult<()> {
        let listener_closure = Closure::wrap(Box::new(move |custom_event: CustomEvent| {
            let sender1 = sender.clone();
            let sender2 = sender.clone();
            let sender3 = sender.clone();

            let detail = match custom_event
                .detail()
                .dyn_into::<Function>()
                .map_err(|error| {
                    let outcome: WalletError = error.into();
                    outcome
                }) {
                Ok(value) => value,
                Err(error) => {
                    Self::run_executor(Self::dispatch_error_event(error, sender1));

                    return;
                }
            };

            // Call the JavaScript function passed as `detail`, passing the `register_object`
            if let Err(error) = Utils::jsvalue_to_error(
                detail.call1(&JsValue::null(), &Self::register_object(sender2)),
            ) {
                Self::run_executor(Self::dispatch_error_event(error, sender3));
            }
        }) as Box<dyn Fn(_)>);

        let listener_fn = listener_closure
            .as_ref()
            .dyn_ref::<Function>()
            .ok_or(WalletError::CastClosureToFunction)?;

        self.window().add_event_listener_with_callback(
            crate::WINDOW_REGISTER_WALLET_EVENT_TYPE,
            listener_fn,
        )?;

        listener_closure.forget();

        Ok(())
    }

    fn run_executor(future_value: impl Future<Output = ()> + 'static) {
        wasm_bindgen_futures::spawn_local(async move { future_value.await });
    }

    pub fn register_object(sender: MessageSender) -> Object {
        let sender3 = sender.clone();

        // The `register` function that logs and returns a closure like in your JS code
        let register = Closure::wrap(Box::new(move |value: JsValue| {
            let sender1 = sender.clone();
            let sender2 = sender.clone();

            match Wallet::from_jsvalue(value) {
                Ok(wallet) => {
                    wasm_bindgen_futures::spawn_local(async move {
                        if let Err(sender_error) = sender1.send(MessageType::Success(wallet)).await
                        {
                            panic!("Unable to send Wallet details `{sender_error:?}` to the channel receiver. Maybe the receiver has been dropped.")
                        }
                    });
                }
                Err(error) => wasm_bindgen_futures::spawn_local(async move {
                    Self::dispatch_error_event(error, sender2).await
                }),
            }
        }) as Box<dyn Fn(_)>);

        // Create an object and set the `register` property
        let register_object = Object::new();

        if let Err(error) = Reflect::set(
            &register_object,
            &JsValue::from("register"),
            &register.into_js_value(),
        ) {
            Self::run_executor(Self::dispatch_error_event(error.into(), sender3))
        }

        register_object
    }
}
