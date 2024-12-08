use sycamore::{futures::spawn_local_scoped, prelude::*};
use wallet_adapter::{Wallet, WalletAdapter};

#[component]
pub fn Header() -> View {
    let adapter = use_context::<Signal<WalletAdapter>>();
    let show_modal = create_signal(false);

    let show_dropdown = create_signal("wallet-adapter-dropdown-list");

    let toggle_dropdown = move || {
        if show_dropdown.get_clone() == "wallet-adapter-dropdown-list" {
            show_dropdown.set("wallet-adapter-dropdown-list wallet-adapter-dropdown-list-active");
        } else {
            show_dropdown.set("wallet-adapter-dropdown-list");
        }
    };

    view! {
        div(class="navbar text-neutral-content flex-col md:flex-row space-y-2 md:space-y-0"){
            div(class="flex-1"){
                a(class="btn btn-ghost normal-case text-xl", href="/"){
                    img(class="h-4 md:h-6", alt="Logo", src="logo.png"){}
                }
                ul(class="menu menu-horizontal px-1 space-x-2"){
                    li{  a(href="/"){ "Home" } }
                }
            }

            div(class="flex-none space-x-2"){
                div(class="wallet-adapter-dropdown"){
                    (if adapter.get_clone().is_connected() {
                        let address = adapter.get_clone().connected_account().as_ref().unwrap().shorten_address().unwrap().to_string();
                        view!{
                            div(class="wallet-adapter-dropdown"){
                                button (
                                    on:click=move|_| {
                                        toggle_dropdown();
                                    },
                                    on:blur=move|_| {
                                        toggle_dropdown();
                                    },
                                    r#type="button",
                                    tabindex="0",
                                    class="wallet-adapter-button wallet-adapter-button-trigger"){
                                    i(class="wallet-adapter-button-start-icon"){
                                        img(src=adapter.get_clone().connected_wallet().as_ref().unwrap().icon().as_ref().unwrap().to_string(),
                                                alt="{adapter.read().connection.connected_wallet().as_ref().unwrap().name()} icon" ){}
                                    }
                                    (address)
                                }
                                ul (
                                    role="menu",
                                    "aria-label"="dropdown-list",
                                    class=show_dropdown.get_clone()){
                                    li (role="menuitem", class="wallet-adapter-dropdown-list-item"){"Copy address" }
                                    li (on:click=move|_| {show_modal.set(true)}, role="menuitem", class="wallet-adapter-dropdown-list-item"){"Change wallet" }
                                    li (on:click=move|_| {
                                            spawn_local_scoped(async move{
                                                let mut inner_adapter = adapter.get_clone().clone();
                                                inner_adapter.disconnect().await.unwrap();
                                                adapter.set(inner_adapter);

                                            });
                                        },
                                        role="menuitem", class="wallet-adapter-dropdown-list-item"){"Disconnect"
                                    }
                                }
                            }
                        }
                    }else {
                        view!{
                            button(on:click=move |_| {
                                show_modal.set(true);
                            }, class="wallet-adapter-button wallet-adapter-button-trigger",
                                style="pointer-events: auto;", tabindex="0", r#type="button"){"Select Wallet"
                            }
                        }
                    })
                }
            }
        }

        (if show_modal.get() {
            ShowModal(show_modal)
        }else {view!()})
    }
}

#[component]
pub fn ShowModal(show_modal: Signal<bool>) -> View {
    let adapter = use_context::<Signal<WalletAdapter>>();

    let build_node = |wallet: Wallet| -> View {
        let icon = wallet.icon().as_ref().unwrap().to_string();
        let name = wallet.name().to_string();
        view! {
            li {
                button(on:click=move|_|{
                    let wallet = wallet.clone();
                        spawn_local_scoped(async move {
                            let mut adapter_inner = adapter.get_clone().clone();
                            adapter_inner.connect(wallet.clone()).await.unwrap();
                            adapter.set(adapter_inner);

                            show_modal.set(false);


                        });
                    },
                    class="wallet-adapter-button", tabindex="0", r#type="button"){
                    i(class="wallet-adapter-button-start-icon"){
                        img(src=icon, alt="{name} icon"){}
                    }
                    (name)
                    span { "Detected" }
                }
            }
        }
    };

    let wallets_list_views = adapter
        .get_clone()
        .wallets()
        .into_iter()
        .map(build_node)
        .collect::<Vec<View>>();

    view! {
        div(aria-labelledby="wallet-adapter-modal-title", aria-modal="true",
            class="wallet-adapter-modal wallet-adapter-modal-fade-in", role="dialog"){
            div(class="wallet-adapter-modal-container"){
                div(class="wallet-adapter-modal-wrapper"){
                    button(on:click=move |_|{
                        show_modal.set(false);
                    },
                        class="wallet-adapter-modal-button-close"){ (CloseSvg())
                    }
                    h1(class="wallet-adapter-modal-title"){"Connect a wallet on Solana to continue" }
                    ul(class="wallet-adapter-modal-list"){
                        (wallets_list_views)
                    }
                }
                div(class="wallet-adapter-modal-overlay"){}
            }
        }
    }
}

#[component]
pub fn CloseSvg() -> View {
    view! {
        svg(width="14", height="14"){
            path(d="M14 12.461 8.3 6.772l5.234-5.233L12.006 0 6.772 5.234 1.54 0 0 1.539l5.234 5.233L0 12.006l1.539 1.528L6.772 8.3l5.69 5.7L14 12.461z"){}
        }
    }
}
