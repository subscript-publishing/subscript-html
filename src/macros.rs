//! This crate refers to Subscript macros but also includes some misc rust macro helpers.
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
// DSL HELPERS
///////////////////////////////////////////////////////////////////////////////

pub fn value_to_string(x: impl std::fmt::Display) -> String {
    x.to_string()
}

#[macro_use]
macro_rules! html_attrs {
    () => {{
        use std::collections::HashMap;
        let mut attrs = HashMap::<String, String>::new();
        attrs
    }};
    ($($key:tt : $val:tt),* $(,)?) => {{
        use std::collections::HashMap;
        let mut attrs = HashMap::<String, String>::new();
        $({
            let key: String = $crate::macros::value_to_string($key);
            let val: String = $crate::macros::value_to_string($val);
            attrs.insert(key, val);
        })*
        attrs
    }};
}



///////////////////////////////////////////////////////////////////////////////
// TAG MACROS
///////////////////////////////////////////////////////////////////////////////

fn include_tag(env: &Env) -> TagMacro {
    let env = env.clone();
    let callback = Rc::new(move |node: &mut Node| {
        node
            .get_attr("src")
            .and_then(|src| env.try_load_text_file(src).ok())
            .map(|(path, text)| {
                (path, Node::parse_string(text))
            })
            .map(|(template_path, mut template)| {
                let mut template_env = env.clone();
                template_env.current_dir = template_path.parent().unwrap().to_owned();
                crate::frontend::apply_macros(&template_env, &mut template);
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

pub fn page_nav_tag(ctx: &Env) -> TagMacro {
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

pub fn layout_tag(ctx: &Env) -> TagMacro {
    let ctx = ctx.clone();
    TagMacro {
        tag: String::from("layout"),
        callback: MacroCallbackMut(Rc::new(move |node: &mut Node| {
            node.set_tag("div");
            node.set_attr("macro", String::from("layout"));
            // Allow for `cols` or `columns`; normalize to `columns`.
            node.get_attr("cols")
                .map(|val| {
                    node.set_attr("columns", val)
                });
        })),
    }
}

pub fn link_tag(env: &Env) -> TagMacro {
    let env = env.clone();
    TagMacro {
        tag: String::from("link"),
        callback: MacroCallbackMut(Rc::new(move |node: &mut Node| {
            let processed_attr = "ss.link.processed";
            if node.has_attr(processed_attr) {
                return ();
            }
            let sass_pipeline = |node: &mut Node, href: &str, path: &PathBuf| -> Option<String> {
                let env = env.clone();
                let sass_changed = env.changed
                    .clone()
                    .and_then(|changed| changed.extension().map(|x| x.to_owned()))
                    .map(|x| {
                        x == "sass" || x == "scss"
                    })
                    .unwrap_or(false);
                if let Some(path) = crate::frontend::cache::lookup_hash_file(&env, href) {
                    if !sass_changed {
                        return Some(path);
                    }
                }
                let mut options = grass::Options::default();
                let result = grass::from_path(
                    path.to_str().unwrap(),
                    &options,
                );
                match result {
                    Ok(contents) => {
                        crate::frontend::cache::cache_hash_file(&env, href, &contents)
                    }
                    Err(msg) => {
                        eprintln!("[warning] sass compiler failed:");
                        eprintln!("{}\n", msg);
                        None
                    }
                }
            };
            node.get_attr("href")
                .and_then(|href| {
                    let path = env.current_dir.join(&href);
                    match &(path.extension()?.to_str().unwrap())[..] {
                        "sass" | "scss" => {
                            let result = sass_pipeline(
                                node,
                                &href,
                                &path,
                            );
                            match result {
                                None => {
                                    eprintln!(
                                        "[warning] ignoring asset: {:?}",
                                        path
                                    );
                                }
                                Some(out_path) => {
                                    *node = Node::new_element(
                                        "link",
                                        html_attrs!{
                                            "href": out_path,
                                            "rel": "stylesheet",
                                        },
                                        Vec::new(),
                                    );
                                }
                            }
                            Some(())
                        }
                        "css" | _ => {
                            crate::frontend::cache::cache_file(&env, &href).map(|out_path| {
                                node.set_attr("href", out_path);
                            })
                        }
                    }
                })
                .map(|_| {
                    node.set_attr(processed_attr, String::new());
                });
        })),
    }
}

pub fn asset_glob_tag(env: &Env) -> TagMacro {
    let env = env.clone();
    TagMacro {
        tag: String::from("asset-glob"),
        callback: MacroCallbackMut(Rc::new(move |node: &mut Node| {
            let asset_nodes = node.get_attr("src")
                .map(|src| {
                    crate::frontend::cache::cache_file_glob(&env, &src)
                })
                .unwrap_or(Vec::new())
                .into_iter()
                .map(|out_path: String| {
                    Node::new_element(
                        "img",
                        html_attrs!{
                            "src": out_path,
                        },
                        Vec::new(),
                    )
                })
                .collect::<Vec<_>>();
            let mut contents = Node::Fragment(node.get_children());
            contents.eval(Rc::new(move |child: &mut Node| {
                if child.is_tag("content") {
                    *child = Node::Fragment(asset_nodes.clone());
                }
            }));
            *node = contents;
        })),
    }
}

pub fn tag_macros(env: &Env) -> Vec<TagMacro> {
    let mut items = vec![
        include_tag(env),
        subscript_deps(env),
        link_tag(&env),
        page_nav_tag(env),
        layout_tag(env),
        asset_glob_tag(&env),
        img_tag(&env),
    ];
    items.append(&mut latex_suit(env));
    items
}

