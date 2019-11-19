extern crate html5ever;
extern crate web_sys;

mod utils;

use html5ever::driver::ParseOpts;
use html5ever::tendril::TendrilSink;
use html5ever::tree_builder::TreeBuilderOpts;
use html5ever::{parse_document, rcdom};
use std::collections::HashMap;
use std::default::Default;
use std::io;
use wasm_bindgen::prelude::*;
use web_sys::Document;

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

type ComponentConstructor = Box<dyn Fn() -> Box<dyn Component>>;

// ************** Framework structure **************
struct Framework {
    templates: HashMap<&'static str, String>,
    components: HashMap<&'static str, ComponentConstructor>,
}

impl Framework {
    fn new() -> Self {
        Framework {
            templates: HashMap::new(),
            components: HashMap::new(),
        }
    }

    fn register_template(&mut self, name: &'static str, template: String) {
        self.templates.insert(name, template);
    }

    fn register_component(&mut self, name: &'static str, constructor: ComponentConstructor) {
        self.components.insert(name, constructor);
    }

    fn mount(&mut self, target_id: &'static str, root_component: &'static str) -> io::Result<()> {
        log!("Mounting {} into #{}", root_component, target_id);
        Document::new()
            .expect("could not get js/document instance")
            .get_element_by_id(target_id)
            .expect(&*format!("could not find target element {}", target_id))
            .set_inner_html("MOUNTED!");

        Ok(())
    }
}

// ************** Trait that enforces component specific methods **************
trait Component {
    fn doathing(&self);
}

// ************** Sample component **************
struct Root {}

impl Root {
    fn new() -> Self {
        Root {}
    }
}

impl Component for Root {
    fn doathing(&self) {}
}

// ************** Entrypoint **************
#[wasm_bindgen]
pub fn run() {
    let opts = ParseOpts {
        tree_builder: TreeBuilderOpts {
            drop_doctype: true,
            ..Default::default()
        },
        ..Default::default()
    };

    let dom = parse_document(rcdom::RcDom::default(), opts)
        .from_utf8()
        .read_from(&mut "<p></p>".as_bytes())
        .unwrap();

    log!("Doc: {:#?}", dom.document);

    utils::set_panic_hook();
    let mut framework = Framework::new();
    framework.register_template("main", "<p>hello<p>".to_string());
    framework.register_component("root", Box::new(|| Box::new(Root::new())));
    framework
        .mount("framework-root", "root")
        .expect("could not mount component");
}
