use std::str::FromStr;

use serde::Deserialize;
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL, pubkey::Pubkey, system_instruction, transaction::Transaction,
};
use wallet_adapter::{Cluster, SendOptions, Utils};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Headers, Request, RequestInit, Response};
use yew::prelude::*;

use super::ConnectedAccounts;

#[function_component]
pub fn SignAndSendTxComponent(connection: &ConnectedAccounts) -> Html {
    let connected_account = connection.account.clone();
    let connected_wallet = connection.wallet.clone();

    let public_key = connected_account.public_key;
    let pubkey = Pubkey::new_from_array(public_key);
    let recipient_pubkey = Pubkey::new_from_array(Utils::public_key_rand());
    let sol = LAMPORTS_PER_SOL;

    let signed_tx_output: UseStateHandle<String> = use_state(String::default);
    let output = (*signed_tx_output.clone()).clone();

    html! {
        if signed_tx_output.is_empty() {
            <div class="inner-section">
                    <div class="inner-header"> {"SIGN AND SEND SOL TX"}</div>
                    <div class="inner-body"> {"FROM: "} {connected_account.address.as_str()}</div>
                    <div class="inner-body"> {"TO: "} {recipient_pubkey}</div>
                    <div class="inner-body"> {"LAMPORTS: "} {sol}</div>
                    <button class="btn-inner"
                        onclick={
                            Callback::from(move |_| {
                                let connected_wallet= connected_wallet.clone();
                                let connected_account= connected_account.clone();
                                let signed_tx_output= signed_tx_output.clone();

                                wasm_bindgen_futures::spawn_local(async move {
                                    let instr = system_instruction::transfer(&pubkey, &recipient_pubkey, sol);
                                    let mut tx = Transaction::new_with_payer(&[instr], Some(&pubkey));
                                    let blockhash = get_blockhash().await;
                                    tx.message.recent_blockhash = blockhash;
                                    let tx_bytes = bincode::serialize(&tx).unwrap();
                                    let signature = connected_wallet.sign_and_send_transaction(&tx_bytes, Cluster::DevNet, SendOptions::default(), &connected_account).await;
                                    let signature = signature.unwrap();
                                    let output = String::from("https://explorer.solana.com/tx/") + Utils::base58_signature(signature).as_str() + "?cluster=devnet";
                                    signed_tx_output.set(output);
                                });
                            })
                        }
                    >{"TRANSFER SOL TX"}
                    </button>
            </div>
        }else {
            <div class="inner-section">
                <div class="inner-header"> {"SIGNED SEND SOL TX"}</div>
                <div class="inner-body"> {"FROM: "} {connected_account.address.as_str()}</div>
                <div class="inner-body"> {"TO: "} {recipient_pubkey}</div>
                <div class="inner-body"> {"LAMPORTS: "} {sol}</div>
                <div class="inner-body">
                    <a href={output}>{signed_tx_output.clone().as_str()}</a>
                </div>
            </div>
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
