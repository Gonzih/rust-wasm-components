/// This package should represent virtual dom structures and diffing and changeset generation
/// functionality
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

pub type SharableDomNode = Rc<RefCell<DomNode>>;

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

pub type HandlerClosure = Closure<dyn FnMut(web_sys::Event)>;

pub enum VAttribute {
    Attribute(String),
    Handler(HandlerClosure),
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

pub type VDom = Vec<VNode>;

pub struct VNode {
    pub data: VNodeData,
    pub children: Vec<VNode>,
}

impl VNode {
    pub fn to_dom(&self) -> SharableDomNode {
        let document = web_sys::window()
            .expect("could not get js/window")
            .document()
            .expect("could not get js/document instance");

        match &self.data {
            VNodeData::Text { content } => {
                let txt = document.create_text_node(&content.clone());

                Rc::new(RefCell::new(DomNode::Text(txt)))
            }
            VNodeData::Element { tag, attributes } => {
                let element = document
                    .create_element(tag)
                    .expect("could not create dom element");

                for (name, attribute) in attributes {
                    match attribute {
                        VAttribute::Attribute(value) => element
                            .set_attribute(name, value)
                            .expect("could not set attribute"),
                        VAttribute::Handler(closure) => {
                            element
                                .add_event_listener_with_callback(
                                    name,
                                    closure.as_ref().unchecked_ref(),
                                )
                                .expect("colud not add event listener");
                        }
                    }
                }

                let children: Vec<_> = self.children.iter().map(|c| c.to_dom()).collect();

                for child in children {
                    element
                        .append_child(&*(child.borrow()))
                        .expect("could not insert a child");
                }

                Rc::new(RefCell::new(DomNode::Element(element)))
            }
        }
    }
}
