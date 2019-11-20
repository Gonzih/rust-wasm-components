use html5ever::driver::ParseOpts;
use html5ever::tendril::TendrilSink;
use html5ever::tree_builder::TreeBuilderOpts;
use html5ever::{parse_document, rcdom};
use std::cell::{Ref, RefCell};
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

struct Node {
    tag: String,
    children: Vec<Node>,
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
            rcdom::NodeData::Element { name, .. } => res.push(Node {
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
}
