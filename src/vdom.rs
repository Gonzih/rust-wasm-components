use std::collections::HashMap;

#[derive(Debug)]
pub enum VAttribute {
    Static(String),
    Dynamic(String),
    Handler(String),
}

impl VAttribute {
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

pub type VAttributes = HashMap<String, VAttribute>;

pub enum VNodeData {
    Element {
        tag: String,
        attributes: VAttributes,
    },
    Text {
        content: String,
    },
}

pub struct VNode {
    pub data: VNodeData,
    pub children: Vec<VNode>,
}
