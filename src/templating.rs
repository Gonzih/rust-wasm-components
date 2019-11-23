/// This package represents intermidiate not evaluated dom tree
/// that should be stored within a component as a templating language
use std::collections::HashMap;
use std::ops::Deref;

use crate::framework::ComponentInstance;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

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

pub enum DomNode {
    Text(web_sys::Text),
    Element(web_sys::Element),
}

impl Deref for DomNode {
    type Target = web_sys::Node;

    fn deref(&self) -> &web_sys::Node {
        match self {
            DomNode::Text(txt) => txt.as_ref(),
            DomNode::Element(el) => el.as_ref(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct VNode {
    pub data: NodeData,
    pub children: Vec<VNode>,
}

impl VNode {
    pub fn render(&self) -> DomNode {
        let document = web_sys::window()
            .expect("could not get js/window")
            .document()
            .expect("could not get js/document instance");

        match &self.data {
            NodeData::Text { content } => {
                let txt = document.create_text_node(&content.clone());

                DomNode::Text(txt)
            }
            NodeData::Element { tag, attributes } => {
                let children: Vec<_> = self.children.iter().map(|c| c.render()).collect();

                let element = document
                    .create_element(tag)
                    .expect("could not create dom element");

                for (name, attribute) in attributes {
                    match attribute {
                        Attribute::Static(value) => element
                            .set_attribute(name, value)
                            .expect("could not set attribute"),
                        Attribute::Handler(value) => {
                            let message_value = value.clone();
                            let closure = Closure::wrap(Box::new(move |e: web_sys::Event| {
                                log!("Got value {:?} and should send {}", e, message_value);
                            })
                                as Box<dyn FnMut(_)>);
                            element
                                .add_event_listener_with_callback(
                                    name,
                                    closure.as_ref().unchecked_ref(),
                                )
                                .expect("colud not add event listener");

                            // TODO fix this memory leak!
                            closure.forget();
                        }
                        _ => (),
                    }
                }

                for child in children {
                    element
                        .append_child(&*child)
                        .expect("could not insert a child");
                }

                DomNode::Element(element)
            }
        }
    }
}

impl Node {
    pub fn realize(&self, component: &ComponentInstance) -> VNode {
        let data = match &self.data {
            txt @ NodeData::Text { .. } => txt.clone(),
            NodeData::Element { tag, attributes } => NodeData::Element {
                tag: tag.clone(),
                attributes: attributes
                    .iter()
                    .map(|(k, v)| {
                        let newv = match v {
                            Attribute::Dynamic(value) => {
                                let v = component
                                    .lookup(value)
                                    .expect(&*format!(
                                        "could not find key {} in a component",
                                        value
                                    ))
                                    .to_string();

                                Attribute::Static(v)
                            }
                            attr @ _ => attr.clone(),
                        };

                        (k.clone(), newv)
                    })
                    .collect(),
            },
        };

        let children = self
            .children
            .iter()
            .map(|ch| ch.realize(component))
            .collect();

        VNode { data, children }
    }
}

pub type Template = Vec<Node>;
