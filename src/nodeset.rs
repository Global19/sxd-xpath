use std::collections::HashMap;
use std::iter::FromIterator;
use std::slice::Iter;
use std::vec;

use document::QName;
use document::dom4;

use super::EvaluationContext;

macro_rules! unpack(
    ($enum_name:ident, $name:ident, $wrapper:ident, dom4::$inner:ident) => (
        impl<'d> $enum_name<'d> {
            pub fn $name(self) -> Option<dom4::$inner<'d>> {
                match self {
                    $enum_name::$wrapper(n) => Some(n),
                    _ => None,
                }
            }
        }
    )
);

macro_rules! conversion_trait(
    ($tr_name:ident, $method:ident, $res_type:ident,
        { $(dom4::$leaf_type:ident => Node::$variant:ident),* }
    ) => (
        pub trait $tr_name<'d> {
            fn $method(self) -> $res_type<'d>;
        }

        impl<'d> $tr_name<'d> for $res_type<'d> {
            fn $method(self) -> $res_type<'d> {
                self
            }
        }

        $(impl<'d> $tr_name<'d> for dom4::$leaf_type<'d> {
            fn $method(self) -> $res_type<'d> {
                Node::$variant(self)
            }
        })*
    )
);

#[derive(Copy,Clone,PartialEq,Eq,Hash,Debug)]
pub enum Node<'d> {
    Root(dom4::Root<'d>),
    Element(dom4::Element<'d>),
    Attribute(dom4::Attribute<'d>),
    Text(dom4::Text<'d>),
    Comment(dom4::Comment<'d>),
    ProcessingInstruction(dom4::ProcessingInstruction<'d>),
}

unpack!(Node, root, Root, dom4::Root);
unpack!(Node, element, Element, dom4::Element);
unpack!(Node, attribute, Attribute, dom4::Attribute);
unpack!(Node, text, Text, dom4::Text);
unpack!(Node, comment, Comment, dom4::Comment);
unpack!(Node, processing_instruction, ProcessingInstruction, dom4::ProcessingInstruction);

impl<'d> Node<'d> {
    pub fn document(&self) -> &'d dom4::Document<'d> {
        use self::Node::*;
        match self {
            &Root(n)                  => n.document(),
            &Element(n)               => n.document(),
            &Attribute(n)             => n.document(),
            &Text(n)                  => n.document(),
            &Comment(n)               => n.document(),
            &ProcessingInstruction(n) => n.document(),
        }
    }

    pub fn expanded_name(&self) -> Option<QName<'d>> {
        use self::Node::*;
        match self {
            &Root(_)                  => None,
            &Element(n)               => Some(n.name()),
            &Attribute(n)             => Some(n.name()),
            &Text(_)                  => None,
            &Comment(_)               => None,
            &ProcessingInstruction(n) => Some(QName::new(n.target())),
        }
    }

    pub fn parent(&self) -> Option<Node<'d>> {
        use self::Node::*;
        match self {
            &Root(_)                  => None,
            &Element(n)               => n.parent().map(|n| n.to_node()),
            &Attribute(n)             => n.parent().map(|n| n.to_node()),
            &Text(n)                  => n.parent().map(|n| n.to_node()),
            &Comment(n)               => n.parent().map(|n| n.to_node()),
            &ProcessingInstruction(n) => n.parent().map(|n| n.to_node()),
        }
    }

    pub fn children(&self) -> Vec<Node<'d>> {
        use self::Node::*;
        match self {
            &Root(n)                  => n.children().iter().map(|n| n.to_node()).collect(),
            &Element(n)               => n.children().iter().map(|n| n.to_node()).collect(),
            &Attribute(_)             => Vec::new(),
            &Text(_)                  => Vec::new(),
            &Comment(_)               => Vec::new(),
            &ProcessingInstruction(_) => Vec::new(),
        }
    }

    pub fn preceding_siblings(&self) -> Vec<Node<'d>> {
        use self::Node::*;
        match self {
            &Root(_)                  => Vec::new(),
            &Element(n)               => n.preceding_siblings().iter().rev().map(|n| n.to_node()).collect(),
            &Attribute(_)             => Vec::new(),
            &Text(n)                  => n.preceding_siblings().iter().rev().map(|n| n.to_node()).collect(),
            &Comment(n)               => n.preceding_siblings().iter().rev().map(|n| n.to_node()).collect(),
            &ProcessingInstruction(n) => n.preceding_siblings().iter().rev().map(|n| n.to_node()).collect(),
        }
    }

    pub fn following_siblings(&self) -> Vec<Node<'d>> {
        use self::Node::*;
        match self {
            &Root(_)                  => Vec::new(),
            &Element(n)               => n.following_siblings().iter().map(|n| n.to_node()).collect(),
            &Attribute(_)             => Vec::new(),
            &Text(n)                  => n.following_siblings().iter().map(|n| n.to_node()).collect(),
            &Comment(n)               => n.following_siblings().iter().map(|n| n.to_node()).collect(),
            &ProcessingInstruction(n) => n.following_siblings().iter().map(|n| n.to_node()).collect(),
        }
    }
}

conversion_trait!(ToNode, to_node, Node, {
    dom4::Root => Node::Root,
    dom4::Element => Node::Element,
    dom4::Attribute => Node::Attribute,
    dom4::Text => Node::Text,
    dom4::Comment => Node::Comment,
    dom4::ProcessingInstruction => Node::ProcessingInstruction
});

impl<'d> ToNode<'d> for dom4::ChildOfRoot<'d> {
    fn to_node(self) -> Node<'d> {
        use self::Node::*;
        match self {
            dom4::ChildOfRoot::Element(n)               => Element(n),
            dom4::ChildOfRoot::Comment(n)               => Comment(n),
            dom4::ChildOfRoot::ProcessingInstruction(n) => ProcessingInstruction(n),
        }
    }
}

impl<'d> ToNode<'d> for dom4::ChildOfElement<'d> {
    fn to_node(self) -> Node<'d> {
        use self::Node::*;
        match self {
            dom4::ChildOfElement::Element(n)               => Element(n),
            dom4::ChildOfElement::Text(n)                  => Text(n),
            dom4::ChildOfElement::Comment(n)               => Comment(n),
            dom4::ChildOfElement::ProcessingInstruction(n) => ProcessingInstruction(n),
        }
    }
}

impl<'d> ToNode<'d> for dom4::ParentOfChild<'d> {
    fn to_node(self) -> Node<'d> {
        use self::Node::*;
        match self {
            dom4::ParentOfChild::Root(n)    => Root(n),
            dom4::ParentOfChild::Element(n) => Element(n),
        }
    }
}

/// A collection of nodes
#[derive(PartialEq,Debug,Clone)]
pub struct Nodeset<'d> {
    nodes: Vec<Node<'d>>,
}

impl<'d> Nodeset<'d> {
    pub fn new() -> Nodeset<'d> {
        Nodeset { nodes: Vec::new() }
    }

    pub fn add<N : ToNode<'d>>(&mut self, node: N) {
        self.nodes.push(node.to_node());
    }

    pub fn iter(&self) -> Iter<Node<'d>> {
        self.nodes.iter()
    }

    pub fn add_nodeset(& mut self, other: &Nodeset<'d>) {
        self.nodes.push_all(&other.nodes);
    }

    pub fn size(&self) -> usize {
        self.nodes.len()
    }

    pub fn into_iter(self) -> vec::IntoIter<Node<'d>> {
        self.nodes.into_iter()
    }

    pub fn document_order_first(&self) -> Option<Node<'d>> {
        let doc = match self.nodes.first() {
            Some(n) => n.document(),
            None => return None,
        };

        let mut idx = 0;
        let mut stack = vec![doc.root().to_node()];
        let mut order: HashMap<Node, usize> = HashMap::new();

        // Rebuilding this each time cannot possibly be performant,
        // but I want to see how widely used this is first before
        // picking an appropriate caching point.

        while let Some(n) = stack.pop() {
            order.insert(n, idx);
            idx += 1;
            let c = n.children();

            stack.extend(c.into_iter().rev());

            if let Node::Element(e) = n {
                // TODO: namespaces
                stack.extend(e.attributes().into_iter().map(|a| a.to_node()));
            }
        }

        self.nodes.iter().min_by(|&&n| order[n]).map(|n| *n)
    }
}

impl<'a, 'd : 'a> FromIterator<EvaluationContext<'a, 'd>> for Nodeset<'d> {
    fn from_iter<T>(iterator: T) -> Nodeset<'d>
        where T: Iterator<Item=EvaluationContext<'a, 'd>>
    {
        let mut ns = Nodeset::new();
        for n in iterator { ns.add(n.node) };
        ns
    }
}

#[cfg(test)]
mod test {
    use document::Package;

    use super::Node::{
        Attribute,
        Comment,
        Element,
        ProcessingInstruction,
        Root,
        Text,
    };
    use super::{Nodeset,ToNode};

    #[test]
    fn nodeset_can_include_all_node_types() {
        let package = Package::new();
        let doc = package.as_document();
        let mut nodes = Nodeset::new();

        let r = doc.root();
        let e = doc.create_element("element");
        let a = e.set_attribute_value("name", "value");
        let t = doc.create_text("text");
        let c = doc.create_comment("comment");
        let p = doc.create_processing_instruction("pi", None);

        nodes.add(r);
        nodes.add(e);
        nodes.add(a);
        nodes.add(t);
        nodes.add(c);
        nodes.add(p);

        let node_vec: Vec<_> = nodes.iter().collect();

        assert_eq!(6, node_vec.len());
        assert_eq!(node_vec[0], &Root(r));
        assert_eq!(node_vec[1], &Element(e));
        assert_eq!(node_vec[2], &Attribute(a));
        assert_eq!(node_vec[3], &Text(t));
        assert_eq!(node_vec[4], &Comment(c));
        assert_eq!(node_vec[5], &ProcessingInstruction(p));
    }

    #[test]
    fn nodesets_can_be_combined() {
        let package = Package::new();
        let doc = package.as_document();

        let mut all_nodes = Nodeset::new();
        let mut nodes1 = Nodeset::new();
        let mut nodes2 = Nodeset::new();

        let e1 = doc.create_element("element1");
        let e2 = doc.create_element("element2");

        all_nodes.add(e1);
        all_nodes.add(e2);

        nodes1.add(e1);
        nodes2.add(e2);

        nodes1.add_nodeset(&nodes2);

        assert_eq!(all_nodes, nodes1);
    }

    #[test]
    fn nodeset_knows_first_node_in_document_order() {
        let package = Package::new();
        let doc = package.as_document();

        let c1 = doc.create_comment("1");
        let c2 = doc.create_comment("2");
        doc.root().append_child(c1);
        doc.root().append_child(c2);

        let nodes = nodeset![c2, c1];

        assert_eq!(Some(c1.to_node()), nodes.document_order_first());
    }

    #[test]
    fn attributes_come_before_children_in_document_order() {
        let package = Package::new();
        let doc = package.as_document();

        let parent = doc.create_element("parent");
        let attr = parent.set_attribute_value("a", "v");
        let child = doc.create_element("child");

        doc.root().append_child(parent);
        parent.append_child(child);

        let nodes = nodeset![child, attr];

        assert_eq!(Some(attr.to_node()), nodes.document_order_first());
    }
}
