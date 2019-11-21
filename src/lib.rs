extern crate html5ever;
extern crate web_sys;

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

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// ************** Sample component **************
struct Root {
    template: Template,
}

impl Root {
    fn new(template: Template) -> Self {
        Root { template }
    }
}

// can be macro generated
impl Component for Root {
    fn render(&self) -> String {
        format!("{:?}", self.template)
    }
}

// ************** Entrypoint **************
#[wasm_bindgen]
pub fn run() {
    utils::set_panic_hook();

    let mut framework = Framework::new();
    framework.register_template("main", "<p>hello from root<p>".to_string());
    framework.register_component("root", Box::new(|tmpl| Box::new(Root::new(tmpl))));
    framework.register_component_template_mapping("root", "main");
    framework
        .mount("main-container", "root")
        .expect("could not mount component");
}
