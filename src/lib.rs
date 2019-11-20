extern crate html5ever;
extern crate web_sys;

use std::collections::HashMap;
use std::io;
use wasm_bindgen::prelude::*;

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

mod html;
mod utils;
mod vdom;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

type ComponentConstructor = Box<dyn Fn(String) -> Box<dyn Component>>;

// ************** Framework structure **************
struct Framework {
    templates: HashMap<&'static str, String>,
    components: HashMap<&'static str, ComponentConstructor>,
    component_templates: HashMap<&'static str, &'static str>,
}

impl Framework {
    fn new() -> Self {
        Framework {
            templates: HashMap::new(),
            components: HashMap::new(),
            component_templates: HashMap::new(),
        }
    }

    fn register_template(&mut self, name: &'static str, template: String) {
        self.templates.insert(name, template);
    }

    fn register_component(&mut self, name: &'static str, constructor: ComponentConstructor) {
        self.components.insert(name, constructor);
    }

    fn register_component_template_mapping(
        &mut self,
        component: &'static str,
        template: &'static str,
    ) {
        self.component_templates.insert(component, template);
    }

    fn instantiate(&self, component: &'static str) -> Box<dyn Component> {
        let constructor = self
            .components
            .get(component)
            .expect(&*format!("Unknown component {}", component));
        let template_name = self
            .component_templates
            .get(component)
            .expect(&*format!("Could not find template for {}", component));
        let template = self.templates.get(template_name).expect(&*format!(
            "Could not find template {} for component {}",
            template_name, component
        ));

        constructor(template.clone())
    }

    fn mount(&mut self, target_id: &'static str, component: &'static str) -> io::Result<()> {
        log!("Mounting {} into #{}", component, target_id);

        let cmp = self.instantiate(component);

        web_sys::window()
            .expect("could not get js/window")
            .document()
            .expect("could not get js/document instance")
            .get_element_by_id(target_id)
            .expect(&*format!("could not find target element {}", target_id))
            .set_inner_html(&*cmp.render());

        Ok(())
    }
}

// ************** Trait that enforces component specific methods **************
trait Component {
    fn render(&self) -> String;
}

// ************** Sample component **************
struct Root {
    template: String,
}

impl Root {
    fn new(template: String) -> Self {
        Root { template }
    }
}

// can be macro generated
impl Component for Root {
    fn render(&self) -> String {
        self.template.clone()
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
