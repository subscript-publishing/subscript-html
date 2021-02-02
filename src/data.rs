use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::borrow::Cow;
use std::collections::HashSet;
use std::path::{PathBuf, Path};
use std::convert::AsRef;
use std::hash::Hash;
use serde::{Serialize, Deserialize};

use crate::parser;
use crate::frontend::Env;

pub mod utils;
pub mod css;


///////////////////////////////////////////////////////////////////////////////
// HELPERS
///////////////////////////////////////////////////////////////////////////////

pub enum Either<L, R> {
    Left(L),
    Right(R),
}


#[derive(Clone)]
pub struct MacroCallbackMut(pub Rc<dyn Fn(&mut Node)>);

impl std::fmt::Debug for MacroCallbackMut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MacroCallbackMut").finish()
    }
}


///////////////////////////////////////////////////////////////////////////////
// MACRO DATA TYPES
///////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct TagMacro {
    pub tag: String,
    pub callback: MacroCallbackMut,
}


// impl TagMacro {
//     pub fn consider(&self, env: &Env, node: &mut Node) {
        
//     }
// }


///////////////////////////////////////////////////////////////////////////////
// URL HELPER TYPE
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub enum Url {
    FilePath(PathBuf),
    Other(String),
}

impl Url {
    pub fn new(value: String) -> Self {
        let is_http = value.starts_with("http");
        if is_http {
            return Url::Other(value)
        }
        Url::FilePath(PathBuf::from(value))
    }
    pub fn map_file_path<T>(&self, f: impl Fn(&Path)->T) -> Option<T> {
        match self {
            Url::Other(_) => None,
            Url::FilePath(x) => Some(f(x))
        }
    }
}


///////////////////////////////////////////////////////////////////////////////
// HTML TREE AST
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Node {
    Element(Box<Element>),
    Text(String),
    Fragment(Vec<Node>),
}

impl Node {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }
    pub fn from_json<T: AsRef<str>>(val: T) -> Result<Self, serde_json::Error> {
        serde_json::from_str(val.as_ref())
    }
    pub fn to_html_str(&self, indent_level: usize) -> String {
        fn render_text(text: &str) -> String {
            text.to_owned()
        }
        let level = {
            if indent_level == 0 {
                String::from("")
            } else {
                std::iter::repeat(" ").take(indent_level * 2).collect::<String>()
            }
        };
        match self {
            Node::Element(element) => {
                let attrs = {
                    let mut ats = element.attrs
                        .iter()
                        .map(|(key, mut value)| {
                            if value.is_empty() {
                                return format!("{}", key);
                            }
                            format!("{}=\"{}\"", key, value)
                        })
                        .collect::<Vec<_>>();
                    if ats.is_empty() {
                        String::new()
                    } else {
                        format!(" {}", ats.join(" "))
                    }
                };
                let children_len = element.children.len();
                let children = {
                    let xs = element.children
                        .iter()
                        .map(|child| {
                            match child {
                                Node::Text(txt) => {
                                    render_text(txt)
                                }
                                _ => format!(
                                    "\n{}",
                                    child.to_html_str(indent_level + 1)
                                )
                            }
                        })
                        .collect::<Vec<_>>();
                    if xs.is_empty() {
                        String::new()
                    } else {
                        xs.join("")
                    }
                };
                let contents = {
                    // SINGLE LINE CHECKS
                    let mut single_line_mode = false;
                    let no_children = element.children.len() == 0;
                    let single_child = element.children.len() == 1;
                    let is_inline_element = utils::is_inline_tag(&element.tag);
                    if no_children || (single_child && is_inline_element) || is_inline_element {
                        single_line_mode = true;
                    }
                    if self.is_tag("p") {
                        single_line_mode = true;
                    }
                    // RENDER
                    if single_line_mode {
                        format!(
                            "{children}",
                            children=children,
                        )
                    } else {
                        // The `\n{lvl}` is for the closing tag.
                        format!(
                            "{children}\n{lvl}",
                            lvl=level,
                            children=children,
                        )
                    }
                };
                format!(
                    "{lvl}<{tag}{attrs}>{contents}</{tag}>",
                    lvl=level,
                    tag=element.tag,
                    attrs=attrs,
                    contents=contents,
                )
            }
            Node::Text(txt) => {render_text(txt)}
            Node::Fragment(xs) => {
                xs  .iter()
                    .map(|child| child.to_html_str(indent_level))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
    }
    pub fn into_fragment(self) -> Vec<Node> {
        match self {
            Node::Fragment(xs) => {xs}
            _ => vec![]
        }
    }
    pub fn only_text_children(&self) -> bool {
        self.get_children()
            .into_iter()
            .all(|x| {
                x.is_text()
            })
    }
    pub fn only_inline_children(&self) -> bool {
        self.get_children()
            .into_iter()
            .all(|x| {
                x.is_inline_node()
            })
    }
    pub fn parse_str(html_str: &str) -> Self {
        Node::Fragment(crate::parser::html::parse_html_str(html_str).payload)
    }
    pub fn parse_string(html_str: String) -> Self {
        Node::Fragment(crate::parser::html::parse_html_str(&html_str).payload)
    }
    pub fn eval(&mut self, f: Rc<dyn Fn(&mut Node)>) {
        match self {
            Node::Element(element) => {
                for child in element.children.iter_mut() {
                    child.eval(f.clone());
                }
            }
            Node::Fragment(xs) => {
                for x in xs.iter_mut() {
                    x.eval(f.clone());
                }
            }
            _ => {}
        }
        f(self);
    }
    // pub fn apply(&mut self, f: Macro) {
    //     self.eval(Rc::new(move |x| {
    //         f.eval(x)
    //     }))
    // }
    // pub fn apply_all(&mut self, macros: Vec<Macro>) {
    //     for mut m in macros {
    //         self.apply(m);
    //     }
    // }
    pub fn set_tag(&mut self, new_tag: &str) {
        match self {
            Node::Element(element) => {
                element.tag = String::from(new_tag);
            },
            _ => ()
        }
    }
    pub fn tag(&self) -> Option<String> {
        match self {
            Node::Element(element) => Some(element.tag.clone()),
            _ => None
        }
    }
    pub fn is_tag(&self, tag: &str) -> bool {
        self.tag() == Some(String::from(tag))
    }
    pub fn has_attr(&self, key: &str) -> bool {
        match self {
            Node::Element(element) => {
                element.attrs.contains_key(key)
            },
            _ => false
        }
    }
    pub fn has_attr_value(&self, key: &str, value: &str) -> bool {
        match self {
            Node::Element(element) => {
                element.attrs.get(key).map(|x| x == value).unwrap_or(false)
            },
            _ => false
        }
    }
    pub fn get_attr(&self, key: &str) -> Option<String> {
        match self {
            Node::Element(element) => {
                if let Some(key) = element.attrs.get(key) {
                    Some(key.clone())
                } else {
                    None
                }
            },
            _ => None
        }
    }
    pub fn set_attr(&mut self, key: &str, value: String) {
        match self {
            Node::Element(element) => {
                element.attrs.insert(key.to_owned(), value);
            },
            _ => ()
        }
    }
    pub fn replace_children(&mut self, new_children: Vec<Node>) {
        match self {
            Node::Element(element) => {
                element.children = new_children;
            },
            _ => ()
        }
    }
    pub fn append_children(&mut self, mut new_children: Vec<Node>) {
        match self {
            Node::Element(element) => {
                element.children.append(&mut new_children);
            },
            _ => ()
        }
    }
    pub fn get_children(&self) -> Vec<Node> {
        match self {
            Node::Element(element) => {
                element.children.clone()
            },
            _ => vec![]
        }
    }
    pub fn get_attributes(&self) -> HashMap<String, String> {
        match self {
            Node::Element(element) => {
                element.attrs.clone()
            },
            _ => Default::default()
        }
    }
    /// If this is a fragment, it returns the contents of such.
    pub fn unwrap_contents(self, tag: &str) -> Vec<Node> {
        match self {
            Node::Fragment(xs) => xs,
            Node::Element(element) if &element.tag == tag => {
                element.children
            }
            x => vec![x]
        }
    }
    pub fn normalize(self) -> Self {
        match self {
            Node::Element(mut element) => {
                let mut new_children = Vec::<Node>::new();
                for child in element.children.into_iter() {
                    match child {
                        Node::Fragment(mut xs) => {
                            for x in xs {
                                new_children.push(x.normalize())
                            }
                        }
                        node => {
                            new_children.push(node.normalize())
                        }
                    }
                }
                element.children = new_children;
                Node::Element(element)
            }
            Node::Fragment(elements) => {
                let mut new_children = Vec::<Node>::new();
                for child in elements.into_iter() {
                    match child {
                        Node::Fragment(mut xs) => {
                            for x in xs {
                                new_children.push(x.normalize())
                            }
                        }
                        node => {
                            new_children.push(node.normalize())
                        }
                    }
                }
                Node::Fragment(new_children)
            }
            node => node
        }
    }
    pub fn get_children_as_text(&self) -> Vec<String> {
        let mut texts = Vec::<String>::new();
        match self {
            Node::Text(text) => vec![text.clone()],
            _ => {
                let mut ys = self
                    .get_children()
                    .into_iter()
                    .flat_map(|x| x.get_children_as_text())
                    .collect::<Vec<_>>();
                return ys;
            }
        }
    }
    pub fn get_text_contents(&self) -> Option<String> {
        let txts = self.get_children_as_text();
        if txts.is_empty() {
            None
        } else {
            Some(txts.join("\n"))
        }
    }
    pub fn is_text(&self) -> bool {
        match self {
            Node::Text(_) => true,
            _ => false,
        }
    }
    pub fn is_element(&self) -> bool {
        match self {
            Node::Element(_) => true,
            _ => false,
        }
    }
    pub fn new_element(
        tag: &str,
        attrs: HashMap<String, String>,
        children: Vec<Node>,
    ) -> Self {
        let mut element = Element{
            tag: String::from(tag),
            styling: css::Styling::default(),
            attrs,
            children: children,
        };
        Node::Element(Box::new(element))
    }
    pub fn new_text(value: &str) -> Self {
        Node::Text(String::from(value))
    }
    pub fn is_inline_node(&self) -> bool {
        if self.get_attr("block").is_some() {
            return false;
        }
        match self {
            Node::Element(element) => {
                if utils::is_inline_tag(&element.tag) {
                    return true;
                }
                if element.tag == String::from("tex") {
                    return true;
                }
                false
            },
            Node::Fragment(xs) => {
                xs.iter().all(|x| x.is_inline_node())
            }
            Node::Text(..) => true,
        }
    }
}

impl Default for Node {
    fn default() -> Self {Node::Fragment(vec![])}
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Element {
    pub tag: String,
    #[serde(default)]
    pub styling: css::Styling,
    #[serde(default)]
    pub attrs: HashMap<String, String>,
    #[serde(default)]
    pub children: Vec<Node>,
}



///////////////////////////////////////////////////////////////////////////////
// SINGLE USE CELL
///////////////////////////////////////////////////////////////////////////////

pub struct Store<T>(Arc<Mutex<T>>);

unsafe impl<T> Send for Store<T> {}
unsafe impl<T> Sync for Store<T> {}

impl<T> Clone for Store<T> {
    fn clone(&self) -> Self {Store(self.0.clone())}
}


impl<T> std::fmt::Debug for Store<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output_ty_name = std::any::type_name::<T>();
        f.debug_struct(&format!(
            "Store(\"{}\")",
            output_ty_name,
        )).finish()
    }
}

impl<T> Store<T> {
    pub fn new(x: T) -> Store<T> {
        Store(Arc::new(Mutex::new(x)))
    }
    pub fn access<U>(&self, f: impl Fn(&T)->U) -> U {
        use std::ops::DerefMut;
        let mut lock = self.0.lock().unwrap();
        f(lock.deref_mut())
    }
    pub fn access_mut<U>(&self, mut f: impl FnMut(&mut T)->U) -> U {
        use std::ops::DerefMut;
        let mut lock = self.0.lock().unwrap();
        f(lock.deref_mut())
    }
    pub fn into_inner(self) -> Arc<Mutex<T>> {
        self.0
    }
}

