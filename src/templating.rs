/// This package represents intermidiate not evaluated dom tree
/// that should be stored within a component as a templating language
use crate::framework::{ComponentInstance, DirtyInstance};
use crate::vdom::*;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone)]
pub enum Attribute {
    Static(String),
    Dynamic(String),
    Handler(String),
}

pub type Attributes = HashMap<String, Attribute>;

#[derive(Debug, Clone)]
pub enum NodeData {
    Element { tag: String, attributes: Attributes },
    Text { content: String },
}

#[derive(Debug, Clone)]
pub struct Node {
    pub data: NodeData,
    pub children: Vec<Node>,
}

impl Node {
    /// What have I done...
    pub fn realize(&self, component: ComponentInstance, dirty: DirtyInstance) -> VNode {
        let data = match &self.data {
            NodeData::Text { content } => VNodeData::Text {
                content: content.clone(),
            },
            NodeData::Element { tag, attributes } => VNodeData::Element {
                tag: tag.clone(),
                attributes: attributes
                    .iter()
                    .map(|(k, v)| {
                        let newv = match v {
                            Attribute::Static(value) => VAttribute::Attribute(value.clone()),
                            Attribute::Dynamic(value) => {
                                let v = component
                                    .borrow()
                                    .lookup(value)
                                    .expect(&*format!(
                                        "could not find key {} in a component",
                                        value
                                    ))
                                    .to_string();

                                VAttribute::Attribute(v)
                            }
                            Attribute::Handler(value) => {
                                // weak references for closure
                                // closure should not care if component object is still in memory
                                // in ideal scenario closure should not exist with component not
                                // being in memory
                                // if this is the case everything is fucked anyways, so whatever
                                let component_instance = Rc::downgrade(&component);
                                let dirty_instance = Rc::downgrade(&dirty);
                                // need to clone this and move this in to closure
                                let message_value = value.clone();

                                let closure = Closure::wrap(Box::new(move |e: web_sys::Event| {
                                    // Try to upgrade component weak reference to a strong one
                                    let cmp = component_instance.upgrade();
                                    if let Some(cmp_rc) = cmp {
                                        // execute handle method on component and get bool value
                                        let is_dirty =
                                            cmp_rc.borrow_mut().handle(message_value.clone());
                                        // Try to upgrade dirty weak reference to a strong one
                                        let dirty = dirty_instance.upgrade();

                                        // if we are able to acquire pointer to dirty config
                                        // and if we should mark our thing dirty we will set
                                        // the flag via interrior mutability of RefCell
                                        if is_dirty {
                                            if let Some(dirty) = dirty {
                                                dirty.borrow_mut().dirty = true;
                                            }
                                        }
                                    } else {
                                        log!(
                                            "Could not get instance of commponent, might be freed"
                                        );
                                    };

                                    log!("Got value {:?} and did send {}", e, message_value);
                                })
                                    as Box<dyn FnMut(_)>);

                                VAttribute::Handler(closure)
                            }
                        };

                        (k.clone(), newv)
                    })
                    .collect(),
            },
        };

        let children = self
            .children
            .iter()
            .map(|ch| ch.realize(Rc::clone(&component), Rc::clone(&dirty)))
            .collect();

        VNode { data, children }
    }
}

pub type Template = Vec<Node>;
