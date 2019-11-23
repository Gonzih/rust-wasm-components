use crate::html::*;
use crate::templating::*;
use std::collections::HashMap;
use std::fmt::Display;
use std::io;

pub type ComponentInstance = Box<dyn Component>;
type ComponentConstructor = Box<dyn Fn() -> ComponentInstance>;

// ************** Trait that enforces component specific methods **************
pub type LookupValue = Box<dyn Display>;

pub trait Lookup {
    fn lookup(&self, k: &String) -> Option<LookupValue>;
}

pub trait Component: Lookup {
    fn render(&self) -> Vec<DomNode>;
    fn handle(&mut self, message: String) -> bool;
}

pub struct ComponentRuntime {
    pub component: ComponentInstance,
    pub template: Template,
}

impl ComponentRuntime {
    pub fn render(&self) -> Vec<DomNode> {
        self.template
            .iter()
            .map(|node| node.realize(&self.component).render())
            .collect()
    }
}

pub struct ComponentWrapper {
    pub template: Template,
    pub constructor: ComponentConstructor,
}

impl ComponentWrapper {
    pub fn new(constructor: ComponentConstructor) -> Self {
        ComponentWrapper {
            constructor,
            template: vec![],
        }
    }

    pub fn construct(&self) -> ComponentRuntime {
        ComponentRuntime {
            component: (self.constructor)(),
            template: self.template.clone(),
        }
    }
}

// ************** Framework structure **************
pub struct Framework {
    components: HashMap<&'static str, ComponentWrapper>,
}

fn load_template_data(id: &'static str) -> String {
    web_sys::window()
        .expect("could not get js/window")
        .document()
        .expect("could not get js/document instance")
        .get_element_by_id(id)
        .expect(&*format!("could not find target element {}", id))
        .inner_html()
}

impl Framework {
    pub fn new() -> Self {
        Framework {
            components: HashMap::new(),
        }
    }

    pub fn register_component_wrapper(
        &mut self,
        name: &'static str,
        mut wrapper: ComponentWrapper,
        template_id: &'static str,
    ) {
        wrapper.template = extract_html(&mut load_template_data(template_id));
        self.components.insert(name, wrapper);
    }

    fn instantiate(&self, component: &'static str) -> &ComponentWrapper {
        self.components
            .get(component)
            .expect(&*format!("Unknown component {}", component))
    }

    pub fn mount(&mut self, target_id: &'static str, component: &'static str) -> io::Result<()> {
        log!("Mounting {} into #{}", component, target_id);

        let wrapper = self.instantiate(component);
        let cmp = wrapper.construct();

        let target = web_sys::window()
            .expect("could not get js/window")
            .document()
            .expect("could not get js/document instance")
            .get_element_by_id(target_id)
            .expect(&*format!("could not find target element {}", target_id));

        // clear element
        target.set_inner_html("");

        let elements = &*cmp.render();

        for element in elements {
            target
                .append_child(element)
                .expect("colud not append child");
        }

        Ok(())
    }
}
