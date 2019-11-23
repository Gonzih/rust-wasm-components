/// This package represents intermidiate not evaluated dom tree
/// that should be stored within a component as a templating language
use std::collections::HashMap;
use std::ops::Deref;

use crate::framework::ComponentInstance;

#[derive(Debug, Clone)]
pub enum Attribute {
    Static(String),
    Dynamic(String),
    Handler(String),
}

impl Attribute {
    pub fn is_handler(&self) -> bool {
        match self {
            Self::Handler(_) => true,
            _ => false,
        }
    }

    pub fn is_attribute(&self) -> bool {
        !self.is_handler()
    }

    pub fn value(&self) -> &String {
        match self {
            Self::Static(value) => value,
            Self::Dynamic(value) => value,
            Self::Handler(value) => value,
        }
    }
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
