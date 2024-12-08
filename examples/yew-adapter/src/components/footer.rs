use yew::prelude::*;

#[function_component(Footer)]
pub fn footer() -> Html {
    html! {
        <footer class="footer footer-center p-4">
            <aside>
                <p> {"Generated using "}
                    <a class="link hover=text-white" href="https://cargo-generate.github.io/cargo-generate/"> {"cargo-generate"}</a>
                    {" with template "}
                    <a class="link hover:text-white"
                        href="https://github.com/JamiiDao/Solana-Rust-Wallet-Adapter-Templates/tree/master/yew" target="_blank"
                        rel="noopener noreferrer"> {"Yew Template" } </a>
                </p>
            </aside>
        </footer>
    }
}
