use async_channel::Sender;
use log::Level;
use wallet_adapter::{MessageType, WalletAdapter};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn main(connect_ev_node: &str) {
    console_error_panic_hook::set_once();
    console_log::init_with_level(Level::Trace).unwrap();

    log::info!("TARGET NODE: {}", connect_ev_node);

    let adapter = WalletAdapter::init().unwrap();

    adapter.execute(runner);
}

async fn runner(sender: Sender<MessageType>) {
    sender.send(MessageType::Connect("Solflare")).await.unwrap()
}
