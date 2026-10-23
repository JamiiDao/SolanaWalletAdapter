use std::str::FromStr;

use dioxus::prelude::*;
use serde::Deserialize;
use solana_pubkey::Pubkey;
use solana_transaction::Transaction;
use solana_transaction_error::TransactionError;
use wallet_adapter::{
    web_sys::{js_sys::Date, wasm_bindgen::JsValue},
    SendOptions, WalletError, WalletResult,
};

use crate::{
    views::AccountState, FetchReq, ACCOUNT_STATE, CLUSTER_STORAGE, LAMPORTS_PER_SOL, WALLET_ADAPTER,
};

pub fn format_timestamp(unix_timestamp: i64) -> String {
    let timestamp_ms = unix_timestamp as f64 * 1000.0; //Convert seconds to milliseconds

    let js_date = Date::new(&JsValue::from_f64(timestamp_ms));

    js_date
        .to_string()
        .as_string()
        .unwrap_or("Invalid Timestamp".to_string())
}

pub async fn get_blockhash() -> WalletResult<solana_hash::Hash> {
    let options = jzon::object! {
        "id":1,
        "jsonrpc":"2.0",
        "method":"getLatestBlockhash",
        "params":[

        ]
    }
    .to_string();

    // NOTE: You can use Reqwest crate instead to fetch the blockhash but
    // this code shows how to use the browser `fetch` api

    let response_body = FetchReq::new("POST")?
        .add_header("content-type", "application/json")?
        .add_header("Accept", "application/json")?
        .set_body(&options)
        .send()
        .await?;

    let deser = serde_json::from_str::<RpcResponse<ResponseWithContext<BlockHashResponseValue>>>(
        &response_body,
    )
    .unwrap();

    solana_hash::Hash::from_str(deser.result.value.blockhash)
        .map_err(|error| WalletError::Op(error.to_string()))
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockHashResponseValue<'a> {
    #[serde(borrow)]
    pub blockhash: &'a str,
    pub last_valid_block_height: u64,
}

pub async fn get_balance(address: &str) -> WalletResult<String> {
    let balance_options = jzon::object! {
        "id":1,
        "jsonrpc":"2.0",
        "method": "getBalance",
        "params": [
            address
        ]
    }
    .to_string();

    let balance_response = FetchReq::new_for_rpc()?
        .set_body(&balance_options)
        .send()
        .await?;

    let parsed_balance =
        serde_json::from_str::<RpcResponse<ResponseWithContext<u64>>>(&balance_response)
            .map_err(|error| WalletError::Op(error.to_string()))?;

    // WARNING: Do better financial math here
    Ok((parsed_balance.result.value as f64 / LAMPORTS_PER_SOL as f64).to_string())
}

pub async fn send_sol_req(
    recipient: &str,
    lamports: u64,
    public_key_bytes: [u8; 32],
) -> WalletResult<()> {
    let cluster = CLUSTER_STORAGE.read().active_cluster().cluster();

    let pubkey = Pubkey::new_from_array(public_key_bytes);
    let recipient = Pubkey::from_str(recipient).or(Err(WalletError::Op(
        "Invalid Recipient Address".to_string(),
    )))?;

    let send_sol_instruction =
        solana_system_interface::instruction::transfer(&pubkey, &recipient, lamports);
    let mut tx = Transaction::new_with_payer(&[send_sol_instruction], Some(&pubkey));
    let blockhash = get_blockhash().await?;

    tx.message.recent_blockhash = blockhash;
    let tx_bytes = bincode::serialize(&tx).map_err(|error| WalletError::Op(error.to_string()))?;

    WALLET_ADAPTER
        .read()
        .sign_and_send_transaction(&tx_bytes, cluster, SendOptions::default())
        .await?;

    Ok(())
}

pub async fn request_airdrop(lamports: u64, address: &str) -> WalletResult<()> {
    let options = jzon::object! {
        "id":1,
        "jsonrpc":"2.0",
        "method": "requestAirdrop",
        "params": [
            address,
            lamports
        ]
    }
    .to_string();

    let response = FetchReq::new_for_rpc()?.set_body(&options).send().await?;

    serde_json::from_str::<RpcResponse<String>>(&response)
        .map_err(|error| WalletError::Op(error.to_string()))?;

    Ok(())
}

pub async fn accounts_runner(address: &str) -> WalletResult<AccountState> {
    *ACCOUNT_STATE.write() = AccountState::default();

    let balance = crate::get_balance(&address).await?;

    let token_accounts_options = jzon::object! {
        "id":1,
        "jsonrpc":"2.0",
        "method": "getTokenAccountsByOwner",
        "params": [
            address,
            {
                "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
            },
            {
                "encoding": "jsonParsed"
            }
        ]
    }
    .to_string();

    let fetched_token_accounts = FetchReq::new_for_rpc()?
        .set_body(&token_accounts_options)
        .send()
        .await?;

    let parsed_token_accounts = serde_json::from_str::<
        RpcResponse<ResponseWithContext<Vec<TokenAccountResponse>>>,
    >(&fetched_token_accounts)
    .map_err(|error| WalletError::Op(error.to_string()))?;

    let token_accounts = parsed_token_accounts.result.value;

    let get_signatures_options = jzon::object! {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getSignaturesForAddress",
        "params": [
          address
        ]
    }
    .to_string();
    let fetched_signatures = FetchReq::new("POST")?
        .add_header("Content-Type", "application/json")?
        // .add_header("Accept", "application/json")?
        .set_body(&get_signatures_options)
        .send()
        .await?;

    let parsed_signatures_response =
        serde_json::from_str::<RpcResponse<Vec<SignaturesResponse>>>(&fetched_signatures)
            .map_err(|error| WalletError::Op(error.to_string()))?;

    let signatures = parsed_signatures_response.result;

    Ok(AccountState {
        balance,
        token_accounts,
        transactions: signatures.clone(),
    })
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcResponse<T> {
    pub jsonrpc: String,
    pub id: u8,
    pub result: T,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignaturesResponse {
    pub block_time: Option<i64>,
    pub confirmation_status: Option<String>,
    pub err: Option<TransactionError>,
    pub signature: String,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseWithContext<O> {
    pub value: O,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenAccountResponse {
    pub pubkey: String,
    pub account: Account,
}

impl TokenAccountResponse {
    pub fn mint(&self) -> String {
        self.account.data.parsed.info.mint.to_owned()
    }

    pub fn ata_address(&self) -> String {
        self.pubkey.to_owned()
    }

    pub fn balance(&self) -> String {
        self.account
            .data
            .parsed
            .info
            .token_amount
            .ui_amount_string
            .to_owned()
    }

    pub fn state(&self) -> String {
        self.account.data.parsed.info.state.to_uppercase()
    }
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub data: TokenData,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenAmount {
    pub amount: String,
    pub decimals: u8,
    pub ui_amount: f64,
    pub ui_amount_string: String,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParseInfo {
    pub mint: String,
    pub state: String,
    pub token_amount: TokenAmount,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Parsed {
    pub info: ParseInfo,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenData {
    pub parsed: Parsed,
}
