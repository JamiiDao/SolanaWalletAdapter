#![allow(non_snake_case)]

use std::str::FromStr;

use dioxus::prelude::*;
use dioxus_logger::tracing::{info, Level};
use serde::Deserialize;
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL, pubkey::Pubkey, system_instruction, transaction::Transaction,
};
use wallet_adapter::{
    Cluster, SendOptions, SignInOutput, SignedMessageOutput, SigninInput, Utils, WalletAdapter,
};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    wasm_bindgen::{prelude::Closure, JsCast},
    Headers, HtmlElement, Request, RequestInit, Response,
};

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
                {SignIn(adapter)}
                {SignMessage(adapter)}
                {SignTx(adapter)}
                {SignAndSendTx(adapter)}
            }
        }else {
            div{
                id:"disconnected",
                p {"DISCONNECTED"}

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

pub fn SignAndSendTx(adapter: Signal<WalletAdapter>) -> Element {
    let mut signed_tx_output: Signal<String> = use_signal(|| String::default());
    let public_key = adapter.read().connected_account().unwrap().public_key;
    let pubkey = Pubkey::new_from_array(public_key);
    let recipient_pubkey = Pubkey::new_from_array(Utils::public_key_rand());
    let sol = LAMPORTS_PER_SOL;

    rsx! {
        if signed_tx_output.read().is_empty() {
            div {class:"inner-section",
                    div {class:"inner-header", "SIGN AND SEND SOL TX"}
                    div {class:"inner-body", "FROM: {adapter.read().connected_account().unwrap().address.as_str()}"}
                    div {class:"inner-body", "TO: {recipient_pubkey}"}
                    div {class:"inner-body", "LAMPORTS: {sol}"}
                    button {
                        id:"btn-primary",
                        onclick: move |_| {
                            spawn(async move {
                                let instr = system_instruction::transfer(&pubkey, &recipient_pubkey, sol);
                                let mut tx = Transaction::new_with_payer(&[instr], Some(&pubkey));
                                let blockhash = get_blockhash().await;
                                tx.message.recent_blockhash = blockhash;
                                let tx_bytes = bincode::serialize(&tx).unwrap();
                                let signature = adapter.read().sign_and_send_transaction(&tx_bytes, Cluster::DevNet, SendOptions::default()).await;
                                info!("RAW: {:?}", &signature);
                                let signature = signature.unwrap();
                                let output = String::from("https://explorer.solana.com/tx/") + &Utils::base58_signature(signature).as_str() + "?cluster=devnet";
                                *signed_tx_output.write()=output;
                            });
                        },
                        "TRANSFER SOL TX"
                    }
            }
        }else {
            div {class:"inner-section",
                div {class:"inner-header", "SIGNED SEND SOL TX"}
                div {class:"inner-body", "FROM: {adapter.read().connected_account().unwrap().address.as_str()}"}
                div {class:"inner-body", "TO: {recipient_pubkey}"}
                div {class:"inner-body", "LAMPORTS: {sol}"}
                div {class:"inner-body",
                    a{
                        href:signed_tx_output.read().as_str(),
                        "{signed_tx_output.read().as_str()}"
                    }
                }
            }
        }
    }
}

async fn get_blockhash() -> solana_sdk::hash::Hash {
    let devnet_uri = Cluster::DevNet.endpoint();
    let body = jzon::object! {
        "id":1,
        "jsonrpc":"2.0",
        "method":"getLatestBlockhash",
        "params":[

        ]
    };

    // NOTE: You can use Reqwest crate instead to fetch the blockhash but
    // this code shows how to use the browser `fetch` api

    let headers = Headers::new().unwrap();
    headers.append("content-type", "application/json").unwrap();
    headers.append("Accept", "application/json").unwrap();

    let opts = RequestInit::new();
    opts.set_method("POST");
    opts.set_headers(&headers);
    opts.set_body(&body.to_string().as_str().into());

    let request = Request::new_with_str_and_init(&devnet_uri, &opts).unwrap();

    let window = web_sys::window().unwrap();
    let fetch_promise = window.fetch_with_request(&request);

    // Await the fetch promise to get a `Response` object
    let resp_value = JsFuture::from(fetch_promise).await.unwrap();
    let resp = resp_value.dyn_into::<Response>().unwrap();

    let body_as_str = JsFuture::from(resp.text().unwrap())
        .await
        .unwrap()
        .as_string()
        .unwrap();

    let deser = serde_json::from_str::<GetBlockHashResponse>(&body_as_str).unwrap();

    solana_sdk::hash::Hash::from_str(deser.result.value.blockhash).unwrap()
}

pub fn SignTx(adapter: Signal<WalletAdapter>) -> Element {
    let mut signed_tx_output: Signal<Option<Transaction>> = use_signal(|| Option::default());
    let public_key = adapter.read().connected_account().unwrap().public_key;
    let pubkey = Pubkey::new_from_array(public_key);
    let recipient_pubkey = Pubkey::new_from_array(Utils::public_key_rand());
    let sol = LAMPORTS_PER_SOL;

    rsx! {
        if signed_tx_output.read().is_none() {
            div {class:"inner-section",
                    div {class:"inner-header", "SEND SOL TX"}
                    div {class:"inner-body", "FROM: {adapter.read().connected_account().unwrap().address.as_str()}"}
                    div {class:"inner-body", "TO: {recipient_pubkey}"}
                    div {class:"inner-body", "LAMPORTS: {sol}"}
                    button {
                        id:"btn-primary",
                        onclick: move |_| {
                            spawn(async move {
                                let instr = system_instruction::transfer(&pubkey, &recipient_pubkey, sol);
                                let tx = Transaction::new_with_payer(&[instr], Some(&pubkey));
                                let tx_bytes = bincode::serialize(&tx).unwrap();
                                let output = adapter.read().sign_transaction(&tx_bytes, Some(Cluster::DevNet)).await.unwrap();
                                let deser_tx_output = bincode::deserialize::<Transaction>(&output[0]).unwrap();
                                signed_tx_output.write().replace(deser_tx_output);
                            });
                        },
                        "SIGN TX"
                    }
            }
        }else {
            div {class:"inner-section",
                div {class:"inner-header", "SIGNED SEND SOL TX"}
                div {class:"inner-body", "FROM: {adapter.read().connected_account().unwrap().address.as_str()}"}
                div {class:"inner-body", "TO: {recipient_pubkey}"}
                div {class:"inner-body", "LAMPORTS: {sol}"}
            }
        }
    }
}

pub fn SignMessage(adapter: Signal<WalletAdapter>) -> Element {
    let mut signed_message_output: Signal<Option<SignedMessageOutput>> =
        use_signal(|| Option::None);
    let message = "Using Dioxus Framework";
    let sign_message_supported = adapter
        .read()
        .connected_wallet()
        .unwrap()
        .solana_sign_message();
    rsx! {
        div {
            if signed_message_output.read().is_none() {
                div {class:"inner-section",
                    div {class:"inner-header", "MESSAGE TO SIGN"}
                    div {class:"inner-body", "MESSAGE: {message}"}

                    if sign_message_supported {
                        button{
                            id:"btn-primary",
                            onclick: move |_| {
                                spawn(async move {
                                    let output = adapter.read().sign_message(message.as_bytes()).await.unwrap();
                                    signed_message_output.write().replace(output);
                                });
                            },
                            "SIGN MESSAGE"
                        }
                    }else {
                        button{
                            id:"btn-primary-disabled",
                            "SIGN MESSAGE UNSUPPORTED"
                        }
                    }
                }
            }else {
                div {class:"inner-section",
                    div {class:"inner-header", "MESSAGE SIGNED"}
                    div {class:"inner-body", "ADDRESS: {signed_message_output.read().unwrap().address().unwrap()}"}
                    div {class:"inner-body", "MESSAGE: {signed_message_output.read().unwrap().message()}"}
                    div {class:"inner-body", "SIGNATURE: {signed_message_output.read().unwrap().base58_signature().unwrap()}"}
                }
            }
        }
    }
}

pub fn SignIn(adapter: Signal<WalletAdapter>) -> Element {
    let mut signin_output: Signal<Option<SignInOutput>> = use_signal(|| Option::None);
    let message = "DIOXUS LOGIN";
    let signin_supported = adapter.read().solana_signin().unwrap();

    rsx! {
        if signin_output.read().is_none() {
            div {class:"inner-section",
                div {class:"inner-header", "SIGN IN DETAILS"},
                div {class:"inner-body", "ADDRESS: {adapter.read().connected_account().unwrap().address}"}
                div {class:"inner-body", "MESSAGE: {message}"}
                if signin_supported {
                    button {class:"btn-primary",
                        onclick: move |_| {
                        let mut signin_input = SigninInput::new();
                        signin_input
                            .set_domain(&adapter.read().window())
                            .unwrap()
                            .set_statement(message)
                            .set_chain_id(wallet_adapter::Cluster::DevNet)
                            // NOTE: Some wallets require this field or the wallet adapter
                            // will return an error `MessageReponseMismatch` which is as
                            // a result of the sent message not corresponding with the signed message
                            .set_address(&adapter.read().connected_account().unwrap().address)
                            .unwrap();

                        let public_key = adapter.read().connected_account().unwrap().public_key;

                        spawn(async move {
                            let outcome = adapter.read().sign_in(&signin_input, public_key).await.unwrap();
                            signin_output.set(Some(outcome));
                        });
                    },

                    "SIGN IN"
                    }
                }else {
                    button {
                        class:"btn-primary-disabled",
                        "SIWS Unsupported"
                    }
                }
            }
        }else {
            div {class:"inner-section",
                div {class:"inner-header", "SIGNED DETAILS"}
                div {class:"inner-body","ACCOUNT  : {signin_output.read().as_ref().unwrap().address()}"}
                div {class:"inner-body","MESSAGE  : {signin_output.read().as_ref().unwrap().message.as_str()}"}
                p {class:"inner-body", "SIGNATURE: {signin_output.read().as_ref().unwrap().signature()}"}
            }
        }
    }
}

pub fn Header(mut adapter: Signal<WalletAdapter>) -> Element {
    rsx! {
            div{id:"header",
                span {"DIOXUS DEMO"}
                span {
                    class:"menu-item",
                    a {id:"address", href:"#",
                        "data-wallet-address":adapter.read().connected_account().unwrap().address.as_str(),
                        "{adapter.read().connected_account().unwrap().shorten_address().unwrap()}"
                    }
                    ul{ class:"dropdown",
                        li{class:"dropdown-entry", a {
                            onclick: move |_| {
                                let address_element = adapter.read().document().get_element_by_id("address").unwrap();
                                let text_to_copy = address_element.get_attribute("data-wallet-address").unwrap();
                                let address_menu = adapter.read().document().get_element_by_id("copy-address").unwrap();
                                address_menu.set_text_content(Some("Copied"));
                                spawn(async move{
                                let writer_promise = adapter.read().window().navigator().clipboard().write_text(&text_to_copy);
                                wasm_bindgen_futures::JsFuture::from(writer_promise).await.unwrap();
                                let set_timeout = wasm_bindgen_futures::wasm_bindgen::closure::Closure::wrap(Box::new(move |_: web_sys::Event| {
                                    address_menu.set_text_content(Some("Copy Address"));
                                }) as Box<dyn Fn(_)>);

                                use wasm_bindgen_futures::wasm_bindgen::JsCast;
                                let set_timeout_fn = set_timeout.as_ref().dyn_ref::<wasm_bindgen_futures::js_sys::Function>().unwrap();
                                adapter.read().window()
                                    .set_timeout_with_callback_and_timeout_and_arguments_0(
                                        set_timeout_fn,
                                        1000,
                                    )
                                    .expect("failed to set timeout");
                                    set_timeout.forget();
                                });
                            },
                        id:"copy-address",
                        href:"#",
                        "Copy Address"
                    }}
                    // li{class:"dropdown-entry", a {
                    //     onclick: move |_| {
                    //         show_modal.set(true);
                    //     },
                    //     href:"#",
                    //     "Change Wallet"
                    // }}
                    li{class:"dropdown-entry", a {
                        onclick: move |_| {
                            spawn(async move {
                                adapter.write().disconnect().await.unwrap();
                            });
                        },
                        href:"#",
                        "Disconnect"
                    }}
                }
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
                            adapter.write().connect(&target_wallet.name()).await.unwrap();
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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetBlockHashResponse<'a> {
    #[serde(borrow)]
    pub jsonrpc: &'a str,
    pub id: u8,
    pub result: ResponseResult<'a>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseResult<'a> {
    #[serde(borrow)]
    pub context: Context<'a>,
    #[serde(borrow)]
    pub value: ResponseValue<'a>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Context<'a> {
    #[serde(borrow)]
    pub api_version: &'a str,
    pub slot: u64,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseValue<'a> {
    #[serde(borrow)]
    pub blockhash: &'a str,
    pub last_valid_block_height: u64,
}
