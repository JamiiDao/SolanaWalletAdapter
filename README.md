# Wallet-Adapter
<img src="https://raw.githubusercontent.com/JamiiDao/SolanaWalletAdapter/refs/heads/master/LOGO.svg" alt="Solana Rust Wallet-Adapter Logo" width="50%">

[![crates.io](https://img.shields.io/crates/v/wallet-adapter.svg)](https://crates.io/crates/wallet-adapter)[![Docs](https://docs.rs/wallet-adapter/badge.svg)](https://docs.rs/wallet-adapter)[![Rust](https://github.com/JamiiDao/SolanaWalletAdapter/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/JamiiDao/SolanaWalletAdapter/actions/workflows/rust.yml)![License](https://img.shields.io/crates/l/wallet-adapter)![Passively Maintained](https://img.shields.io/badge/status-passively%20maintained-cyan)

A lightweight Rust Solana Wallet that can be used in Rust based frontends and WebAssembly.

This project was supported by the Solana Foundation.

Read the book at: [https://jamiidao.github.io/SolanaWalletAdapter/](https://jamiidao.github.io/SolanaWalletAdapter/)

### Documentation Links
- [Usage](#usage) - How to add this library and required features for `web-sys` crate
- [Initializing](#initializing-register-and-appready) - How `AppReady` and `Register` wallet events are initialized
- [Wallet Storage](#in-memory-storage-for-registered-wallets) - How the wallets registered are stored in memory within the dapp
- [Connect and Check for Supported Features](#connecting-to-a-browser-extension-wallet-and-checking-for-features) - How to connect to a browser wallet and check which features the connected wallet supports
- [Disconnect](#disconnecting-from-the-wallet) - Disconnected an account from the connected wallet
- [Sign In With Solana](#sign-in-with-solana-siws) - Sign In With Solana (SIWS)
- [Sign Message](#sign-message) - Signing a message with a browser wallet
- [Sign Transaction](#sign-transaction) - Signing a transaction with a browser wallet
- [Sign and Send Transaction](#sign-and-send-transaction) - Sign and Send Transaction with a browser wallet
- [License](#license) - Licensed under Apache-2.0 or MIT
- [Features](#features) - What features of the wallet standard are supported by this library
- [Templates](#templates) - Which Rust frontend framework templates have been implemented

### Usage
Building the project requires a web-assembly environment.

See [Template Usage](#template-usage) for more details


### Initializing `Register` and `AppReady`
This is done automatically when calling `WalletAdapter::init()`. The `Register` and `AppReady` events are registered to the browser window and document in the current page allowing browser extension wallet to register themselves as specified in the wallet standard.
```rust
use wallet_adapter::{WalletAdapter, WalletResult};

async fn foo() -> WalletResult<()>{
    // Initializing the wallet adapter with default channel capacity of `5`.
    // This library uses async-channel in order to listen for background tasks like
    // connect, disconnect and re-connect
    let adapter = WalletAdapter::init()?;
    
    // Initializing the wallet adapter with a custom channel
    let adapter = WalletAdapter::init_with_channel_capacity(
        10, // Custom capacity
    )?;
    
    // Initializing the wallet adapter with a
    // `Window` and `Document` that might have been initialized
    // elsewhere.
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let adapter = WalletAdapter::init_custom(
        window, // web_sys::Window,
        document, // web_sys::Document
    )?;
    
    // Initializing the wallet adapter with a custom channel,
    // and `Window` and `Document` that might have been initialized
    // elsewhere.
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let adapter = WalletAdapter::init_with_channel_capacity_window_and_document(
        10, // Custom capacity
        window, // web_sys::Window,
        document, // web_sys::Document
    )?;

    // Get the event receiver that you can use to listen
    // for `Register`, `Disconnected` and other wallet events.
    if let Ok(wallet_event) = adapter.events().recv().await {
        // Do something with the wallet event
    }

    // Get wallets registered
    adapter.wallets();

    // Get a wallet by name
    adapter.get_wallet("Phantom");

    // Get connection information.
    // Returns a `ConnectionInfo` struct which contains the
    // connected wallet and connected account. It is wrapped in an
    // async `RwLock` so you can safely pass it around in background
    // browser tasks
    adapter.connection_info();

    // Get the storage where the registered wallets are stored
    adapter.storage();

    // Expose the browser window
    adapter.window();
    // Expose the browser document
    adapter.document();

	Ok(())
}
```

#### In-memory storage for registered wallets.
`wallet_adapter::WalletStorage` handles storage of registered wallets. The in-memory storage is a `HashMap<hash, Wallet>`
where the `hash` is the hash of the wallet name.
```rust
use wallet_adapter::WalletStorage;

    let storage = WalletStorage::default();

    // Get all registered wallets
    storage.get_wallets();

    // Get a wallet by its name
    storage.get_wallet("Phantom");

    // Clone the storage inside a closure, method or function that moves variables out of their environment
    // `WalletStorage` internally representation is `Rc<RefCell<HashMap<hash, Wallet>>>`
    // this makes it cheap to clone `WalletStorage` where one needs to access `HashMap<hash, Wallet>`
    storage.clone_inner();
```

### Connecting to a browser extension wallet and checking for features
```rust
use wallet_adapter::{WalletAdapter, WalletResult};

async fn foo() -> WalletResult<()> {

    let mut adapter = WalletAdapter::init()?;

    // Lookup a wallet from the registered wallet by it's name
    // and then use that wallet entry to perform a connection request
    adapter.connect_by_name("Phantom").await?;

    // Assuming in a frontend application you have a list of 
    // registered wallets to iterate through in order to show a modal
    // or a dropdown of registered wallets and you are building an
    // onclick event to do a connection, you cal use the `WalletAdapter::connect()` method instead

    for wallet in adapter.wallets() {
        // An onclick event from any frontend framework
        // onclick:move|_| {
            adapter.connect(wallet).await?; // The `wallet` can be used inside the onclick event
        // }
    }

    // Get all clusters supported by a connected wallet
    adapter.clusters().await?;

    // Is MainNet cluster supported
    adapter.mainnet().await?;

    // Is DevNet cluster supported
    adapter.devnet().await?;

    // Is TestNet cluster supported
    adapter.testnet().await?;

    // Is LocalNet cluster supported
    adapter.localnet().await?;

    // Is `standard:connect` feature specified in wallet standard supported
    adapter.standard_connect().await?;

    // Is `standard:disconnect` feature specified in wallet standard supported
    adapter.standard_disconnect().await;

    // Is `standard:events` feature specified in wallet standard supported
    adapter.standard_events().await?;

    // Is `solana:signIn` feature specified in wallet standard supported
    adapter.solana_signin().await?;

    // Is `solana:signMessage` feature specified in wallet standard supported
    adapter.solana_sign_message().await?;

    // Is `solana:signTransaction` feature specified in wallet standard supported
    adapter.solana_sign_transaction().await?;

    // Is `solana:signAndSendTransaction` feature specified in wallet standard supported
    adapter.solana_sign_and_send_transaction().await?;

    Ok(())
}
```

### Disconnecting from the wallet
```rust
use wallet_adapter::{WalletAdapter, WalletResult, SigninInput};

async fn foo() -> WalletResult<()> {
    let mut adapter = WalletAdapter::init()?;
    adapter.connect_by_name("Phantom").await?;

    // Disconnect from the wallet
    adapter.disconnect().await;

    Ok(())
}
```

### Sign In With Solana (SIWS)
```rust
use wallet_adapter::{WalletAdapter, WalletResult, SigninInput};

async fn foo() -> WalletResult<()> {
    let mut adapter = WalletAdapter::init()?;
    adapter.connect_by_name("Phantom").await?;

    // The message to show the user
    let statement = "Login To Dev Website";

    // Get the public key bytes of the connected account within the connected wallet
    let public_key = adapter.connection_info().await.connected_account()?.public_key();
    // Get the address of the connected account within the connected wallet
    let address = adapter.connection_info().await.connected_account()?.address().to_string();

    let mut signin_input = SigninInput::new();
    signin_input
        .set_domain(&adapter.window())?
        .set_statement(statement)
        .set_chain_id(wallet_adapter::Cluster::DevNet)
        // NOTE: Some wallets require this field or the wallet adapter
        // will return an error `MessageResponseMismatch` which is as
        // a result of the sent message not corresponding with the signed message
        .set_address(&address)?;

    // Get the public key in bytes of the connected 
    let signin_output = adapter.sign_in(&signin_input, public_key).await.unwrap();

    Ok(())
}
```
Sign In With Solana (SIWS) supports more options for the Sign In With Solana Standard. Check the methods on the [SigninInput] struct.
**NOTE** that an error is thrown by the library in case the message signed, public key don't match or if the signature is not valid for the signing public key.

### Sign Message
All messages must be UTF-8 encoded string of bytes
```rust
use wallet_adapter::{WalletAdapter, WalletResult, SigninInput};

async fn foo() -> WalletResult<()> {
    let mut adapter = WalletAdapter::init()?;
    adapter.connect_by_name("Phantom").await?;
    // Check if the wallet supports signing a message
    if adapter.solana_sign_message().await? {
        adapter.sign_message(b"SOLANA ROCKS!!!").await?;
    }else {
        // Tell user Sign message is not supported
    }
    Ok(())
}
```
**NOTE** that an error is thrown by the library in case the message, public key don't match or if the signature is not valid for the signing public key.
### Sign Transaction
Here, we simulate signing a SOL transfer instruction
```rust
use wallet_adapter::{WalletAdapter, WalletResult, Cluster, Utils,};
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL, pubkey::Pubkey, system_instruction, transaction::Transaction,
};

async fn foo() -> WalletResult<()> {
    let mut adapter = WalletAdapter::init()?;
    adapter.connect_by_name("Phantom").await?;

    // Construct a transaction in a manner that the browser wallet extension
    // can deserialize the transaction from bytes.
    // Here we will use `solana-sdk` crate since it can be converted to
    // bytes using a crate like `bincode` that understands serializing
    // and deserializing the transaction to and from bytes.
    //
    // Get the public key bytes from the connected account
    let public_key = adapter.connection_info().await.connected_account()?.public_key();

    // Convert the public key bytes of the sender to a `solana_sdk::pubkey::Pubkey`
    let pubkey = Pubkey::new_from_array(public_key);

    // Convert the public key bytes of the recipient to a `solana_sdk::pubkey::Pubkey`.
    // Here we use `wallet_adapter::Utils::public_key_rand()` to generate unique public key bytes
    // for testing. Make sure you use a valid public key with corresponding private key
    // or your funds will be lost.
    let recipient_pubkey = Pubkey::new_from_array(Utils::public_key_rand());

    // How many SOL to send.
    // The `solana_sdk::native_token::LAMPORTS_PER_SOL` constant contains the number of lamports
    // equal to `1 SOL` so calculating `2 SOL` can be achieved using `2 * LAMPORTS_PER_SOL`
    let sol = LAMPORTS_PER_SOL;

    // Create an instruction to transfer the SOL
    let instr = system_instruction::transfer(&pubkey, &recipient_pubkey, sol);
    // Create a new unsigned transaction
    let tx = Transaction::new_with_payer(&[instr], Some(&pubkey));
    // Serialize the transaction into bytes using `bincode`
    let tx_bytes = bincode::serialize(&tx).unwrap();

    // Specify to use devnet cluster
    let cluster = Cluster::DevNet;

    // You can check if a wallet is connected first to display
    // a certain view to a user or make a user connect first if the account was disconnected
    if adapter.is_connected().await {
        // Request the browser wallet to sign the transaction.
        let output = adapter.sign_transaction(&tx_bytes, Some(cluster)).await?;

        // Deserialize the signed transaction bytes back into a transaction
        let deser_tx_output = bincode::deserialize::<Transaction>(&output[0]).unwrap();
    }

    Ok(())
}

```
Remember to add the necessary dependencies for this part in the `Cargo.toml` manifest.
```toml
[dependencies]
# Add latest versions of these crates
solana-sdk = "^2.1.2"
bincode = "^1.3.3"
```
**NOTE** that if the signed transaction is verified by the library and an error is thrown in case of signature mismatch.

### Sign And Send Transaction
Here, we simulate signing and sending a SOL transfer instruction
```rust
use std::str::FromStr;

use wallet_adapter::{WalletAdapter, WalletResult, Cluster, Utils, SendOptions};
use serde::Deserialize;
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL, pubkey::Pubkey, system_instruction, transaction::Transaction,
};
use wasm_bindgen_futures::JsFuture;
use web_sys::{wasm_bindgen::JsCast, Headers, Request, RequestInit, Response};

async fn foo() -> WalletResult<()> {
    let mut adapter = WalletAdapter::init()?;
    adapter.connect_by_name("Phantom").await?;

    // The variables for the code is the same as the one for Sign Transaction

    let public_key = adapter.connection_info().await.connected_account()?.public_key();
    let pubkey = Pubkey::new_from_array(public_key);
    let recipient_pubkey = Pubkey::new_from_array(Utils::public_key_rand());
    let sol = LAMPORTS_PER_SOL;
    let instr = system_instruction::transfer(&pubkey, &recipient_pubkey, sol);

    // This part is different from Sign Transaction above since we need a valid recent blockhash
    // as part of the `SendAndSignTransaction` specification.


    // First let's construct structs from serde that will allow us to deserialize the 
    // response of the recent blockhash from a Solana cluster

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

    // Create code to use browser fetch API to request a recent blockhash from the
    // Solana cluster, in our case Devnet cluster.
    // NOTE: You can use other crates like `reqwest` and `gloo-net` do this. However,
    // this example will use browser fetch API to give an example of how you would use
    // Fetch API or when you don't want to add external dependencies to do this
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

    // Create a new mutable unsigned transaction
    let mut tx = Transaction::new_with_payer(&[instr], Some(&pubkey));

    // You can check if a wallet is connected first to display
    // a certain view to a user or make a user connect first if the account was disconnected
    if adapter.is_connected().await {
        // Get the blockhash
        let blockhash = get_blockhash().await;
        // Add the blockhash to the transaction
        tx.message.recent_blockhash = blockhash;
        // Serialize the transaction into bytes
        let tx_bytes = bincode::serialize(&tx).unwrap();

        // Specify which options to pass to the browser wallet.
        // Here we use default options
        let send_options = SendOptions::default();

        // Request the wallet to sign and send the transaction, returning the signature
        let signature = adapter.sign_and_send_transaction(&tx_bytes, Cluster::DevNet, send_options).await?;
        let signature_with_link = String::from("https://explorer.solana.com/tx/") + &Utils::base58_signature(signature).as_str() + "?cluster=devnet";
    }
    Ok(())
}

```
Remember to add the necessary dependencies for this part in the `Cargo.toml` manifest.
```toml
[dependencies]
# Add latest versions of these crates
solana-sdk = "^2.1.2"
bincode = "^1.3.3"
jzon = "^0.12.5"
serde_json = "1.0.133"
serde = { version = "^1.0.215", features = ["derive"] }
```
**NOTE** that if the signed transaction is verified by the library and an error is thrown in case of signature mismatch.

### LICENSE
Apache-2.0 OR MIT

### Features
- [x] Register `wallet-standard:register-wallet` custom event
- [x] App Ready `wallet-standard:app-ready` custom event
- [x] Wallet Info
- [x] Wallet Account parsing
- [x] Wallet Icon
- [x] Chains
- [x] Clusters
- [x] Version (Semver Versionin)
- [x] Features
- [x] Connect Wallet `standard:connect`
- [x] Disconnect Wallet `standard:disconnect`
- [x] SignIn (Sign In With Solana SIWS)
- [x] Sign Message
- [x] Sign Transaction
- [x] Sign and Send Transaction
- [x] Standard Events like Connect, Disconnect and (Re-connect & Account Changed (for wallets that support this)


### Templates
All templates can be found at the [templates directory](https://github.com/JamiiDao/SolanaWalletAdapter/tree/master/templates/).

Learn how the templates work from [https://github.com/JamiiDao/SolanaWalletAdapter/blob/master/templates/README.md](https://github.com/JamiiDao/SolanaWalletAdapter/blob/master/templates/README.md)



### Code of conduct for contributions

All conversations and contributors must agree to [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct)

