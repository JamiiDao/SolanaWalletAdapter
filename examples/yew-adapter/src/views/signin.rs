use wallet_adapter::{SignInOutput, SigninInput};
use yew::prelude::*;

use super::ConnectedAccounts;

#[function_component]
pub fn SignInComponent(connection: &ConnectedAccounts) -> Html {
    let connected_account = connection.account.clone();
    let connected_wallet = connection.wallet.clone();

    let signin_output: UseStateHandle<Option<SignInOutput>> = use_state(|| Option::None);
    let message = "YEW LOGIN";
    let signin_supported = connected_wallet.solana_signin();
    let connected_wallet = connected_wallet.clone();
    let connected_account = connected_account.clone();
    let address = connected_account.address.clone();

    html! {
        if signin_output.is_none() {
            <div class={classes!("inner-section")}>
                <div class={classes!("inner-header")}> {"SIGN IN DETAILS"}</div>
                <div class={classes!("inner-body")}> {"ADDRESS:" } {address}</div>
                <div class={classes!("inner-body")}> {"MESSAGE: "} {message}</div>
                if signin_supported {
                    <button class={"btn-inner"}
                        onclick={
                            Callback::from(move |_| {
                                let signin_output = signin_output.clone() ;
                                let connected_wallet = connected_wallet.clone();
                                let address = connected_account.address.clone();
                                let public_key = connected_account.public_key;


                                let mut signin_input = SigninInput::new();
                                signin_input
                                    .set_domain(&web_sys::window().unwrap())
                                    .unwrap()
                                    .set_statement(message)
                                    .set_chain_id(wallet_adapter::Cluster::DevNet)
                                    // NOTE: Some wallets require this field or the wallet adapter
                                    // will return an error `MessageResponseMismatch` which is as
                                    // a result of the sent message not corresponding with the signed message
                                    .set_address(&address)
                                    .unwrap();


                                    wasm_bindgen_futures::spawn_local(async move {
                                        let outcome = connected_wallet.sign_in(&signin_input, public_key).await.unwrap();
                                        signin_output.set(Some(outcome));
                                    });
                            })
                        }> {"SIGN IN"}</button>
                }else {
                    <button class="btn-inner-disabled">{"SIWS Unsupported"}</button>
                }
            </div>
        }else {
            <div class="inner-section">
                <div class="inner-header">{ "SIGNED DETAILS"}</div>
                <div class="inner-body">{"ACCOUNT  : "} {signin_output.as_ref().unwrap().address()}</div>
                <div class="inner-body">{"MESSAGE  : "} {signin_output.as_ref().unwrap().message.as_str()}</div>
                <p class="inner-body">{ "SIGNATURE: "} {signin_output.as_ref().unwrap().signature()}</p>
            </div>
        }
    }
}
