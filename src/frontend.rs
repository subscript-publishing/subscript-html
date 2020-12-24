use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashSet, HashMap};
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use serde::{Serialize, Deserialize};
use std::iter::FromIterator;
use crate::data::{Store};

pub mod browser;

///////////////////////////////////////////////////////////////////////////////
// CURRENT ENV
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct Env {
    pub current_dir: PathBuf,
    pub output_dir: PathBuf,
    pub base_url: Option<String>,
    pub handles: Store<Handles>,
    pub macro_system: MacroSystem,
    pub io_paths: Vec<IoPath>,
    pub changed: Option<PathBuf>,
}

impl Env {
    pub fn try_load_text_file<P: AsRef<Path>>(&self, path: P) -> Result<(PathBuf, String), ()> {
        let mut path = path.as_ref().to_owned();
        if path.starts_with(".") || path.starts_with("..") {
            path = self.current_dir.join(path);
        }
        io::try_load_text_file(&path).map(move |source| {
            (path, source)
        })
    }
}

#[derive(Debug, Clone)]
pub struct IoPath {
    pub input_file: PathBuf,
    pub output_file: PathBuf,
    pub output_dir: PathBuf,
}


///////////////////////////////////////////////////////////////////////////////
// MISC
///////////////////////////////////////////////////////////////////////////////

pub struct Handles {
    rhai_subsystem: crate::embed::rhai::RhaiSubSystem,
}

#[derive(Clone)]
pub struct MacroSystem {
    apply_macros: fn(env: &Env, html: &mut crate::data::Node),
}

impl std::fmt::Debug for MacroSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MacroSystem(\"...\")").finish()
    }
}


///////////////////////////////////////////////////////////////////////////////
// FRONTEND IO HELPERS
///////////////////////////////////////////////////////////////////////////////

pub mod io {
    use super::*;

    pub fn expand_globs(globs: Vec<String>) -> Vec<PathBuf> {
        globs
            .into_iter()
            .filter_map(|x| glob::glob(&x).ok())
            .flat_map(|x| x.into_iter())
            .filter_map(Result::ok)
            .collect::<Vec<_>>()
    }
    
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
        pub(crate) plugins: Option<Vec<String>>,
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
    #[derive(Clone)]
    pub struct Config {
        pub input_files: Vec<PathBuf>,
        pub root: PathBuf,
        pub output_dir: PathBuf,
        pub handles: Store<Handles>,
        pub macro_system: MacroSystem,
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
            let input_files = io::expand_globs(manifest.project.pages.clone());
            let output_dir = manifest.project.output_dir.clone();
            if !output_dir.exists() {
                std::fs::create_dir_all(&output_dir);
            }
            let handles = {
                let rhai_subsystem = crate::embed::rhai::RhaiSubSystem::new(
                    manifest.project.plugins.unwrap_or(Vec::new())
                );
                Handles {
                    rhai_subsystem: rhai_subsystem,
                }
            };
            let macro_system = MacroSystem {
                apply_macros: apply_macros,
            };
            Config {
                input_files,
                root: manifest.project.root.clone(),
                output_dir,
                handles: Store::new(handles),
                macro_system,
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
        Serve {
            /// Explicit path to the manifest file
            #[structopt(long, default_value="./subscript.toml")]
            manifest: String,

            #[structopt(long, default_value="3000")]
            port: u16,

            /// Automatically open chrome in kiosk mode.
            #[structopt(long)]
            open_browser: bool,
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
    pub fn cache_file(env: &Env, source_path: &str) -> Option<String> {
        GLOBAL_CACHE.cache_file(env, source_path)
    }
    pub fn cache_file_glob(env: &Env, source: &str) -> Vec<String> {
        GLOBAL_CACHE.cache_file_glob(env, source)
    }
    pub fn cache_inline_text(env: &Env, source_path: &str) -> Option<String> {
        GLOBAL_CACHE.cache_inline_text(env, source_path)
    }
    pub fn cache_value(env: &Env, key: &str, value: &str) {
        GLOBAL_CACHE.insert(
            key,
            CachedItem::InlineText{contents: value.to_owned()}
        )
    }
    pub fn lookup_value(env: &Env, key: &str) -> Option<String> {
        GLOBAL_CACHE.lookup(key).and_then(|cached| match cached {
            CachedItem::InlineText{contents} => Some(contents),
            _ => None
        })
    }
    pub fn lookup_hash_file(env: &Env, key: &str) -> Option<String> {
        GLOBAL_CACHE.lookup(key).and_then(|cached| match cached {
            CachedItem::FilePath{output} => Some(output),
            _ => None
        })
    }
    pub fn cache_hash_file(env: &Env, key: &str, value: &str) -> Option<String> {
        let hash = crate::data::utils::hash_value(&value);
        let file_name = format!("{}", hash);
        let out_file_path = env.output_dir
            .join("ss-data")
            .join(&PathBuf::from(&file_name));
        std::fs::create_dir_all(out_file_path.parent().unwrap());
        std::fs::write(&out_file_path, value).unwrap();
        let out_src_path = format!("/ss-data/{}", file_name);
        let cached_entry = CachedItem::FilePath {
            output: out_src_path.clone(),
        };
        GLOBAL_CACHE.insert(key, cached_entry);
        Some(out_src_path)
    }
    type SourcePath = PathBuf;
    type OutputPath = PathBuf;
    struct Cache(Arc<Mutex<HashMap<String, CachedItem>>>);
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
        FileGlob {
            output: Vec<String>,
        }
    }
    impl Cache {
        fn new() -> Self {
            Cache(Arc::new(Mutex::new(HashMap::default())))
        }
        fn lookup(&self, path: &str) -> Option<CachedItem> {
            self.0.lock().unwrap().get(path).map(|x| x.clone())
        }
        fn insert(&self, source_path: &str, cached_file: CachedItem) {
            self.0.lock().unwrap().insert(
                source_path.to_owned(),
                cached_file,
            );
        }
        fn cache_file(&self, env: &Env, source_path: &str) -> Option<String> {
            if let Some(CachedItem::FilePath{output}) = self.lookup(source_path) {
                return Some(output)
            }
            let out_path = utils::cache_file_dep(env, &PathBuf::from(source_path))?;
            let cached_file = CachedItem::FilePath {
                output: out_path.clone(),
            };
            self.insert(source_path, cached_file);
            Some(out_path)
        }
        fn cache_inline_text(&self, _: &Env, source_path: &str) -> Option<String> {
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
        fn cache_file_glob(&self, env: &Env, source: &str) -> Vec<String> {
            if let Some(CachedItem::FileGlob{output}) = self.lookup(source) {
                return output;
            }
            let source = env.current_dir.join(source);
            let source = source.to_str().unwrap();
            let out_paths = crate::frontend::io::expand_globs(vec![source.to_owned()])
                .into_iter()
                .filter_map(|path| {
                    let out_path = utils::cache_file_dep_without_normalizing(
                        env,
                        &PathBuf::from(&path),
                    );
                    if out_path.is_none() {
                        eprintln!("[warning] ignoring asset: {:?}", path);
                    }
                    out_path
                })
                .collect::<Vec<_>>();
            let cached_entry = CachedItem::FileGlob {
                output: out_paths.clone(),
            };
            self.insert(source, cached_entry);
            out_paths
        }
    }
}


///////////////////////////////////////////////////////////////////////////////
// BUILD
///////////////////////////////////////////////////////////////////////////////

pub fn apply_macros(env: &Env, html: &mut crate::data::Node) {
    use crate::data::Node;
    let env = env.clone();
    let rhai_macros = env.handles.access(|handles: &Handles| {
        handles.rhai_subsystem.get_macro_tag_names()
    });
    let rust_macros = crate::macros::tag_macros(&env)
        .into_iter()
        .map(|x| x.tag)
        .collect::<HashSet<_>>();
    html.eval(Rc::new({
        let env = env.clone();
        let rhai_macros = rhai_macros.clone();
        move |node: &mut Node| {
            node.tag()
                .map({
                    |node_tag| {
                        if rhai_macros.clone().contains(&node_tag) {
                            env.handles.access_mut({
                                |handles: &mut Handles| {
                                    handles.rhai_subsystem.consider_node(node)
                                }
                            });
                        }
                        if rust_macros.contains(&node_tag) {
                            for tag_macro in crate::macros::tag_macros(&env) {
                                if tag_macro.tag == node_tag {
                                    (tag_macro.callback.0)(node);
                                }
                            }
                        }
                    }
                });
        }
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

pub fn init(manifest_path: &str) -> (config::Config, Vec<IoPath>) {
    use crate::{data::*};
    let config = config::Config::init(manifest_path);
    let intersection = intersect(config.input_files.clone(), config.output_dir.clone());
    let io_paths = config.input_files
        .clone()
        .into_iter()
        .map(|path| {
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
            IoPath {
                input_file: path,
                output_file: output_path,
                output_dir: config.output_dir.clone(),
            }
        })
        .collect::<Vec<_>>();
    (config, io_paths)
}

pub fn build(config: &config::Config, io_paths: &[IoPath], changed: Option<PathBuf>) {
    use crate::{data::*};
    for IoPath{input_file: path, output_file: output_path, ..} in io_paths.clone() {
        let env = Env {
            current_dir: path.parent().unwrap().to_owned(),
            output_dir: config.output_dir.clone(),
            base_url: None,
            handles: config.handles.clone(),
            macro_system: config.macro_system.clone(),
            io_paths: io_paths.to_owned(),
            changed: changed.clone(),
        };
        let html = io::load_text_file(&path);
        let mut html = Node::parse_string(html);
        apply_macros(&env, &mut html);
        crate::macros::postproc_document_macros(&env, &mut html);
        let html_str = html.to_html_str(0);
        std::fs::create_dir_all(output_path.parent().unwrap());
        std::fs::write(&output_path, html_str).unwrap();
    }
}

pub fn serve(manifest_path: &str, port: u16, open_browser: bool) {
    use crate::{data::*};
    use config::Config;
    let (config, io_paths) = init(manifest_path);

    use hotwatch::{Hotwatch, Event};
    let mut hotwatch = Hotwatch::new().expect("hotwatch failed to initialize!");
    let fast_upate_mode = false;

    let rebuild = |config: Store<Config>, io_paths: &[IoPath], path: &PathBuf| {
        let root = std::env::current_dir().unwrap();
        let path = path.strip_prefix(&root).unwrap().to_owned();
        config.access(|config| {
            if !path.starts_with(&config.output_dir.clone()) {
                build(config, io_paths, Some(path.clone()));
                println!("[Subscript] Compiled [{}]", path.to_str().unwrap());
            }
        })
    };
    hotwatch.unwatch(config.output_dir.clone());
    hotwatch.watch(config.root.clone(), {
        let output = config.output_dir.clone();
        let root = config.root.clone();
        let config: Store<Config> = Store::new(config.clone());
        let io_paths = io_paths.clone();
        move |event: Event| {
            let config = config.clone();
            match event {
                Event::Create(path) => {
                    rebuild(config.clone(), &io_paths, &path);
                }
                Event::Write(path) => {
                    rebuild(config.clone(), &io_paths, &path);
                }
                Event::Remove(path) => {
                    rebuild(config.clone(), &io_paths, &path);
                }
                Event::Rename(from, to) => {
                    rebuild(config.clone(), &io_paths, &to);
                }
                Event::NoticeWrite(_) => {}
                Event::NoticeRemove(_) => {}
                Event::Chmod(_) => {}
                Event::Rescan => {}
                Event::Error(error, path) => {}
            };
        }
    }).expect("failed to watch file!");
    build(&config, &io_paths, None);
    if open_browser {
        std::thread::spawn({
            move || {
                browser::run(port);
            }
        });
    }
    crate::server::run_server(
        crate::server::Args{
            address: String::from("127.0.0.1"),
            port,
            cache: 0,
            cors: false,
            compress: false,
            path: config.output_dir.clone(),
            all: true,
            ignore: false,
            follow_links: true,
            render_index: true,
            log: false,
            path_prefix: None,
        }
    );
}

pub fn main() {
    match cli::Cli::from_args() {
        cli::Cli::Compile{manifest} => {
            let (config, io_paths) = init(&manifest);
            build(&config, &io_paths, None);
        }
        cli::Cli::Serve{manifest, port, open_browser} => {
            serve(&manifest, port, open_browser)
        }
    }
}

