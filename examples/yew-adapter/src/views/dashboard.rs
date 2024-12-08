use wallet_adapter::{Wallet, WalletAccount, WalletAdapter};
use yew::{platform::spawn_local, prelude::*};
use yew_router::prelude::Link;

use crate::{
    Footer, Route, SignAndSendTxComponent, SignInComponent, SignMessageComponent, SignTxComponent,
};

#[derive(Debug, Properties, PartialEq, Clone)]
pub struct ConnectedAccounts {
    pub wallet: Wallet,
    pub account: WalletAccount,
}

#[function_component(Dashboard)]
pub fn dashboard() -> Html {
    let global_adapter =
        use_context::<WalletAdapter>().expect("no global ctx `WalletAdapter` found");
    let adapter = use_state(|| global_adapter);

    let show_dropdown = use_state(|| "wallet-adapter-dropdown-list");
    let show_modal = use_state(|| false);

    html! {
        <div id="root">
                <div class="h-full flex flex-col">
                    {header(adapter.clone(), show_modal, show_dropdown)}

                   <div class="flex-grow mx-4 lg-mx-auto">

                        if adapter.is_connected() {
                            <>
                            <SignInComponent
                                wallet={adapter.connected_wallet().cloned().as_ref().unwrap().clone()}
                                account={adapter.connected_account().cloned().as_ref().unwrap().clone()}
                            />
                            <SignMessageComponent
                                wallet={adapter.connected_wallet().cloned().as_ref().unwrap().clone()}
                                account={adapter.connected_account().cloned().as_ref().unwrap().clone()}
                            />
                            <SignTxComponent
                                wallet={adapter.connected_wallet().cloned().as_ref().unwrap().clone()}
                                account={adapter.connected_account().cloned().as_ref().unwrap().clone()}
                            />
                            <SignAndSendTxComponent
                                wallet={adapter.connected_wallet().cloned().as_ref().unwrap().clone()}
                                account={adapter.connected_account().cloned().as_ref().unwrap().clone()}
                            />
                            <div style="position: fixed; z-index: 9999; inset: 16px; pointer-events: none;"></div>
                            </>
                        }else {
                            <div class="min-height40vh centered"> {"CONNECT A WALLET FIRST"} </div>
                        }
                    </div>
                          <Footer/>

         </div>

     </div>
    }
}

pub fn header(
    adapter: UseStateHandle<WalletAdapter>,
    show_modal: UseStateHandle<bool>,
    show_dropdown: UseStateHandle<&'static str>,
) -> Html {
    let connection = adapter.clone();

    let inner_connection = (*connection.clone()).clone();
    let show_dropdow_inner = show_dropdown.clone();

    let toggle_dropdown = move || {
        let show_dropdow_inner = show_dropdow_inner.clone();
        if *show_dropdow_inner == "wallet-adapter-dropdown-list" {
            show_dropdow_inner
                .clone()
                .set("wallet-adapter-dropdown-list wallet-adapter-dropdown-list-active");
        } else {
            show_dropdow_inner.set("wallet-adapter-dropdown-list");
        }
    };

    html! {
        <><div class="navbar flex-col md:flex-row space-y-2 md:space-y-0">
            <div class="flex-1">
                <a class="btn btn-ghost normal-case text-xl" href="/">
                    <img class="h-4 md-h-6" alt="Logo" src="/logo.png" />
                </a>
                <ul class="menu menu-horizontal px-1 space-x-2">
                    <li> <Link<Route> to={Route::Home}>{ "Home" }</Link<Route>> </li>
                </ul>
            </div>

            <div class="flex-none space-x-2">
                <div class="wallet-adapter-dropdown">
                    if adapter.is_connected() {
                        <div class="wallet-adapter-dropdown">
                            <button
                                onclick={
                                    let toggle_dropdown = toggle_dropdown.clone();
                                    Callback::from(move |_| {
                                    toggle_dropdown();
                                })}
                                onblur={Callback::from(move |_| {
                                    toggle_dropdown();
                                })}

                                type="button"
                                tabindex="0"
                                class="wallet-adapter-button wallet-adapter-button-trigger">
                                    <i class={"wallet-adapter-button-start-icon"}>
                                        <img src={adapter.connected_wallet().as_ref().unwrap().icon().as_ref().unwrap().to_string()}
                                            alt="{adapter.read().connection.connected_wallet().as_ref().unwrap().name()} icon" />
                                    </i>
                                    {adapter.connected_account().as_ref().unwrap().shorten_address().unwrap()}
                                </button>
                            <ul role="menu" aria-label="dropdown-list"
                                class={*show_dropdown}>
                                <li role="menuitem" class="wallet-adapter-dropdown-list-item"> {"Copy address"} </li>
                                <li onclick={
                                    let show_modal = show_modal.clone();

                                    Callback::from(move|_| {
                                        show_modal.set(true)})}
                                    role="menuitem" class="wallet-adapter-dropdown-list-item">
                                    {"Change wallet"}
                                </li>
                                <li onclick={
                                    let inner_connection = inner_connection.clone();
                                    let adapter = adapter.clone();
                                    Callback::from(move|_| {
                                        let mut inner_connection = inner_connection.clone();
                                        let adapter = adapter.clone();

                                        spawn_local(async move{
                                            inner_connection.disconnect().await.unwrap();
                                            adapter.set(inner_connection);
                                        });
                                    })}
                                    role="menuitem" class="wallet-adapter-dropdown-list-item"> {"Disconnect"}
                                </li>
                            </ul>
                        </div>
                    }else {
                        <button
                        onclick={
                            let show_modal = show_modal.clone();
                            Callback::from(move |_| {
                                show_modal.set(true);
                            })
                        }
                        class="wallet-adapter-button wallet-adapter-button-trigger"
                        style="pointer-events=auto;" tabindex="0" type="button">{"Select Wallet"}</button>
                    }
                </div>
            </div>
        </div>

        if *show_modal {
            {show_modal_component(show_modal, adapter)}
        }
        </>
    }
}

pub fn show_modal_component(
    show_modal: UseStateHandle<bool>,
    adapter: UseStateHandle<WalletAdapter>,
) -> Html {
    let show_modal = show_modal.clone();
    let connection1 = adapter.clone();
    let connection = (*connection1.clone()).clone();

    let build_node = |wallet: Wallet| -> Html {
        let icon = wallet.icon().as_ref().unwrap().to_string();
        let alt = wallet.name().to_string() + " icon";

        html! {
            <li>
                <button
                    onclick={
                        let wallet = wallet.clone();
                        let show_modal = show_modal.clone();
                        let connection = connection.clone();
                        let adapter = adapter.clone();

                        Callback::from(move|_|{
                        let wallet = wallet.clone();
                        let show_modal = show_modal.clone();
                        let mut connection = connection.clone();
                        let adapter = adapter.clone();

                        spawn_local(async move {
                            connection.connect(wallet).await.unwrap();
                            adapter.set(connection);
                            show_modal.set(false);

                        });
                    })}
                    class="wallet-adapter-button" tabindex= "0" type= "button">
                    <i class="wallet-adapter-button-start-icon">
                        <img src={icon} alt={alt} />
                    </i>
                    {wallet.name()}
                    <span> { "Detected" }</span>
                </button>
            </li>
        }
    };

    html! {
        <div aria-labelledby="wallet-adapter-modal-title" aria-modal="true"
            class="wallet-adapter-modal wallet-adapter-modal-fade-in" role="dialog">
            <div class="wallet-adapter-modal-container">
                <div class="wallet-adapter-modal-wrapper">
                    <button
                        onclick={
                            let show_modal = show_modal.clone();
                            Callback::from(move |_|{
                                show_modal.clone().set(false)
                            })}
                        class="wallet-adapter-modal-button-close">
                        <CloseSvg/>
                    </button>
                    <h1 class="wallet-adapter-modal-title"> {"Connect a wallet on Solana to continue"} </h1>
                    <ul class="wallet-adapter-modal-list">
                        { adapter.wallets().iter().map(|wallet|{
                            {build_node(wallet.clone())}
                        }).collect::<Vec<Html>>() }
                    </ul>
                </div>
                <div class="wallet-adapter-modal-overlay"></div>
            </div>
        </div>
    }
}

#[function_component(CloseSvg)]
pub fn close_svg() -> Html {
    html! {
        <svg width="14" height="14">
            <path d="M14 12.461 8.3 6.772l5.234-5.233L12.006 0 6.772 5.234 1.54 0 0 1.539l5.234 5.233L0 12.006l1.539 1.528L6.772 8.3l5.69 5.7L14 12.461z"/>
        </svg>
    }
}
