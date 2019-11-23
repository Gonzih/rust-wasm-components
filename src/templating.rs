/// This package represents intermidiate not evaluated dom tree
/// that should be stored within a component as a templating language
///
use crate::framework::ComponentInstance;
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
    pub fn realize(&self, component: ComponentInstance) -> VNode {
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
                                let component_instance = Rc::downgrade(&component);
                                let message_value = value.clone();
                                let closure = Closure::wrap(Box::new(move |e: web_sys::Event| {
                                    let cmp = component_instance.upgrade();

                                    match cmp {
                                        Some(cmp_rc) => {
                                            cmp_rc.borrow_mut().handle(message_value.clone());
                                        }
                                        None => log!(
                                            "Could not get instance of commponent, might be freed"
                                        ),
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
            .map(|ch| ch.realize(Rc::clone(&component)))
            .collect();

        VNode { data, children }
    }
}

pub type Template = Vec<Node>;
