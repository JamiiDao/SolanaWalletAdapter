use solana_sdk::{
    native_token::LAMPORTS_PER_SOL, pubkey::Pubkey, system_instruction, transaction::Transaction,
};
use sycamore::{futures::spawn_local_scoped, prelude::*};
use wallet_adapter::{Cluster, Utils, WalletAdapter};

#[component]
pub fn SignTxComponent() -> View {
    let adapter = use_context::<Signal<WalletAdapter>>();

    let signed_tx_output: Signal<Option<Transaction>> = create_signal(Option::default());

    let connected_account = adapter
        .get_clone()
        .connected_account()
        .cloned()
        .as_ref()
        .unwrap()
        .clone();

    let public_key = connected_account.public_key;
    let recipient_pubkey = Pubkey::new_from_array(Utils::public_key_rand());
    let sol = LAMPORTS_PER_SOL;
    let from = connected_account.address.clone();
    let pubkey = Pubkey::new_from_array(public_key);

    view! {
        (if signed_tx_output.get_clone().is_none() {
            let from = from.clone();
            view!{
                div(class="inner-section"){
                    div(class="inner-header"){ "SEND SOL TX"}
                    div(class="inner-body"){ "FROM: " (from)}
                    div(class="inner-body"){ "TO: " (recipient_pubkey.to_string())}
                    div(class="inner-body"){ "LAMPORTS: " (sol)}

                    button (class="btn-inner", on:click={

                        move |_| {

                            spawn_local_scoped(async move {
                                let instr = system_instruction::transfer(&pubkey, &recipient_pubkey, sol);
                                let tx = Transaction::new_with_payer(&[instr], Some(&pubkey));
                                let tx_bytes = bincode::serialize(&tx).unwrap();
                                let output = adapter.get_clone().sign_transaction(&tx_bytes, Some(Cluster::DevNet)).await.unwrap();
                                let deser_tx_output = bincode::deserialize::<Transaction>(&output[0]).unwrap();
                                signed_tx_output.set(Some(deser_tx_output));
                            });
                        }
                    }){"SIGN TX"}
                }
            }
        }else {
            let from = from.clone();

            view!{div(class="inner-section"){
                div(class="inner-header"){ "SIGNED SEND SOL TX"}
                div(class="inner-body"){ "FROM: " (from)}
                div(class="inner-body"){ "TO: " (recipient_pubkey.to_string())}
                div(class="inner-body"){ "LAMPORTS: " (sol)}
            }}
        })
    }
}
