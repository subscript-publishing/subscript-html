use std::rc::Rc;
use std::collections::{HashSet, HashMap};
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use serde::{Serialize, Deserialize};
use std::iter::FromIterator;

///////////////////////////////////////////////////////////////////////////////
// CURRENT ENV
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq)]
pub struct Env {
    // pub root_dir: PathBuf,
    pub current_dir: PathBuf,
    pub output_dir: PathBuf,
    pub base_url: Option<String>,
}

impl Env {
    pub fn try_load_text_file<P: AsRef<Path>>(&self, path: P) -> Result<String, ()> {
        let mut path = path.as_ref().to_owned();
        if path.starts_with(".") || path.starts_with("..") {
            path = self.current_dir.join(path);
        }
        io::try_load_text_file(path)
    }
}

///////////////////////////////////////////////////////////////////////////////
// FRONTEND IO HELPERS
///////////////////////////////////////////////////////////////////////////////

pub mod io {
    use super::*;
    
    pub fn load_text_file<P: AsRef<Path>>(path: P) -> String {
        match try_load_text_file(&path) {
            Ok(x) => x,
            Err(_) => {
                let path = path.as_ref().to_str();
                eprintln!("missing file {:?}", path);
                panic!()
            }
        }
    }
    pub fn load_binary_file<P: AsRef<Path>>(path: P) -> Vec<u8> {
        match try_load_binary_file(&path) {
            Ok(x) => x,
            Err(_) => {
                let path = path.as_ref().to_str();
                eprintln!("missing file {:?}", path);
                panic!()
            }
        }
    }
    pub fn try_load_text_file<P: AsRef<Path>>(path: P) -> Result<String, ()> {
        std::fs::read(path.as_ref())
            .map_err(|_| ())
            .and_then(|x| String::from_utf8(x).map_err(|_| ()))
    }
    pub fn try_load_binary_file<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, ()> {
        match std::fs::read(path.as_ref()) {
            Ok(x) => Ok(x),
            Err(_) => Err(())
        }
    }
}


///////////////////////////////////////////////////////////////////////////////
// MANIFEST FILE
///////////////////////////////////////////////////////////////////////////////

pub(crate) mod manifest_format {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Project {
        pub(crate) pages: Vec<String>,
        pub(crate) root: PathBuf,
        pub(crate) output_dir: PathBuf,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct OnStartup {
        pub(crate) open_browser: bool,
    }

    #[derive(Serialize, Deserialize, Debug, Clone, Default)]
    pub struct OnEvents {
        pub(crate) startup: Option<OnStartup>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Manifest {
        pub(crate) project: Project,
        #[serde(default)]
        pub(crate) on: OnEvents,
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Manifest {
        let path = path.as_ref().to_owned();
        match toml::from_str::<Manifest>(&io::load_text_file(&path)) {
            Ok(x) => x,
            Err(msg) => {
                let path = path.to_str();
                eprintln!("failed to load manifest file from {:?}", path);
                panic!("{:#?}", msg);
            }
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// CONFIG
///////////////////////////////////////////////////////////////////////////////

/// Normalized version of the raw manifest format.
pub mod config {
    use super::*;
    #[derive(Debug, Clone)]
    pub struct Config {
        pub input_files: Vec<PathBuf>,
        pub root: PathBuf,
        pub output_dir: PathBuf,
    }
    impl Config {
        /// Only call this once.
        /// Sets the current working dir to `normalized_root_dir`.
        pub fn init<P: AsRef<Path>>(manifest_path: P) -> Self {
            let manifest_path = manifest_path.as_ref().to_owned();
            let manifest_root_dir = manifest_path.parent().unwrap();
            // let pwd = std::env::current_dir().unwrap();
            let manifest = manifest_format::load(&manifest_path);
            let normalized_root_dir = manifest_root_dir.join(&manifest.project.root);
            std::env::set_current_dir(&normalized_root_dir).unwrap();
            let input_files = manifest.project.pages
                .clone()
                .into_iter()
                .flat_map(|x: String| -> Vec<PathBuf> {
                    glob::glob(&x)
                        .unwrap()
                        .filter_map(Result::ok)
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            let output_dir = manifest.project.output_dir.clone();
            if !output_dir.exists() {
                std::fs::create_dir_all(&output_dir);
            }
            Config {
                input_files,
                root: manifest.project.root.clone(),
                output_dir,
            }
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// CLI
///////////////////////////////////////////////////////////////////////////////

pub mod cli {
    use super::*;
    /// The Subscript CLI frontend. 
    #[derive(Debug, StructOpt)]
    pub enum Cli {
        /// Compile the given HTML files.
        Compile {
            /// Explicit path to the manifest file
            #[structopt(long, default_value="./subscript.toml")]
            manifest: String,
        },
    }
}

///////////////////////////////////////////////////////////////////////////////
// CACHE
///////////////////////////////////////////////////////////////////////////////

pub mod cache {
    use super::*;
    use crate::data::utils;
    use std::collections::HashMap;
    use std::rc::Rc;
    use std::sync::{Arc, Mutex};
    use std::borrow::Cow;
    use std::collections::HashSet;
    use std::path::{PathBuf, Path};
    pub fn cache_file<P: AsRef<Path>>(env: &Env, source_path: &P) -> Option<String> {
        GLOBAL_CACHE.cache_file(env, &source_path.as_ref().to_owned())
    }
    pub fn cache_inline_text<P: AsRef<Path>>(env: &Env, source_path: &P) -> Option<String> {
        GLOBAL_CACHE.cache_inline_text(env, &source_path.as_ref().to_owned())
    }
    type SourcePath = PathBuf;
    type OutputPath = PathBuf;
    struct Cache(Arc<Mutex<HashMap<SourcePath, CachedItem>>>);
    lazy_static! {
        /// This is an example for using doc comment attributes
        static ref GLOBAL_CACHE: Cache = Cache::new();
    }
    #[derive(Debug, Clone)]
    enum CachedItem {
        /// For files that are loaded/feteched at runtime.
        FilePath {output: String},
        /// For file-contents that are inlined in the HTML tree.
        InlineText {contents: String},
    }
    impl Cache {
        fn new() -> Self {
            Cache(Arc::new(Mutex::new(HashMap::default())))
        }
        fn lookup(&self, path: &PathBuf) -> Option<CachedItem> {
            self.0.lock().unwrap().get(path).map(|x| x.clone())
        }
        fn insert(&self, source_path: &PathBuf, cached_file: CachedItem) {
            self.0.lock().unwrap().insert(source_path.clone(), cached_file);
        }
        fn cache_file(&self, env: &Env, source_path: &PathBuf) -> Option<String> {
            if let Some(CachedItem::FilePath{output}) = self.lookup(source_path) {
                return Some(output)
            }
            let out_path = utils::cache_file_dep(env, source_path)?;
            let cached_file = CachedItem::FilePath {
                output: out_path.clone(),
            };
            self.insert(source_path, cached_file);
            Some(out_path)
        }
        fn cache_inline_text(&self, _: &Env, source_path: &PathBuf) -> Option<String> {
            if let Some(CachedItem::InlineText{contents}) = self.lookup(source_path) {
                return Some(contents)
            }
            // LOAD FILE
            if let Ok(contents) = crate::frontend::io::try_load_text_file(source_path) {
                let cached_file = CachedItem::InlineText {
                    contents: contents.clone(),
                };
                self.insert(source_path, cached_file);
                Some(contents)
            } else {
                eprintln!(
                    "[warning] ignoring asset: {:?}",
                    source_path,
                );
                None
            }
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// BUILD
///////////////////////////////////////////////////////////////////////////////

pub fn apply_macros(env: &Env, html: &mut crate::data::Node) {
    use crate::data::Node;
    let env = env.clone();
    html.eval(Rc::new(move |node: &mut Node| {
        node.tag()
            .map({
                let env = env.clone();
                move |node_tag| {
                    for tag_macro in crate::macros::tag_macros(&env) {
                        if tag_macro.tag == node_tag {
                            (tag_macro.callback.0)(node);
                        }
                    }
                }
            });
    }));
}


/// Given e.g
/// - pages
/// - pages/sample
/// - pages/sample
/// - pages/sample
/// - pages
/// The intersection will be 'pages'.
pub fn intersect(pages: Vec<PathBuf>, output_dir: PathBuf) -> Option<PathBuf> {
    let mut base_path = Option::<PathBuf>::None;
    let intersection = pages
        .clone()
        .into_iter()
        .filter_map(|path| {
            path
                .strip_prefix(&output_dir)
                .map(|x| x.to_owned())
                .unwrap_or(path)
                .parent()
                .map(|x| x.to_owned())
        })
        .fold(vec![], |prev, parent| -> Vec<String> {
            let xs = parent
                .into_iter()
                .filter_map(|x| x.to_str())
                .map(|x| x.to_owned())
                .collect::<Vec<_>>();
            if prev.len() == 0 {
                return xs;
            }
            let ys = prev.into_iter();
            xs
                .into_iter()
                .zip(ys)
                .filter_map(|(left, right)| {
                    if left == right {
                        return Some(left)
                    }
                    None
                })
                .collect::<Vec<_>>()
        });
    if intersection.len() == 0 {
        return None
    }
    Some(PathBuf::from_iter(intersection))
}

pub fn build(manifest_path: &str) {
    use crate::{data::*};
    let config = config::Config::init(manifest_path);
    let intersection = intersect(config.input_files.clone(), config.output_dir.clone());
    for path in config.input_files.clone() {
        let output_path = {
            let base_path = intersection
                .as_ref()
                .and_then(|intersection| {
                    path.strip_prefix(&intersection)
                        .ok()
                        .map(|x| x.to_owned())
                })
                .unwrap_or_else(|| path.to_owned());
            config.output_dir.join(base_path)
        };
        let env = Env {
            current_dir: path.parent().unwrap().to_owned(),
            output_dir: config.output_dir.clone(),
            base_url: None,
        };
        let html = io::load_text_file(&path);
        let mut html = Node::parse_string(html);
        apply_macros(&env, &mut html);
        let html_str = html.to_html_str(0);
        std::fs::create_dir_all(output_path.parent().unwrap());
        std::fs::write(&output_path, html_str).unwrap();
    }
}

pub fn main() {
    match cli::Cli::from_args() {
        cli::Cli::Compile{manifest} => {
            build(&manifest)
        }
    }
}

