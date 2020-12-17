use std::rc::Rc;
use std::cell::RefCell;
use std::path::{PathBuf, Path};
use std::collections::HashMap;
use std::iter::FromIterator;
use std::convert::AsRef;
use serde::{Serialize, Deserialize};

use crate::data::*;
use crate::frontend::Env;
// use crate::utils::{
//     cache_file_dep,
// };


///////////////////////////////////////////////////////////////////////////////
// TAG MACROS
///////////////////////////////////////////////////////////////////////////////

fn include_tag(env: &Env) -> TagMacro {
    let env = env.clone();
    let callback = Rc::new(move |node: &mut Node| {
        let result = node
            .get_attr("src")
            .and_then(|src| env.try_load_text_file(src).ok())
            .and_then(|text| Some(Node::parse_string(text)))
            .map(|mut template| {
                let embedded = node.get_children();
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

pub fn tag_macros(env: &Env) -> Vec<TagMacro> {
    vec![
        include_tag(env),
    ]
}

// pub fn include_tag() -> Macro {
//     // let ctx = ctx.clone();
//     Macro::match_tag("include", Rc::new(move |node: &mut Node| {
//         let source_dir = ctx.source_dir();
//         let root_dir = ctx.root_dir.clone();
//         node.get_attr("src")
//             .and_then(|src_path_str| {
//                 let src_path = FilePath::resolve_include_path(
//                     &ctx,
//                     &src_path_str,
//                 )?;
//                 if !src_path.exists() {
//                     eprintln!("[WARNING] missing file: {}", src_path);
//                     return None;
//                 }
//                 let base: String = src_path.load_text_file();
//                 let had_doctype = base.contains("<!DOCTYPE html>");
//                 let mut base = Node::parse_str(&base);
//                 // Provision the new document:
//                 {
//                     let mut new_ctx = ctx.clone();
//                     new_ctx.source = ctx
//                         .source_dir()
//                         .unwrap()
//                         .join(&ctx.root_dir, &src_path)
//                         .unwrap();
//                     hooks::document(&new_ctx, &mut base);
//                 }
//                 let mut base = base.to_html_str(0);
//                 if had_doctype {
//                     base = format!("<!DOCTYPE html>\n{}", base);
//                 }
//                 Some(base)
//             })
//             .map(|contents| {
//                 let embeded_contents = Node::Fragment(node.get_children()).to_html_str(0);
//                 let contents = contents.replace(
//                     "<content></content>",
//                     &embeded_contents
//                 );
//                 let mut new_node = Node::parse_str(&contents);
//                 *node = new_node;
//             });
//     }))
// }
