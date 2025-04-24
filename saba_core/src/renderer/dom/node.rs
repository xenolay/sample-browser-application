use core::{cell::RefCell, str::FromStr};

use alloc::{format, rc::{Rc, Weak}, string::String, vec::Vec};

use crate::renderer::html::html_tag_attribute::HtmlTagAttribute;



#[derive(Debug, Clone)]
pub struct Node {
    // 親など、自分が所有権を主張したらマズそうなものは全て Weak で宣言する
    // first_child や next_sibling は自身の drop とともに消えてほしいので Rc で宣言する
    // Node -> first_child -> last_child という推移的な所有関係は成り立ち得る
    // html parse のタイミングでこれらの reference の中身をいじりたいので RefCell でくるんで interior mutability を確保する
    pub kind: NodeKind,
    window: Weak<RefCell<Window>>,
    parent: Weak<RefCell<Node>>,
    first_child: Option<Rc<RefCell<Node>>>,
    last_child: Weak<RefCell<Node>>,
    previous_sibling: Weak<RefCell<Node>>,
    next_sibling: Option<Rc<RefCell<Node>>>
}

impl Node {
    pub fn new(kind: NodeKind) -> Self {
        Self { kind, window: Weak::new(), parent: Weak::new(), first_child: None, last_child: Weak::new(), previous_sibling: Weak::new(), next_sibling: None }
    }

    pub fn node_kind(&self) -> NodeKind {
        self.kind.clone()
    }

    pub fn set_parent(&mut self, parent: Weak<RefCell<Node>>) {
        self.parent = parent;
    }

    pub fn parent(&self) -> Weak<RefCell<Node>> {
        Weak::clone(&self.parent)
    }

    pub fn set_first_child(&mut self, first_child: Option<Rc<RefCell<Node>>>) {
        self.first_child = first_child
    }

    pub fn first_child(&self) -> Option<Rc<RefCell<Node>>> {
        self.first_child.as_ref().cloned() // こうしないと move しちゃうのだ
    }

    pub fn set_last_child(&mut self, last_child: Weak<RefCell<Node>>) {
        self.last_child = last_child
    }

    pub fn last_child(&self) -> Weak<RefCell<Node>> {
        Weak::clone(&self.last_child)
    }

    pub fn set_previous_sibling(&mut self, previous_sibling: Weak<RefCell<Node>>) {
        self.previous_sibling = previous_sibling
    }

    pub fn previous_sibling(&self) -> Weak<RefCell<Node>> {
        Weak::clone(&self.previous_sibling)
    }

    pub fn set_next_sibling(&mut self, next_sibling: Option<Rc<RefCell<Node>>>) {
        self.next_sibling = next_sibling
    }

    pub fn next_sibling(&self) -> Option<Rc<RefCell<Node>>> {
        self.next_sibling.as_ref().cloned()
    }

    pub fn get_element(&self) -> Option<Element> {
        match &self.kind {
            NodeKind::Element(element) => Some(element.clone()),
            _ => None,
        }
    }

    pub fn get_element_kind(&self) -> Option<ElementKind> {
        match &self.kind {
            NodeKind::Element(e) => Some(e.kind),
            _ => None
        }
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

// [] 4.2. Node tree | DOM Standard
// https://dom.spec.whatwg.org/#node-trees
// ----- Cited From Reference -----
// 4.2. Node tree
// Nodes are objects that implement Node. Nodes participate in a tree, which is known as the node tree.

// In practice you deal with more specific objects.

// Objects that implement Node also implement an inherited interface: Document, DocumentType, DocumentFragment, Element, CharacterData, or Attr.

// Objects that implement DocumentFragment sometimes implement ShadowRoot.

// Objects that implement Element also typically implement an inherited interface, such as HTMLAnchorElement.

// Objects that implement CharacterData also implement an inherited interface: Text, ProcessingInstruction, or Comment.

// Objects that implement Text sometimes implement CDATASection.

// Thus, every node’s primary interface is one of: Document, DocumentType, DocumentFragment, ShadowRoot, Element or an inherited interface of Element, Attr, Text, CDATASection, ProcessingInstruction, or Comment.
// --------------------------------

// 今回は全部を実装するのは無理なので、Document, Element, Text だけを実装する。
// とはいえ、primary interface として登場し得る要素について、なぜ実装（する｜しない）のか、は理解しておく意味があるだろう。

// Document: 全ての HTML document は HTML UA 上で Document DOM object として表現される以上、ないと一切の HTML document を扱えないので実装する。
// [] 3 Semantics, structure, and APIs of HTML documents | HTML Standard
// https://html.spec.whatwg.org/multipage/dom.html#dom
// ----- Cited From Reference -----
// Every XML and HTML document in an HTML UA is represented by a Document object. [DOM]
// --------------------------------

// ShadowRoot: 今回は shadow dom を実装しないのでパス。

// DocumentType: 本来 DOCTYPE トークンをパースする際に使うが、今回は DOCTYPE トークンは全て無視して全ての入力を html 文書として決め打ちでパースするのでパス。

// DocumentFragment: 親ノードのない Document Fragment を格納するものだが、なくても動くのでパス。

// Element: これなしでどうやって DOM を組めと言うのか。ということで実装する。

// Attr: 今回は Element の中に混ぜ込むことにする。

// Text: parent Element Node の Field に持っても良い気もする。

// CDATASection: 今回は html 文書以外パースしないので対応しない。
// [] CDATASection - Web APIs | MDN
// https://developer.mozilla.org/en-US/docs/Web/API/CDATASection
// ----- Cited From Reference -----
// Note: CDATA sections should not be used within HTML. They are considered comments and are not displayed.
// --------------------------------

// ProcessingInstruction: 今回は html 文書以外パースしないので対応しない。
// [] ProcessingInstruction - Web APIs | MDN
// https://developer.mozilla.org/en-US/docs/Web/API/ProcessingInstruction
// ----- Cited From Reference -----
// Warning: ProcessingInstruction nodes are only supported in XML documents, not in HTML documents. In these, a process instruction will be considered as a comment and be represented as a Comment object in the tree.
// --------------------------------

// Comment: 必須ではないのでパス。

#[derive(Debug, Clone, Eq)]
pub enum NodeKind {
    Document, // https://dom.spec.whatwg.org/#interface-document Document <- Node
    Element(Element), // https://dom.spec.whatwg.org/#interface-element Element <- Node
    Text(String), // https://dom.spec.whatwg.org/#interface-text Text <- CharacterData <- Node
}

impl PartialEq for NodeKind {
    fn eq(&self, other: &Self) -> bool {
        match &self {
            NodeKind::Document => matches!(other, NodeKind::Document),
            NodeKind::Element(e1) => match &other {
                NodeKind::Element(e2) => e1.kind == e2.kind,
                _ => false,
            },
            NodeKind::Text(_) => matches!(other, NodeKind::Text(_)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    kind: ElementKind,
    attributes: Vec<HtmlTagAttribute>,
}

impl Element {
    pub fn new(kind: &str, attributes: Vec<HtmlTagAttribute>) -> Self {
        Element { kind: ElementKind::from_str(kind).expect("failed to convert string to ElementKind"), attributes: attributes }
    }

    pub fn kind(&self) -> ElementKind {
        self.kind
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ElementKind {
    Html,
    Head,
    Style,
    Script,
    Body,
    P,
    A,
}

impl FromStr for ElementKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "html" => Ok(Self::Html),
            "head" => Ok(Self::Head),
            "style" => Ok(Self::Style),
            "script" => Ok(Self::Style),
            "body" => Ok(Self::Body),
            "p" => Ok(Self::P),
            "a" => Ok(Self::A),
            _ => Err(format!("unimplemented element name: {:?}", s)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Window {
    document: Rc<RefCell<Node>>
}

impl Window {
    pub fn new() -> Self {
        Self { document: Rc::new(RefCell::new(Node::new(NodeKind::Document))) }
    }

    pub fn document(&self) -> Rc<RefCell<Node>> {
        Rc::clone(&self.document)
    }
}
