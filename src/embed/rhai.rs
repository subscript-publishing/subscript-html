use std::sync::{Arc, Mutex};
use std::any::Any;
use std::rc::Rc;
use std::cell::RefCell;
use std::path::{PathBuf, Path};
use std::collections::{HashMap, HashSet};
use std::convert::AsRef;
use serde::{Serialize, Deserialize};
use rhai::{
    Engine,
    EvalAltResult,
    RegisterFn,
    AST,
    Scope,
    Dynamic,
    Map,
    Array,
};
use rhai::plugin::*;
use crate::data::{Node, Store};

///////////////////////////////////////////////////////////////////////////////
// EXTERNAL API
///////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct RhaiEnv {
    engine: Rc<RefCell<Engine>>,
    tag_macros: HashMap<String, Rc<AST>>,
}

// impl RhaiEnv {
//     pub fn new<P: AsRef<Path>>(path: P) -> Self {
//         let path = path.as_ref().to_owned();

//         // let engine = Rc::new(RefCell::new(Engine::default()));
//         // let tag_macros = HashMap::default();
//         // RhaiEnv {engine, tag_macros}
//         unimplemented!()
//     }
// }


///////////////////////////////////////////////////////////////////////////////
// PRELUDE
///////////////////////////////////////////////////////////////////////////////


#[export_module]
mod subscript_prelude {
    pub fn new_rand_id() -> String {
        format!(
            "uid{}",
            rand::random::<u64>()
        )
    }
}


///////////////////////////////////////////////////////////////////////////////
// INTERNALS
///////////////////////////////////////////////////////////////////////////////

pub fn build<P: AsRef<Path>>(
    engine: &mut Engine,
    src_path: P,
) -> Result<AST, Box<EvalAltResult>> {
    let src_path = src_path.as_ref().to_owned();
    engine.compile_file(src_path)
}

pub fn parse_dynamic<O: serde::de::DeserializeOwned>(
    value: Dynamic
) -> Result<O, Box<EvalAltResult>> {
    match ::rhai::serde::from_dynamic::<O>(&value) {
        Ok(x) => Ok(x),
        Err(msg) => {
            let output_ty_name = std::any::type_name::<O>();
            let value: String = ::rhai::serde::from_dynamic::<serde_json::Value>(&value)
                .map_err(|err| {
                    eprintln!("user plugin: malformed json");
                    err
                })
                .map(|value: serde_json::Value| {
                    serde_json::to_string_pretty(&value).unwrap()
                })?;
            eprintln!(
                "[user plugin] failed to parse output as `{}`:",
                output_ty_name,
            );
            eprintln!("{}\n", value);
            eprintln!("{}", msg);
            return Err(msg)
        }
    }
}

pub fn call_fn1<I: Serialize, O: serde::de::DeserializeOwned>(
    engine: &Engine,
    scope: &mut Scope,
    ast: &AST,
    name: &str,
    arg: &I,
) -> Result<O, Box<EvalAltResult>> {
    let arg: Dynamic = ::rhai::serde::to_dynamic(arg)?;
    let result = engine.call_fn::<_, Dynamic>(scope, &ast, "apply", (arg,))?;
    parse_dynamic::<O>(result)
}

pub fn eval_plugin_module<P: AsRef<Path>>(
    engine: &mut Engine,
    module_name: &str,
    module_path: P,
) -> Result<(), Box<EvalAltResult>> {
    let scope = Scope::default();
    let ast = build(engine, module_path)?;
    let module = Module::eval_ast_as_new(
        scope,
        &ast,
        &engine,
    )?;
    let plugins = module
        .get_var_value::<Dynamic>("plugins")
        .and_then(|x| x.try_cast::<Array>())
        .unwrap_or(Vec::new())
        .into_iter()
        .filter_map(|x| x.try_cast::<Map>())
        .filter_map(|plugin: Map| {
            let type_ = plugin
                .get("type")
                .and_then(|x| parse_dynamic::<String>(x.clone()).ok())?;
            let tag_ = plugin
                .get("tag")
                .and_then(|x| parse_dynamic::<String>(x.clone()).ok())?;
            let trans = plugin
                .get("trans")
                .and_then(|x| parse_dynamic::<String>(x.clone()).ok())?;
            if type_ != String::from("tag_macro") {
                return None;
            }
            println!("plugin: {:?}", plugin);
            Some(())
        })
        .collect::<Vec<_>>();
    Ok(())
}


struct TagMacro {
    tag: String,
    trans: String,
    ast: AST,
}

pub struct RhaiSubSystem {
    engine: Engine,
    plugins: HashMap<String, TagMacro>,
}

impl RhaiSubSystem {
    pub fn new(globs: Vec<String>) -> Self {
        let mut engine = Engine::new();
        let module = exported_module!(subscript_prelude);
        engine.load_package(module);
        let plugins = crate::frontend::io::expand_globs(globs)
            .into_iter()
            .filter_map(|path| {
                let ast = match engine.compile_file(path.clone()) {
                    Ok(x) => Some(x),
                    Err(msg) => {
                        eprintln!("[rhai error] failed to compile {:?}:", path);
                        eprintln!("{}", msg);
                        None
                    }
                }?;
                let scope = Scope::default();
                let module = Module::eval_ast_as_new(
                    scope,
                    &ast,
                    &engine,
                ).ok()?;
                let plugins = module
                    .get_var_value::<Dynamic>("plugins")
                    .and_then(|x| x.try_cast::<Array>())
                    .unwrap_or(Vec::new())
                    .into_iter()
                    .filter_map(|x| x.try_cast::<Map>())
                    .filter_map(|plugin: Map| {
                        let type_ = plugin
                            .get("type")
                            .and_then(|x| parse_dynamic::<String>(x.clone()).ok())
                            .map(|x| x.replace("-", "_"))?;
                        let tag = plugin
                            .get("tag")
                            .and_then(|x| parse_dynamic::<String>(x.clone()).ok())?;
                        let trans = plugin
                            .get("trans")
                            .and_then(|x| parse_dynamic::<String>(x.clone()).ok())?;
                        if type_ != String::from("tag_macro") {
                            return None;
                        }
                        Some(TagMacro {
                            tag,
                            trans,
                            ast: ast.clone(),
                        })
                    })
                    .collect::<Vec<_>>();
                Some(plugins)
            })
            .flat_map(|xs| xs)
            .map(|x| (x.tag.clone(), x))
            .collect::<HashMap<_, _>>();
        RhaiSubSystem {engine, plugins}
    }
    pub fn get_macro_tag_names(&self) -> HashSet<String> {
        self.plugins
            .keys()
            .map(|x| x.to_owned())
            .collect::<HashSet<_>>()
    }
    pub fn consider_node(&self, node: &mut Node) {
        let node_tag = node.tag();
        let mut apply_macro = |plugin: &TagMacro| -> Result<(), Box<EvalAltResult>> {
            let mut scope = Scope::new();
            *node = call_fn1::<_, Node>(
                &self.engine,
                &mut scope,
                &plugin.ast,
                &plugin.trans,
                &node.clone(),
            )?;
            Ok(())
        };
        if let Some(node_tag) = node_tag {
            self.plugins
                .iter()
                .for_each(|(macro_tag, plugin)| {
                    if &node_tag == macro_tag {
                        apply_macro(plugin);
                    }
                })
        }
    }
}


