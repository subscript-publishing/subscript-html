use std::rc::Rc;
use std::cell::RefCell;
use std::path::{PathBuf, Path};
use std::collections::HashMap;
use std::iter::FromIterator;
use std::convert::AsRef;
use serde::{Serialize, Deserialize};

use crate::data::*;
use crate::frontend::Env;


///////////////////////////////////////////////////////////////////////////////
// TAG MACROS
///////////////////////////////////////////////////////////////////////////////

fn include_tag(env: &Env) -> TagMacro {
    let env = env.clone();
    let callback = Rc::new(move |node: &mut Node| {
        node
            .get_attr("src")
            .and_then(|src| env.try_load_text_file(src).ok())
            .and_then(|text| Some(Node::parse_string(text)))
            .map(|mut template| {
                crate::frontend::apply_macros(&env, &mut template);
                let mut embedded = node.get_children();
                template.eval(Rc::new(move |child: &mut Node| {
                    if child.is_tag("content") {
                        *child = Node::Fragment(embedded.clone());
                    }
                }));
                *node = template;
            });
    });
    TagMacro {
        tag: String::from("include"),
        callback: MacroCallbackMut(callback),
    }
}

pub fn img_tag(env: &Env) -> TagMacro {
    let env = env.clone();
    let processed_attr = "ss.img.processed";
    let callback = Rc::new(move |node: &mut Node| {
        node
            .get_attr("width")
            .map(|width| {
                if node.has_attr("ss.proc.width") {
                    return;
                }
                if let Some(style) = node.get_attr("style") {
                    node.set_attr("style", format!(
                        "{}; min-width: 0; max-width: {}; width: 100%;",
                        style,
                        width,
                    ));
                } else {
                    node.set_attr("style", format!(
                        ";min-width: 0; max-width: {}; width: 100%;",
                        width,
                    ));
                }
                node.set_attr("ss.proc.width", String::new());
            });
        // CACHE ASSET
        node.get_attr("src")
            .and_then(|src_path| {
                if !node.has_attr(processed_attr) {
                    let new_src = crate::frontend::cache::cache_file(&env, &src_path)?;
                    node.set_attr("src", format!(
                        "{}",
                        new_src
                    ));
                    node.set_attr(processed_attr, String::from(""));
                }
                Some(())
            });
    });
    TagMacro {
        tag: String::from("img"),
        callback: MacroCallbackMut(callback),
    }
}

pub fn latex_suit(env: &Env) -> Vec<TagMacro> {
    let ctx = env.clone();
    fn block_latex(node: &Node, value: String) -> Node {
        let mut attrs = node.get_attributes();
        attrs.insert(String::from("latex"), String::from("block"));
        Node::new_element(
            "div",
            attrs,
            vec![Node::new_text(&format!("$${}$$", value))]
        )
    }
    fn inline_latex(node: &Node, value: String) -> Node {
        let mut attrs = node.get_attributes();
        attrs.insert(String::from("latex"), String::from("inline"));
        Node::new_element(
            "span",
            attrs,
            vec![Node::new_text(&format!("\\({}\\)", value))]
        )
    }
    vec![
        TagMacro {
            tag: String::from("tex"),
            callback: MacroCallbackMut(Rc::new(|node: &mut Node| {
                node.get_text_contents()
                    .map(|text_contents| {
                        let new_node = inline_latex(node, text_contents);
                        *node = new_node;
                    });
            })),
        },
        TagMacro {
            tag: String::from("texblock"),
            callback: MacroCallbackMut(Rc::new(|node: &mut Node| {
                node.get_text_contents()
                    .map(|text_contents| {
                        *node = block_latex(node, text_contents);
                    });
            })),
        },
        TagMacro {
            tag: String::from("equation"),
            callback: MacroCallbackMut(Rc::new(|node: &mut Node| {
                node.get_text_contents()
                    .map(|text_contents| {
                        let new_node = block_latex(node, format!(
                            "\\begin{{equation}}\n\\begin{{split}}\n{txt}\n\\end{{split}}\n\\end{{equation}}",
                            txt=text_contents
                        ));
                        *node = new_node;
                    });
            })),
        },
    ]
}

pub fn subscript_deps(ctx: &Env) -> TagMacro {
    let ctx = ctx.clone();
    TagMacro {
        tag: String::from("head"),
        callback: MacroCallbackMut(Rc::new(move |node: &mut Node| {
            let deps = Node::parse_str(include_str!("../assets/deps.html"));
            node.append_children(deps.into_fragment());
        })),
    }
}

pub fn script_tag(env: &Env) -> TagMacro {
    let env = env.clone();
    TagMacro {
        tag: String::from("script"),
        callback: MacroCallbackMut(Rc::new(move |node: &mut Node| {
            let processed_key = "ss-script-processed";
            if node.has_attr(processed_key) {
                return ()
            }
            node.get_attr("src")
                .and_then(|src| {
                    crate::frontend::cache::cache_file(&env, &src)
                })
                .and_then(|new_path| {
                    node.set_attr("src", new_path);
                    node.set_attr("processed_key", String::from(""));
                    Some(())
                });
        })),
    }
}

pub fn page_nav(ctx: &Env) -> TagMacro {
    let ctx = ctx.clone();
    #[derive(Debug, Clone)]
    struct PageTree {
        route: String,
        title: String,
        sub_pages: Vec<PageTree>,
    }
    fn build_page_tree(node: &Node) -> Option<PageTree> {
        let route = node.get_attr("route")?;
        let title = node.get_attr("title")?;
        let sub_pages = node
            .get_children()
            .into_iter()
            .filter_map(|child| build_page_tree(&child))
            .collect::<Vec<_>>();
        Some(PageTree {
            route,
            title,
            sub_pages,
        })
    }
    fn page_tree_to_html(page: PageTree) -> Node {
        let children = page.sub_pages
            .clone()
            .into_iter()
            .map(|x| page_tree_to_html(x))
            .collect::<Vec<_>>();
        let empty_children = children.is_empty();
        let child_wrapper = Node::new_element(
            "ul",
            HashMap::default(),
            children,
        );
        let link = Node::new_element(
            "a",
            HashMap::from_iter(vec![
                (String::from("href"), page.route)
            ]),
            vec![Node::new_text(&page.title)]
        );
        if empty_children {
            Node::new_element(
                "li",
                HashMap::default(),
                vec![link],
            )
        } else {
            Node::new_element(
                "li",
                HashMap::default(),
                vec![link, child_wrapper],
            )
        }
    }
    TagMacro {
        tag: String::from("page-nav"),
        callback: MacroCallbackMut(Rc::new(move |node: &mut Node| {
            let pages = node
                .get_children()
                .into_iter()
                .filter_map(|x| build_page_tree(&x))
                .map(|x| page_tree_to_html(x))
                .collect::<Vec<_>>();
            *node = Node::new_element(
                "ul",
                HashMap::from_iter(vec![
                    (String::from("macro"), String::from("page-nav"))
                ]),
                pages,
            );
        })),
    }
}

pub fn tag_macros(env: &Env) -> Vec<TagMacro> {
    let mut items = vec![
        include_tag(env),
        subscript_deps(env),
        page_nav(env),
    ];
    items.append(&mut latex_suit(env));
    items
}

