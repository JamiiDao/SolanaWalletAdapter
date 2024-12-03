use solana_sdk::{
    native_token::LAMPORTS_PER_SOL, pubkey::Pubkey, system_instruction, transaction::Transaction,
};
use wallet_adapter::{Cluster, Utils};
use yew::prelude::*;

use crate::AdapterActions;

#[function_component]
pub fn SignTxComponent(controller: &AdapterActions) -> Html {
    let signed_tx_output: UseStateHandle<Option<Transaction>> = use_state(Option::default);
    let public_key = controller.connected_account.public_key;
    let pubkey = Pubkey::new_from_array(public_key);
    let recipient_pubkey = Pubkey::new_from_array(Utils::public_key_rand());
    let sol = LAMPORTS_PER_SOL;

    let connected_wallet = controller.connected_wallet.clone();
    let connected_account = controller.connected_account.clone();

    html! {
        if signed_tx_output.is_none() {
            <div class="inner-section">
                    <div class="inner-header"> {"SEND SOL TX"}</div>
                    <div class="inner-body"> {"FROM: "} {connected_account.address.as_str()}</div>
                    <div class="inner-body"> {"TO: "} {recipient_pubkey}</div>
                    <div class="inner-body"> {"LAMPORTS: "} {sol}</div>
                    <button id="btn-primary"
                        onclick={Callback::from(move |_| {
                            let connected_wallet= connected_wallet.clone();
                            let connected_account= connected_account.clone();
                            let signed_tx_output= signed_tx_output.clone();

                            wasm_bindgen_futures::spawn_local(async move {
                                let instr = system_instruction::transfer(&pubkey, &recipient_pubkey, sol);
                                let tx = Transaction::new_with_payer(&[instr], Some(&pubkey));
                                let tx_bytes = bincode::serialize(&tx).unwrap();
                                let output = connected_wallet.sign_transaction(&tx_bytes, Some(Cluster::DevNet), &connected_account).await.unwrap();
                                let deser_tx_output = bincode::deserialize::<Transaction>(&output[0]).unwrap();
                                signed_tx_output.set(Some(deser_tx_output));
                            });
                        })}> {"SIGN TX"}
                    </button>
            </div>
        }else {
            <div class="inner-section">
                <div class="inner-header"> {"SIGNED SEND SOL TX"}</div>
                <div class="inner-body"> {"FROM: "} {connected_account.address.as_str()}</div>
                <div class="inner-body"> {"TO: "} {recipient_pubkey}</div>
                <div class="inner-body"> {"LAMPORTS: "} {sol}</div>
            </div>
        }
    }
}
