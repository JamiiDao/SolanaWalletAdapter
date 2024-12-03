use std::str::FromStr;

use dioxus::prelude::*;
use dioxus_logger::tracing::info;
use serde::Deserialize;
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL, pubkey::Pubkey, system_instruction, transaction::Transaction,
};
use wallet_adapter::{Cluster, SendOptions, Utils, WalletAdapter};
use wasm_bindgen_futures::JsFuture;
use web_sys::{wasm_bindgen::JsCast, Headers, Request, RequestInit, Response};

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
                                let output = String::from("https://explorer.solana.com/tx/") + Utils::base58_signature(signature).as_str() + "?cluster=devnet";
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

    let request = Request::new_with_str_and_init(devnet_uri, &opts).unwrap();

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
