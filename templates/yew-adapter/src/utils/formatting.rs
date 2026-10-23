use wallet_adapter::web_sys::{js_sys::Date, wasm_bindgen::JsValue};
use yew::prelude::*;

use super::AdapterCluster;

pub fn trunk_cluster_name(name: &str) -> String {
    if name.len() > 10 {
        name.chars().take(10).collect::<String>() + "..."
    } else {
        name.to_string()
    }
}

pub fn format_timestamp(unix_timestamp: i64) -> String {
    let timestamp_ms = unix_timestamp as f64 * 1000.0; //Convert seconds to milliseconds

    let js_date = Date::new(&JsValue::from_f64(timestamp_ms));

    js_date
        .to_string()
        .as_string()
        .unwrap_or("Invalid Timestamp".to_string())
}

const EXPLORER: &str = "https://explorer.solana.com/";

pub fn format_address_url(address: &str, active_cluster: &AdapterCluster) -> String {
    String::new() + EXPLORER + "address/" + address + &adapter_query_string(active_cluster)
}

pub fn format_tx_url(tx: &str, active_cluster: &AdapterCluster) -> String {
    String::new() + EXPLORER + "tx/" + tx + &adapter_query_string(active_cluster)
}

pub fn adapter_query_string(active_cluster: &AdapterCluster) -> String {
    active_cluster.query_string().to_owned()
}

pub(crate) fn link_target_blank(href: String, text: String) -> Html {
    html! {<a class="underline" href={href}  target="_blank"  rel="noopener noreferrer"> {text}{"⇗"}</a>}
}
