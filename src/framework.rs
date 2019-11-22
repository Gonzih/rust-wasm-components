use crate::html::*;
use crate::templating::*;
use std::collections::HashMap;
use std::io;

type ComponentConstructor = Box<dyn Fn(Template) -> Box<dyn Component>>;

// ************** Trait that enforces component specific methods **************
pub trait Component {
    fn render(&self) -> Vec<DomNode>;
}

// ************** Framework structure **************
pub struct Framework {
    templates: HashMap<&'static str, Template>,
    components: HashMap<&'static str, ComponentConstructor>,
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

    pub fn register_template(&mut self, name: &'static str, id: &'static str) {
        self.templates
            .insert(name, extract_html(&mut load_template_data(id)));
    }

    pub fn register_component(&mut self, name: &'static str, constructor: ComponentConstructor) {
        self.components.insert(name, constructor);
    }

    pub fn register_component_template_mapping(
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

        constructor(template.clone().to_vec())
    }

    pub fn mount(&mut self, target_id: &'static str, component: &'static str) -> io::Result<()> {
        log!("Mounting {} into #{}", component, target_id);

        let cmp = self.instantiate(component);

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
