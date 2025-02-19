use sycamore::{futures::spawn_local_scoped, prelude::*};
use wallet_adapter::{ConnectionInfo, WalletAdapter};

use crate::{app::GlobalMessage, sign_message_svg, NotificationInfo};

#[component]
pub fn SignMessage() -> View {
    let adapter = use_context::<Signal<WalletAdapter>>();
    let active_connection = use_context::<Signal<ConnectionInfo>>();
    let global_message = use_context::<Signal<GlobalMessage>>();

    let message = "Solana Foundation is awesome!";

    let mut solana_signmessage = false;

    if let Ok(wallet_account) = active_connection.get_clone().connected_account() {
        solana_signmessage = wallet_account.solana_sign_message();
    }

    view! {
        div (class="flex dark:bg-[#160231] bg-white flex-col w-[300px] p-5 rounded-lg dark:shadow-2xl shadow-sm border dark:border-none"){
            div (class="w-full flex flex-col items-center text-center text-true-blue justify-center mb-10"){
                div(class="w-[80px] flex flex-col"){ img(src=sign_message_svg()) }
                div(class="w-full text-sm"){"Sign Message"}
            }
            div (class="text-lg text-center"){ (message) }

            div (class="flex items-center justify-center"){
                (if solana_signmessage {
                    view!{button(class="bg-true-blue  hover:bg-cobalt-blue mt-5 text-sm text-white px-5 py-2 rounded-full",
                        on:click=move |_| {
                            spawn_local_scoped(async move {
                                if let Err(error) = adapter.get_clone().sign_message(message.as_bytes()).await{
                                    global_message.update(|store| store.push_back(
                                        NotificationInfo::error(
                                            format!("SIGN MESSAGE ERROR: {error:?}")
                                        ))
                                    );
                                }else {
                                    global_message.update(|store| store.push_back(
                                        NotificationInfo::new("Sign Message Successful")
                                    ));
                                }
                            });
                        }){
                        "SIGN MESSAGE"
                    }}
                }else {
                    view!{div(class="w-full items-center justify-center"){
                        "SIGN MESSAGE UNSUPPORTED"
                    }}
                }
            )
            }
        }
    }
}
