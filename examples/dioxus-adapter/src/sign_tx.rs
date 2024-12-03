use dioxus::prelude::*;
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL, pubkey::Pubkey, system_instruction, transaction::Transaction,
};
use wallet_adapter::{Cluster, Utils, WalletAdapter};

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
