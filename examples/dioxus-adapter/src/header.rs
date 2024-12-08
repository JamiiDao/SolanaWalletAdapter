use dioxus::prelude::*;
use wallet_adapter::Wallet;

use crate::{AdapterCluster, DioxusWalletAdapter, Route};

#[component]
pub fn Header() -> Element {
    let mut adapter: Signal<DioxusWalletAdapter> = use_context();

    let cluster_node = |input_cluster: AdapterCluster| -> Element {
        let (disabled_btn, class_select) = if input_cluster == adapter.read().active_cluster {
            (true, "btn-primary")
        } else {
            (false, "btn-ghost")
        };

        rsx! {
            li{ button{
                    onclick:move|_| {
                        adapter.write().active_cluster = input_cluster.clone();
                    },
                    disabled:disabled_btn,
                    class:"btn btn-sm {class_select}", {input_cluster.name.clone()}}
                }
        }
    };

    let mut show_dropdown = use_signal(|| "");

    let mut toggle_dropdown = move || {
        let option = show_dropdown.read().clone();
        if option.is_empty() {
            show_dropdown.set("wallet-adapter-dropdown-list-active");
        } else {
            show_dropdown.set("");
        }
    };

    rsx! {
        div{class:"navbar flex-col md:flex-row space-y-2 md:space-y-0",
            div{class:"flex-1",
                a{ class:"btn btn-ghost normal-case text-xl", href:"/",
                    img{ class:"h-4 md:h-6", alt:"Logo", src:crate::LOGO }
                }
                ul{class:"menu menu-horizontal px-1 space-x-2",
                    li{  Link { to: Route::WalletDashboard {}, "Home" } }
                }
            }

            div{ class:"flex-none space-x-2",
                div{ class:"wallet-adapter-dropdown",
                    if adapter.read().connection.is_connected() {
                        div { class: "wallet-adapter-dropdown",
                            button {
                                onclick:move|_| {
                                    toggle_dropdown();
                                },
                                onblur:move|_| {
                                    toggle_dropdown();
                                },
                                r#type: "button",
                                tabindex: "0",
                                class: "wallet-adapter-button wallet-adapter-button-trigger",
                                i { class: "wallet-adapter-button-start-icon",
                                    img { src:adapter.read().connection.connected_wallet().as_ref().unwrap().icon().as_ref().unwrap().to_string(),
                                            alt: "{adapter.read().connection.connected_wallet().as_ref().unwrap().name()} icon" }
                                }
                                {adapter.read().connection.connected_account().as_ref().unwrap().shorten_address().unwrap()}
                            }
                            ul {
                                role: "menu",
                                "aria-label": "dropdown-list",
                                class: "wallet-adapter-dropdown-list {show_dropdown.read()}",
                                li { role: "menuitem", class: "wallet-adapter-dropdown-list-item", "Copy address" }
                                li {onclick:move|_| {adapter.write().show_modal=true}, role: "menuitem", class: "wallet-adapter-dropdown-list-item", "Change wallet" }
                                li {
                                    onclick:move|_| {
                                        spawn(async move{
                                            adapter.write().connection.disconnect().await.unwrap();
                                            toggle_dropdown();

                                        });
                                    },
                                    role: "menuitem", class: "wallet-adapter-dropdown-list-item", "Disconnect"
                                }
                            }
                        }
                    }else {
                        button{onclick: move |_| {
                            adapter.write().show_modal =true;
                        }, class:"wallet-adapter-button wallet-adapter-button-trigger",
                            style:"pointer-events: auto;", tabindex:"0", r#type:"button", "Select Wallet",
                        }
                    }
                }
                div{ class:"dropdown dropdown-end",
                    label {tabindex:"0", class:"btn btn-primary rounded-btn", {adapter.read().active_cluster.name.clone()}}
                    ul{ tabindex:"0",
                        class:"menu dropdown-content z-[1] p-2 shadow bg-base-100 rounded-box w-52 mt-4",
                        for cluster in adapter.read().clusters.clone() {
                            {cluster_node(cluster)}
                        }
                    }
                }
            }
        }

        if adapter.read().show_modal {
            {ShowModal()}
        }

        Outlet::<Route> {}
    }
}

pub fn ShowModal() -> Element {
    let mut adapter: Signal<DioxusWalletAdapter> = use_context();

    let build_node = |wallet: Wallet| -> Element {
        let icon = wallet.icon().as_ref().unwrap().to_string();
        let name = wallet.name().to_string();
        rsx! {
            li {
                button {onclick:move|_|{
                    let wallet = wallet.clone();
                        spawn(async move {
                            adapter.write().connection.connect(wallet).await.unwrap();
                            adapter.write().show_modal = false;
                        });
                    },
                    class: "wallet-adapter-button", tabindex: "0", r#type: "button",
                    i {class: "wallet-adapter-button-start-icon",
                        img {src:icon, alt: "{name} icon"}
                    }
                    {wallet.name()},
                    span { "Detected" }
                }
            }
        }
    };

    rsx! {
        div{ aria_labelledby:"wallet-adapter-modal-title", aria_modal:"true",
            class:"wallet-adapter-modal wallet-adapter-modal-fade-in", role:"dialog",
            div{ class:"wallet-adapter-modal-container",
                div{ class:"wallet-adapter-modal-wrapper",
                    button{onclick:move |_|{
                        adapter.write().show_modal =false;
                    },
                        class:"wallet-adapter-modal-button-close", {CloseSvg()}
                    }
                    h1 {class:"wallet-adapter-modal-title", "Connect a wallet on Solana to continue" }
                    ul {
                        class: "wallet-adapter-modal-list",

                        for wallet in adapter.read().connection.wallets(){
                            {build_node(wallet)}
                        }
                    }
                }
                div{class:"wallet-adapter-modal-overlay"}
            }
        }
    }
}

pub fn CloseSvg() -> Element {
    rsx! {
        svg {width:"14", height:"14",
            path{ d:"M14 12.461 8.3 6.772l5.234-5.233L12.006 0 6.772 5.234 1.54 0 0 1.539l5.234 5.233L0 12.006l1.539 1.528L6.772 8.3l5.69 5.7L14 12.461z"}
        }
    }
}
