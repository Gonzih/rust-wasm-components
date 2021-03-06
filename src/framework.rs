/// Framework public API surface
use crate::html::*;
use crate::templating::*;
use crate::vdom::{SharableDomNode, VDom};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;
use std::io;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

pub type ComponentInstance = Rc<RefCell<dyn Component>>;
pub type ComponentConstructor = Box<dyn Fn() -> ComponentInstance>;

// ************** Trait that enforces component specific methods **************
pub type LookupValue = Box<dyn Display>;

pub trait Lookup {
    fn lookup(&self, k: &String) -> Option<LookupValue>;
}

pub trait Component: Lookup {
    fn render(&self) -> Vec<SharableDomNode>;
    fn handle(&mut self, message: String) -> bool;
}

pub struct Dirty {
    pub dirty: bool,
}

impl Dirty {
    pub fn new() -> Self {
        Self { dirty: true }
    }
}

pub type DirtyInstance = Rc<RefCell<Dirty>>;

pub struct ComponentRuntime {
    pub component: ComponentInstance,
    pub dirty: DirtyInstance,
    pub template: Template,
    pub vdom: VDom,
}

impl ComponentRuntime {
    pub fn render(&mut self) -> Vec<SharableDomNode> {
        self.vdom = self
            .template
            .iter()
            .map(|node| node.realize(Rc::clone(&self.component), Rc::clone(&self.dirty)))
            .collect();

        let result = self.vdom.iter().map(|vnode| vnode.to_dom()).collect();

        self.dirty.borrow_mut().dirty = true;

        result
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
            dirty: Rc::new(RefCell::new(Dirty::new())),
            vdom: vec![],
        }
    }
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

// ************** Framework structure **************

#[wasm_bindgen]
pub struct Framework {
    components: HashMap<&'static str, ComponentWrapper>,
    instances: Vec<ComponentRuntime>,
}

#[wasm_bindgen]
impl Framework {
    pub fn tick(&mut self) {
        log!("Tick in Framework");
    }
}

impl Framework {
    pub fn new() -> Self {
        Framework {
            components: HashMap::new(),
            instances: vec![],
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
        let mut runtime = wrapper.construct();

        let target = web_sys::window()
            .expect("could not get js/window")
            .document()
            .expect("could not get js/document instance")
            .get_element_by_id(target_id)
            .expect(&*format!("could not find target element {}", target_id));

        // clear element
        target.set_inner_html("");

        let elements = &*runtime.render();

        for element in elements {
            target
                .append_child(&element.borrow())
                .expect("colud not append child");
        }

        self.instances.push(runtime);

        Ok(())
    }
}
