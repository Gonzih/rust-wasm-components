/// This package represents intermidiate not evaluated dom tree
/// that should be stored within a component as a templating language
use std::collections::HashMap;

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

impl Node {
    pub fn realize(&self) -> Node {
        let data = match &self.data {
            txt @ NodeData::Text { .. } => txt.clone(),
            NodeData::Element { tag, attributes } => NodeData::Element {
                tag: tag.clone(),
                attributes: attributes
                    .iter()
                    .map(|(k, v)| {
                        let newv = match v {
                            Attribute::Dynamic(value) => Attribute::Static(value.clone()),
                            attr @ _ => attr.clone(),
                        };

                        (k.clone(), newv)
                    })
                    .collect(),
            },
        };

        let children = self.children.iter().map(|ch| ch.realize()).collect();

        Node { data, children }
    }
}

pub type Template = Vec<Node>;
