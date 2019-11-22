use crate::html::*;
use crate::templating::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;
use std::io;
use std::rc::Rc;

type ComponentInstance = Box<dyn Component>;
type ComponentConstructor = Box<dyn Fn() -> ComponentInstance>;

// ************** Trait that enforces component specific methods **************
pub trait Component {
    fn render(&self) -> Vec<DomNode>;
}

pub type LookupValue = Box<dyn Display>;
pub type LookupFn = Box<dyn Fn() -> LookupValue>;
pub type Lookups = Rc<RefCell<HashMap<&'static str, LookupFn>>>;

pub struct ComponentRuntime {
    pub component: ComponentInstance,
    pub lookups: Lookups,
    pub template: Template,
}

impl ComponentRuntime {
    pub fn render(&self) -> Vec<DomNode> {
        self.template
            .iter()
            .map(|node| node.realize(&self.lookups).render())
            .collect()
    }
}

pub struct ComponentWrapper {
    pub template: Template,
    pub constructor: ComponentConstructor,
    pub lookups: Lookups,
}

impl ComponentWrapper {
    pub fn new(constructor: ComponentConstructor) -> Self {
        ComponentWrapper {
            constructor,
            template: vec![],
            lookups: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn add_lookup(&mut self, k: &'static str, f: LookupFn) {
        self.lookups.borrow_mut().insert(k, f);
    }

    pub fn construct(&self) -> ComponentRuntime {
        ComponentRuntime {
            component: (self.constructor)(),
            lookups: Rc::clone(&self.lookups),
            template: self.template.clone(),
        }
    }
}

// ************** Framework structure **************
pub struct Framework {
    templates: HashMap<&'static str, Template>,
    components: HashMap<&'static str, ComponentWrapper>,
    component_templates: HashMap<&'static str, &'static str>,
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
            templates: HashMap::new(),
            components: HashMap::new(),
            component_templates: HashMap::new(),
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
