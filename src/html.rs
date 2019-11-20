use html5ever::driver::ParseOpts;
use html5ever::tendril::TendrilSink;
use html5ever::tree_builder::TreeBuilderOpts;
use html5ever::{parse_document, rcdom};
use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::default::Default;
use std::rc::Rc;

fn parse_html(input: &mut String) -> rcdom::RcDom {
    let opts = ParseOpts {
        tree_builder: TreeBuilderOpts {
            drop_doctype: true,
            ..Default::default()
        },
        ..Default::default()
    };

    parse_document(rcdom::RcDom::default(), opts)
        .from_utf8()
        .read_from(&mut input.as_bytes())
        .expect("could not parse html input")
}

#[derive(Debug)]
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

type Attributes = HashMap<String, Attribute>;

struct Node {
    pub tag: String,
    pub children: Vec<Node>,
    attributes: Attributes,
}

fn extract_attribute(attr: &html5ever::Attribute) -> (String, Attribute) {
    use Attribute::*;
    let k = attr.name.local.to_string();
    let v = attr.value.to_string();

    match k.chars().next() {
        Some(':') => (k.replacen(':', "", 1), Dynamic(v)),
        Some('@') => (k.replacen('@', "", 1), Handler(v)),
        _ => (k, Static(v)),
    }
}

fn extract_attributes(attributes: Ref<'_, Vec<html5ever::Attribute>>) -> Attributes {
    attributes.iter().map(extract_attribute).collect()
}

fn extract_children(children: Ref<'_, Vec<Rc<rcdom::Node>>>) -> Vec<Node> {
    let mut res = Vec::new();

    for child in children.iter() {
        match &child.data {
            rcdom::NodeData::Element { name, .. }
                if name.local.to_string() == "html"
                    || name.local.to_string() == "head"
                    || name.local.to_string() == "body" =>
            {
                res = extract_children(child.children.borrow())
            }
            rcdom::NodeData::Element { name, attrs, .. } => res.push(Node {
                attributes: extract_attributes(attrs.borrow()),
                tag: name.local.to_string(),
                children: extract_children(child.children.borrow()),
            }),
            _ => panic!("Unhandled NodeData type"),
        }
    }

    res
}

fn extract_html(input: &mut String) -> Vec<Node> {
    let dom = parse_html(input);

    extract_children(dom.document.children.borrow())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_html_basic() {
        let dom = parse_html(&mut "<p></p>".to_string());
        let html_data = &dom.document.children.borrow()[0].data;
        match html_data {
            rcdom::NodeData::Element { name, .. } => assert_eq!(name.local.to_string(), "html"),
            _ => panic!("Incorrect NodeData type!"),
        }
    }

    #[test]
    fn extract_html_basic() {
        let dom = extract_html(&mut "<p></p>".to_string());
        assert_eq!(dom.len(), 1);
        assert_eq!(dom[0].tag, "p".to_string());
    }

    #[test]
    fn extract_html_basic_nested() {
        let dom = extract_html(&mut "<div><p></p></div>".to_string());
        assert_eq!(dom.len(), 1);
        assert_eq!(dom[0].tag, "div".to_string());
        assert_eq!(dom[0].children.len(), 1);
        assert_eq!(dom[0].children[0].tag, "p".to_string());
    }

    #[test]
    fn extract_html_static_attribute() {
        let dom = extract_html(&mut "<p class=\"hello\"></p>".to_string());
        assert_eq!(dom.len(), 1);
        assert_eq!(dom[0].attributes["class"].value(), &"hello".to_string());
    }

    #[test]
    fn extract_html_dynamic_attribute() {
        let dom = extract_html(&mut "<p :class=\"hello\"></p>".to_string());
        assert_eq!(dom.len(), 1);
        assert_eq!(dom[0].attributes["class"].value(), &"hello".to_string());
    }

    #[test]
    fn extract_html_handler_attribute() {
        let dom = extract_html(&mut "<p @click=\"on-click\"></p>".to_string());
        assert_eq!(dom.len(), 1);
        assert_eq!(dom[0].attributes["click"].value(), &"on-click".to_string());
    }
}
