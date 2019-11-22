extern crate html5ever;
extern crate web_sys;

use std::rc::Rc;
use wasm_bindgen::prelude::*;

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

mod framework;
mod html;
mod templating;
mod utils;

use framework::*;
use html::Template;
use templating::DomNode;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// ************** Sample component **************
struct Root {
    count: i32,
}

impl Root {
    fn new() -> Self {
        Root { count: 0 }
    }
}

// can be macro generated
impl Component for Root {
    fn render(&self) -> Vec<DomNode> {
        vec![]
    }
}

// ************** Entrypoint **************
#[wasm_bindgen]
pub fn run() {
    utils::set_panic_hook();

    let mut framework = Framework::new();

    let mut wrapper = ComponentWrapper::new(Box::new(|| Box::new(Root::new())));
    wrapper.add_lookup("count", Box::new(|| Box::new(325)));

    framework.register_component_wrapper("root", wrapper, "main");

    framework
        .mount("main-container", "root")
        .expect("could not mount component");
}
