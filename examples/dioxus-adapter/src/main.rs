#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_logger::tracing::Level;
use wallet_adapter::WalletAdapter;
use web_sys::wasm_bindgen::JsCast;

mod header;
use header::*;

mod signin;
use signin::*;

mod sign_message;
use sign_message::*;

mod sign_tx;
use sign_tx::*;

mod sign_and_send_tx;
use sign_and_send_tx::*;

fn main() {
    dioxus_logger::init(Level::INFO).expect("failed to init logger");

    launch(App);
}

#[component]
fn App() -> Element {
    let adapter = WalletAdapter::init().unwrap();
    let adapter = use_signal(move || adapter);
    let mut show_modal = use_signal(move || false);

    rsx! {
        link { rel: "stylesheet", href: "normalize.css" }
        link { rel: "stylesheet", href: "main.css" }

        if adapter.read().is_connected() {
            div {
                {Header(adapter)}
                div {id:"body-content",
                    {SignIn(adapter)}
                    {SignMessage(adapter)}
                    {SignTx(adapter)}
                    {SignAndSendTx(adapter)}
                }
            }
        }else {
            div{
                id:"disconnected-content",
                h1 {"Rust Wallet Adapter Demo"}
                div {"WALLET DISCONNECTED"}

                if *show_modal.read() {
                    {ShowModal(adapter, show_modal)}
                }

                button {
                    id:"btn-primary",
                    onclick: move |_| {
                    show_modal.set(true);
                }, "CONNECT WALLET" }
            }
        }
    }
}

pub fn ShowModal(mut adapter: Signal<WalletAdapter>, mut show_modal: Signal<bool>) -> Element {
    rsx! {
        div{
            id:"modal-container",
            div {
                id: "modal-content",
                for wallet in adapter.read().wallets() {
                    div {onclick: move |selected_wallet_event| {
                        spawn(async move {
                            let target = selected_wallet_event.data.downcast::<web_sys::MouseEvent>().unwrap();
                        let target = target.target().unwrap();
                        let target_as_element = target.dyn_ref::<web_sys::Element>().unwrap().closest(".wallet-list-entry").unwrap().unwrap();

                        let attribute_value = target_as_element.get_attribute("data-wallet-name").unwrap();
                            let target_wallet = adapter.read().get_wallet(attribute_value.as_str()).unwrap();
                            adapter.write().connect(target_wallet.name()).await.unwrap();
                            show_modal.set(false);

                        });
                        },
                        class:"wallet-list-entry", "data-wallet-name":wallet.name(),
                        span{
                            class:"wallet-icon",
                            img { src:wallet.icon().unwrap().to_string()}
                        }
                        span{
                            class:"wallet-name",
                            {wallet.name()}
                        }
                    }
                }
            }
        }
    }
}
