use dioxus::prelude::*;

use crate::{
    trunk_cluster_name, utils::copied_address, views::ClusterNetState, ChangeWalletSvg, CloseSvg,
    ClustersSvg, CopySvg, DisconnectSvg, FetchReq, GradientWalletIcon, Loader, NotificationInfo,
    Route, WalletSvg, ACTIVE_CONNECTION, CLUSTER_NET_STATE, CLUSTER_STORAGE, GLOBAL_MESSAGE, LOGO,
    WALLET_ADAPTER,MenuSvg,
};

#[component]
pub fn Header() -> Element {
    let show_modal = use_signal(|| false);
    let show_connecting = use_signal(|| false);
    let mut show_mobile_close_button = use_signal(|| bool::default());

    let mut shortened_address = String::default();

    if let Ok(wallet_account) = ACTIVE_CONNECTION.read().connected_account() {
        shortened_address = wallet_account
            .shorten_address()
            .unwrap_or_default()
            .to_string();
    }

    rsx! {
        div { class:"flex flex-col w-full gap-4 justify-between items-center",
            nav {class:"flex w-full justify-between items-center p-1 dark:shadow-lg shadow-sm border-b-[1px] dark:border-true-blue",
                div{class:"p-1 w-[25%] md:w-[15%]", img{src:LOGO, alt:"LOGO"} }

            div{class:"flex w-[75%] items-center justify-center hidden md:inline-flex",

                div{ class:"flex items-center justify-around w-[80%] mx-2",
                    {NavItem(Route::Dashboard, "Home")}
                    {NavItem(Route::Accounts, "Accounts")}
                    {NavItem(Route::Clusters, "Clusters")}
                    {NavItem(Route::Extras, "Extras")}
                    {NavClusterItem()}
                }
                NavWalletItem{show_modal, show_connecting, shortened_address:shortened_address.clone(), show_mobile_close_button}
            }

                if !*show_mobile_close_button.read() {
                    div { class:"w-[25%] flex ml-2 text-white py-1 px-4 appearance-none items-center justify-center cursor-pointer inline-flex  md:hidden",
                        div{class:"flex appearance-none text-center cursor-pointer",
                            span{
                                onclick:move |_|{show_mobile_close_button.set(true)},
                                class:"flex w-[15px]", {MenuSvg()} 
                            }
                        }
                    }
                }else {
                    div{class:"flex flex-col w-full z-40 absolute bg-black bg-opacity-50 inset-0 h-full p-15 inline-flex md:hidden justify-center items-center",
                        div{class:"flex flex-col w-[80%] h-auto bg-rich-black m-2 p-2 inline-flex md:hidden justify-center items-center",
                            div{class:"flex w-[80%] flex-col md:lg:flex-row items-end justify-end",
                                span{
                                    onclick:move |_|{show_mobile_close_button.set(false)},
                                    class:"flex w-[30px]", {CloseSvg()} 
                                }
                            }
                            
                            div {class:"flex flex-col md:lg:flex-row items-center justify-center w-full",
                                div{ class:"flex flex-col md:lg:flex-row  items-center justify-center w-full md:w-[80%] mx-2",
                                    {NavItem(Route::Dashboard, "Home")}
                                    {NavItem(Route::Accounts, "Accounts")}
                                    {NavItem(Route::Clusters, "Clusters")}
                                    {NavItem(Route::Extras, "Extras")}
                                    {NavClusterItem()}
                                }
                                NavWalletItem{show_modal, show_connecting, shortened_address, show_mobile_close_button}
                            }
                        }
                    }
                }
            }
            PingCluster {  }
        }


        ConnectWalletModalModal { show_modal, show_connecting }

        Outlet::<Route> {}
    }
}

#[component]
pub fn ConnectWalletModalModal(show_modal: Signal<bool>, show_connecting: Signal<bool>,) -> Element {
    if *show_modal.read() {
        rsx! {
            div{class:"flex flex-col w-full h-full bg-[#1a1a1a88] absolute items-center justify-center z-50",

                div { class: "flex relative w-full max-w-[90%] lg:max-w-[40%] md:max-w-[55%] max-h-full",
                    if !WALLET_ADAPTER.read().wallets().is_empty() {
                        div { class: "relative bg-white rounded-lg shadow-lg dark:bg-rich-black items-center justify-between flex flex-col w-full h-full min-h-[40vh]",
                            div { class: "flex items-center justify-center p-4 md:p-5 rounded-t w-full dark:border-gray-600 border-gray-200",
                                div {
                                    class:"flex w-5/6 items-center justify-center",
                                    h3 { class: "text-2xl flex items-center justify-center font-semibold text-blue-yonder dark:text-white",
                                        span{class:"w-[30px] mr-2 flex", {GradientWalletIcon()}} "Connect A Wallet"
                                    }
                                }
                                div { class:"flex w-1/6",
                                    button {
                                        onclick:move|_| {show_modal.set(false);},
                                        class: "text-gray-400 bg-transparent hover:bg-gray-200 hover:text-gray-900 rounded-lg text-sm w-8 h-8 ms-auto inline- justify-center items-center dark:hover:bg-gray-600 dark:hover:text-white",
                                        "data-modal-hide": "default-modal",
                                        r#type: "button",
                                        {CloseSvg()}
                                        span { class: "sr-only", "Close modal" }
                                    }
                                }
                            }
                            ul { class: "flex space-y-4 mb-5 w-full justify-center flex-col items-center h-full",
                                for wallet in WALLET_ADAPTER.read().wallets().clone() {
                                    li {
                                        onclick:move|_|{
                                            let wallet = wallet.clone();

                                            spawn(async move {
                                                show_modal.set(false);
                                                show_connecting.set(true);

                                                if let Err(error) = WALLET_ADAPTER.write().connect(wallet).await {
                                                    GLOBAL_MESSAGE.write().push_back(NotificationInfo::new(error));
                                                }

                                                show_connecting.set(false);
                                            });
                                        },
                                        class: "flex justify-center cursor-pointer w-full text-lg hover:bg-true-blue  text-true-blue hover:text-white dark:text-white px-4 py-2",
                                        div{class:"max-w-[80%] flex justify-between w-full",
                                            div {class:"flex items-center",
                                                if let Some(icon) = wallet.icon() {
                                                    img {class:"flex w-[25px] mr-2 items-center", src:icon.to_string(), alt:wallet.name()}
                                                }else {
                                                    span {class:"flex w-[25px] mr-2 items-center", {WalletSvg()}}
                                                }
                                                span {class:"flex", {wallet.name()}  }
                                            }
                                            span {class:"flex", "Detected"  }
                                        }
                                    }
                                }
                            }
                        }
                    }else {
                        div { class: "relative bg-white rounded-lg shadow-lg dark:bg-rich-black items-center justify-start p-2 flex flex-col w-full h-full min-h-[40vh]",
                                div { class:"flex w-full mr-5",
                                    button {
                                        onclick:move|_|{
                                            show_modal.set(false);
                                        },
                                        class: "text-gray-400 bg-transparent hover:bg-gray-200 hover:text-gray-900 rounded-lg text-sm w-8 h-8 ms-auto inline- justify-center items-center dark:hover:bg-gray-600 dark:hover:text-white",
                                        "data-modal-hide": "default-modal",
                                        r#type: "button",
                                        {CloseSvg()}
                                        span { class: "sr-only", "Close modal" }
                                    }
                                }
                                div{class:"flex text-2xl w-full p-5 flex-col items-center justify-around h-full",
                                    div{class:"flex w-full items-center justify-center",
                                        span{class:"flex w-[50px] mr-5 items-center", {GradientWalletIcon()}}
                                        span{"No Solana Wallets Detected"},
                                    }
                                    div {class:"flex text-lg", "Install a Solana Wallet Installed on your browser!"}
                                }
                        }
                    }
                }
            }
        }
    } else {
        rsx! {}
    }
}

fn NavItem(route: fn() -> Route, text: &str) -> Element {
    rsx! {
        Link {class:"w-full md:w-[10%] hover:bg-transparent dark:text-blue-yonder dark:hover:text-white text-true-blue hover:text-black rounded-lg text-center p-1", to: route(), {text}}
    }
}

fn NavClusterItem() -> Element {
    rsx! {
        div{
            class:"flex w-full items-center justify-center md:w-[15%]",
            select{
                onchange:move |event| {
                    let cluster = CLUSTER_STORAGE.read().get_cluster(&event.data.value()).cloned().unwrap_or_default();
                    let cluster_identifier = String::new() + cluster.name() + " cluster now active!";
                    CLUSTER_STORAGE.write().set_active_cluster(cluster);

                    GLOBAL_MESSAGE.write().push_back(NotificationInfo::new(cluster_identifier));
                },
                class:"flex text-sm hover:bg-true-yonder bg-true-blue text-white rounded-full md:py-1 md:px-4 appearance-none text-center cursor-pointer",
                for adapter_cluster in CLUSTER_STORAGE.read().get_clusters() {
                    option {
                        key:adapter_cluster.identifier.as_str(),
                        value:adapter_cluster.name(), selected:adapter_cluster.name().as_bytes() == CLUSTER_STORAGE.read().active_cluster().name().as_bytes(),
                        {trunk_cluster_name(adapter_cluster.name())},}
                }
            }
        }
    }
}

#[component]
fn NavWalletItem(
    show_modal: Signal<bool>,
    show_connecting: Signal<bool>,
    shortened_address: String,
    show_mobile_close_button: Signal<bool>
) -> Element {
    let compute_wallet = || {
        if let Ok(connected_account) = ACTIVE_CONNECTION.read().connected_account() {
            let shortened_address = connected_account.shorten_address().unwrap();
            let address = connected_account.address();

            rsx! { ActiveAccountDropDown{show_modal, shortened_address, address, show_mobile_close_button} }
        } else {
            rsx! {
                div {class:"flex w-full items-center justify-center",
                button {class:"flex w-full text-sm",
                    onclick:move|_|{
                        show_modal.set(true);
                        show_mobile_close_button.set(false);
                    },
                        "Select Wallet"
                    }
                }
            }
        }
    };

    rsx! {
        div { class:"w-full md:w-[25%] flex mt-5 md:mt-0 ml-2 text-white py-1 px-4 appearance-none items-center justify-center cursor-pointer",
            if *show_connecting.read() {
                div {class:"py-1 px-4 flex items-center justify-center hover:bg-true-yonder bg-true-blue rounded-full",
                    span{class:"flex w-[20px] mr-5", {WalletSvg()}}
                    span{class:"flex mr-5", {Loader()}}
                }
            } else {
                div{class:"flex hover:bg-true-yonder bg-true-blue text-white rounded-full py-1 px-4 appearance-none text-center cursor-pointer",
                    {compute_wallet()}
                }
            }
        }
    }
}

#[component]
pub fn ActiveAccountDropDown(show_modal: Signal<bool>, address: String, shortened_address: String, show_mobile_close_button: Signal<bool>) -> Element {
    let mut show_dropdown = use_signal(|| false);

    let disconnect_callback = move || {
        spawn(async move {
            WALLET_ADAPTER.write().disconnect().await;
            
            show_mobile_close_button.set(false);
        });
    };

    let clone_address = address.clone();

    let copy_callback = move || {
        let inner_address = clone_address.clone();
        spawn(async move {
            if let Err(error) = copied_address(&inner_address).await {
                GLOBAL_MESSAGE
                    .write()
                    .push_back(NotificationInfo::new(error));
            } else {
                GLOBAL_MESSAGE
                    .write()
                    .push_back(NotificationInfo::new("Copied to clipboard"));
            }
            show_mobile_close_button.set(false);
        });
    };

    let change_wallet_callback = move || {
        show_modal.set(true);
        show_mobile_close_button.set(false);

    };

    let connected_wallet = ACTIVE_CONNECTION.read().connected_wallet().unwrap().clone();

    rsx! {
        div {
            class:"relative inline-block rounded-full",
            div {
                onclick:move|_| {
                    if *show_dropdown.read() {
                        show_dropdown.set(false);
                    }else {
                        show_dropdown.set(true);
                    }
                },
                class:"flex w-full text-center items-center justify-center",
                span{class:"flex w-[20px] mr-2",
                    if let Some(icon) = connected_wallet.icon() {
                        img{class:"rounded-lg", src:icon.to_string()}
                    }else {
                        span{class:"text-sm", {WalletSvg()} }
                    }
                }
                {shortened_address}
            }

            if *show_dropdown.read() {
                ul {class:"w-full min-w-[130px] text-white flex flex-col absolute z-1 text-md mt-2 bg-true-blue rounded-lg shadow-xl list-none",
                    {DropdownItem("Copy Address", CopySvg(), show_dropdown, copy_callback)}
                    {DropdownItem("Change Wallet", ChangeWalletSvg(), show_dropdown, change_wallet_callback)}
                    {DropdownItem("Disconnect", DisconnectSvg(), show_dropdown, disconnect_callback)}
                }
            }
        }
    }
}

fn DropdownItem<F>(
    value: &str,
    icon: Element,
    mut show_dropdown: Signal<bool>,
    mut callback: F,
) -> Element
where
    F: FnMut() + 'static,
{
    rsx! {
        li{class:"flex w-full mb-2 mt-2 hover:bg-cobalt-blue cursor-pointer",
            onclick:move|_| {
                show_dropdown.set(false);
                callback();
            },
            div { class:"flex text-sm justify-left items-center ",
                span {class:"p-2 w-[30px]", {icon} }
                span { {value} }
            }
        }
    }
}

#[component]
fn PingCluster() -> Element {
    use_effect(move || {
        CLUSTER_STORAGE.read();
        spawn(async move {
            FetchReq::ping().await;
        });
    });

    if *CLUSTER_NET_STATE.read() == ClusterNetState::Failure {
        rsx! {
            div {class:"flex w-full justify-center min-h-[40px] bg-red-800 text-center items-center text-2xl justify-center items-center",
                div{ class:"flex px-4 py-2 justify-center items-center",
                    div{class:"flex flex-col md:flex-row w-full mr-2",
                        span { class:"flex hidden md:inline-flex w-[30px] mr-1 text-white text-[30px] md:text-md", {ClustersSvg()}}
                        {CLUSTER_STORAGE.read().active_cluster().name()} " cluster is unreachable!"
                    }
                    button {
                        onclick:move|_| {
                            let active_cluster = CLUSTER_STORAGE.read().active_cluster().clone();
                            CLUSTER_STORAGE.write().set_active_cluster(active_cluster);
                        },
                        class:"flex bg-true-blue items-center justify-center text-sm text-white px-2 py-1 rounded-full hover:bg-cobalt-blue",
                        "REFRESH"
                    }
                }
            }
        }
    } else {
        rsx! {}
    }
}
