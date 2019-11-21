use html5ever::driver::ParseOpts;
use html5ever::tendril::TendrilSink;
use html5ever::tree_builder::TreeBuilderOpts;
use html5ever::{parse_document, rcdom};
use std::cell::Ref;
use std::default::Default;
use std::rc::Rc;

pub use crate::templating::*;

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

fn extract_children(children: Ref<'_, Vec<Rc<rcdom::Node>>>) -> Template {
    let mut res = Vec::new();

    for child in children.iter() {
        let children = extract_children(child.children.borrow());

        match &child.data {
            rcdom::NodeData::Element { name, .. }
                if name.local.to_string() == "html"
                    || name.local.to_string() == "head"
                    || name.local.to_string() == "body" =>
            {
                res = children;
            }
            rcdom::NodeData::Element { name, attrs, .. } => res.push(Node {
                data: NodeData::Element {
                    attributes: extract_attributes(attrs.borrow()),
                    tag: name.local.to_string(),
                },
                children,
            }),
            rcdom::NodeData::Text { contents } => res.push(Node {
                data: NodeData::Text {
                    content: contents.borrow().to_string(),
                },
                children,
            }),
            _ => panic!("Unhandled NodeData type"),
        }
    }

    res
}

pub fn extract_html(input: &mut String) -> Template {
    let dom = parse_html(input);

    extract_children(dom.document.children.borrow())
}

#[cfg(test)]
mod tests {
    use super::*;

    impl NodeData {
        //{{{
        fn tag(&self) -> Option<&String> {
            match self {
                Self::Element { tag, .. } => Some(tag),
                _ => None,
            }
        }

        fn attributes(&self) -> Option<&Attributes> {
            match self {
                Self::Element { attributes, .. } => Some(attributes),
                _ => None,
            }
        }

        fn content(&self) -> Option<&String> {
            match self {
                Self::Text { content } => Some(content),
                _ => None,
            }
        }
    } //}}}

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
        assert_eq!(dom[0].data.tag().unwrap(), &"p");
    }

    #[test]
    fn extract_html_basic_nested() {
        let dom = extract_html(&mut "<div><p></p></div>".to_string());
        assert_eq!(dom.len(), 1);
        assert_eq!(dom[0].data.tag().unwrap(), &"div");
        assert_eq!(dom[0].children.len(), 1);
        assert_eq!(dom[0].children[0].data.tag().unwrap(), &"p");
    }

    #[test]
    fn extract_html_static_attribute() {
        let dom = extract_html(&mut "<p class=\"hello\"></p>".to_string());
        assert_eq!(dom.len(), 1);
        assert_eq!(dom[0].data.attributes().unwrap()["class"].value(), &"hello");
    }

    #[test]
    fn extract_html_dynamic_attribute() {
        let dom = extract_html(&mut "<p :class=\"hello\"></p>".to_string());
        assert_eq!(dom.len(), 1);
        assert_eq!(dom[0].data.attributes().unwrap()["class"].value(), &"hello");
    }

    #[test]
    fn extract_html_handler_attribute() {
        let dom = extract_html(&mut "<p @click=\"on-click\"></p>".to_string());
        assert_eq!(dom.len(), 1);
        assert_eq!(
            dom[0].data.attributes().unwrap()["click"].value(),
            &"on-click"
        );
    }

    #[test]
    fn extract_html_text_node() {
        let dom = extract_html(&mut "<p>im a text</p>".to_string());
        assert_eq!(dom.len(), 1);
        assert_eq!(dom[0].children.len(), 1);
        assert_eq!(dom[0].children[0].data.content().unwrap(), &"im a text");
    }
}
