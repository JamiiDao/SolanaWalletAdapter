#![forbid(unsafe_code)]
#![forbid(missing_docs)]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/JamiiDao/SolanaWalletAdapter/f005b3a555013e2cbfee8825d531b014c180d698/icon.svg"
)]
#![doc = include_str!(concat!("../", std::env!("CARGO_PKG_README")))]

mod adapter;
pub use adapter::*;

mod errors;
pub use errors::*;

mod commitment;
pub use commitment::*;

mod utils;
pub(crate) use utils::*;

mod events;
pub use events::*;

mod wallet_ser_der;
pub use wallet_ser_der::*;

mod storage;
pub use storage::*;

// Re-export common crate
pub use wallet_adapter_common::chains::*;
pub use wallet_adapter_common::clusters::*;
pub use wallet_adapter_common::feature_support::*;
pub use wallet_adapter_common::standardized_events::*;
pub use wallet_adapter_common::WalletCommonUtils as Utils;

// Re-export of crates
pub use async_channel;
pub use blake3;
pub use bs58;
pub use ed25519_dalek;
pub use getrandom;
pub use humantime;
pub use rand_chacha;
pub use rand_core;
pub use thiserror;
pub use wasm_bindgen_futures;
pub use web_sys;
