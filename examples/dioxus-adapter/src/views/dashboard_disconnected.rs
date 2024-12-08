use dioxus::prelude::*;

use crate::{DioxusWalletAdapter, Footer, SignAndSendTx, SignInComponent, SignMessage, SignTx};

pub fn WalletDashboard() -> Element {
    let adapter: Signal<DioxusWalletAdapter> = use_context();

    rsx! {
        div {class:"flex-grow mx-4 lg:mx-auto",
            if adapter.read().connection.is_connected(){
                div{
                    {SignInComponent()}
                    {SignMessage()}
                    {SignTx()}
                    {SignAndSendTx()}
                }
            }else {
                div{class:"min-height40vh centered", "CONNECT A WALLET FIRST"}
            }
            div {style:"position: fixed; z-index: 9999; inset: 16px; pointer-events: none;"}
        }
        Footer{}
    }
}
