use std::{future::Future, pin::Pin};

use async_channel::Receiver;
use web_sys::wasm_bindgen::{prelude::Closure, JsValue};

use crate::{
    ConnectionInfoInner, Reflection, SemverVersion, StandardFunction, WalletAccount, WalletError,
    WalletEvent, WalletEventSender, WalletResult,
};

/// `standard:events` struct containing the `version` and `callback`
/// within the [StandardFunction] field
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StandardEvents(StandardFunction);

impl StandardEvents {
    /// parse the callback for `standard:events` from the [JsValue]
    pub(crate) fn new(reflection: &Reflection, version: SemverVersion) -> WalletResult<Self> {
        let get_standard_event_fn = reflection.get_function("on")?;

        Ok(Self(StandardFunction {
            version,
            callback: get_standard_event_fn,
        }))
    }

    pub(crate) async fn call_on_event(
        &self,
        connection_info: ConnectionInfoInner,
        wallet_name: String,
        sender: WalletEventSender,
        stop_signal: Receiver<()>,
    ) -> WalletResult<()> {
        let sender2 = sender.clone();

        let on_account_change = Closure::wrap(Box::new(move |value: JsValue| {
            let wallet_name = wallet_name.clone();
            web_sys::console::log_3(
                &"CALLED ON EV for ".into(),
                &wallet_name.clone().into(),
                &value,
            );

            let connection_info_inner = connection_info.clone();
            let sender_inner = sender2.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let reflect_accounts =
                    send_wallet_event_error()(Reflection::new(value), sender_inner.clone())
                        .await
                        .unwrap(); // Never fails
                let mut get_accounts = send_wallet_event_error()(
                    reflect_accounts.reflect_js_array("accounts"),
                    sender_inner.clone(),
                )
                .await
                .unwrap()
                .to_vec(); // Never fails

                let processed_wallet_account = if !get_accounts.is_empty() {
                    let first_account = send_wallet_event_error()(
                        Reflection::new(get_accounts.remove(0)),
                        sender_inner.clone(),
                    )
                    .await
                    .unwrap(); // Never fails

                    let account_processing = send_wallet_event_error()(
                        WalletAccount::parse(first_account),
                        sender_inner.clone(),
                    )
                    .await
                    .unwrap(); //Never fails
                    web_sys::console::error_2(
                        &"PRE ACCOUNT PROCESSING".into(),
                        &format!("{account_processing:?}").into(),
                    );

                    Some(account_processing)
                } else {
                    Option::None
                };

                connection_info_inner
                    .borrow_mut()
                    .emit_wallet_event(&wallet_name, processed_wallet_account, sender_inner.clone())
                    .await
            });
        }) as Box<dyn Fn(_)>);

        let on_account_change_fn =
            Reflection::new(on_account_change.into_js_value())?.as_function_owned()?;

        let on_event_fn = self.0.callback.clone();

        wasm_bindgen_futures::spawn_local(async move {
            while let Ok(_) = stop_signal.recv().await {
                let invoke_outcome = on_event_fn
                    .call2(
                        &JsValue::null(),
                        &"change".into(),
                        &on_account_change_fn.clone().into(),
                    )
                    .map_err(|error| {
                        let into_error: WalletError = error.into();

                        into_error
                    });

                send_wallet_event_error()(invoke_outcome, sender.clone())
                    .await
                    .unwrap();
            }
        });

        Ok(())
    }
}

pub(crate) async fn send_wallet_event(wallet_event: WalletEvent, sender: WalletEventSender) {
    if let Err(error) = sender.clone().send(wallet_event).await {
        web_sys::console::log_2(
            &"BACKGROUND TASK ERROR: [standard:events]on() > ".into(),
            &format!("{error:?}").into(),
        );
    }
}

pub(crate) fn send_wallet_event_error<T>(
) -> impl Fn(WalletResult<T>, WalletEventSender) -> Pin<Box<dyn Future<Output = Result<T, ()>>>> + 'static
where
    T: core::fmt::Debug + 'static,
{
    move |outcome: WalletResult<T>, sender: WalletEventSender| {
        Box::pin(async move {
            match outcome {
                Ok(value) => Ok(value),
                Err(error) => {
                    web_sys::console::log_2(
                        &"BACKGROUND TASK ERROR: [standard:events]on() > ".into(),
                        &format!("{error:?}").into(),
                    );

                    if let Err(channel_error) = sender
                        .send(WalletEvent::BackgroundTaskError(error.clone()))
                        .await
                    {
                        web_sys::console::log_2(
                            &"Encountered error while sending a wallet event: ".into(),
                            &format!("{channel_error:?}").into(),
                        );
                    }

                    Err(())
                }
            }
        })
    }
}
