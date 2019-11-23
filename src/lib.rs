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
mod vdom;

use framework::*;
use vdom::DomNode;

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

enum Msg {
    Increment,
}

impl Msg {
    fn from_str(message: String) -> Self {
        match message.as_ref() {
            "increment" => Self::Increment,
            _ => panic!(format!("Unknown message type {}!", message)),
        }
    }
}

// can be macro generated
impl Component for Root {
    fn render(&self) -> Vec<DomNode> {
        vec![]
    }

    fn handle(&mut self, message: String) -> bool {
        let msg = Msg::from_str(message);

        match msg {
            Msg::Increment => {
                self.count += 1;
                true
            }
        }
    }
}

impl Lookup for Root {
    fn lookup(&self, k: &String) -> Option<LookupValue> {
        match k.as_ref() {
            "count" => Some(Box::new(self.count)),
            _ => None,
        }
    }
}

// ************** Entrypoint **************
#[wasm_bindgen]
pub fn run() -> Framework {
    utils::set_panic_hook();

    let mut framework = Framework::new();

    let wrapper = ComponentWrapper::new(Box::new(|| Box::new(Root::new())));

    framework.register_component_wrapper("root", wrapper, "main");

    framework
        .mount("main-container", "root")
        .expect("could not mount component");

    framework
}
