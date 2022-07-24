// (Lines like the one below ignore selected Clippy rules
//  - it's useful when you want to check your code with `cargo make verify`
// but some rules are too "annoying" or are not applicable for your case.)
#![allow(clippy::wildcard_imports)]
use seed::{
    prelude::{wasm_bindgen, Orders},
    App, Url,
};

fn init(_: Url, _: &mut impl Orders<Msg>) -> Model {
    
}

#[wasm_bindgen(start)]
pub fn start() {
    // Mount the `app` to the element with the `id` "app".
    App::start("app", init, update, view);
}
