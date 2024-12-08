use wasm_bindgen::prelude::*;

mod app;
pub(crate) use app::*;

mod components;
pub(crate) use components::*;

mod views;
pub(crate) use views::*;

#[wasm_bindgen(start)]
pub fn main() {
    yew::Renderer::<App>::new().render();
}
