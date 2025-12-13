use leptos::{mount::mount_to_body, prelude::*};

fn main() {
    mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let (count, set_count) = signal(0);

    view! {
        <button
            on:click=move |_| {
                *set_count.write() += 1;
            }
            class:red=move || count.get() % 2 == 1
        >
            "Click me"
        </button>
        <p class:red=move || count.get() % 2 == 1>"Double count: ", {move || count.get() * 2}</p>
    }
}

#[component]
fn ProgressBar(
    #[prop(default = 100)] max: u16,
    progress: impl Fn() -> i32 + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <progress max=max value=progress />
        // Add a line-break to avoid overlap
        <br />
    }
}
