use dioxus::prelude::*;

#[component]
pub fn Footer() -> Element {
    rsx! {
        footer {class:"footer footer-center p-4",
            aside{
                p { "Generated using "
                    a{ class:"link hover:text-white", href:"https://cargo-generate.github.io/cargo-generate/", "cargo-generate"}
                    " with template "
                    a { class:"link hover:text-white",
                        href:"https://github.com/JamiiDao/Solana-Rust-Wallet-Adapter-Templates/tree/master/dioxus", target:"_blank",
                        rel:"noopener noreferrer", "Dioxus Template" }
                }
            }
        }
    }
}
