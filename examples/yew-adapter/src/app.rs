use wallet_adapter::WalletAdapter;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::Dashboard;

#[function_component(App)]
pub fn app() -> Html {
    let init_adapter = WalletAdapter::init().unwrap();

    let adapter = use_state(|| init_adapter);

    html! {
        <ContextProvider<WalletAdapter> context={(*adapter).clone()}>
            <BrowserRouter>
                <Switch<Route> render={switch} />
            </BrowserRouter>
        </ContextProvider<WalletAdapter>>

    }
}

#[derive(Clone, Routable, PartialEq)]
pub(crate) enum Route {
    #[at("/")]
    Home,
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! {<Dashboard/>},
        Route::NotFound => html! { <h1>{ "Page Not Found" }</h1> },
    }
}
