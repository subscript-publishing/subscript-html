use std::rc::Rc;
use std::collections::{HashSet, HashMap};
use std::path::{Path, PathBuf};
use std::convert::AsRef;
use structopt::StructOpt;
use serde::{Serialize, Deserialize};


///////////////////////////////////////////////////////////////////////////////
// CURRENT ENV
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq)]
pub struct Env {
    // pub root_dir: PathBuf,
    pub current_dir: PathBuf,
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

pub fn build(manifest_path: &str) {
    use crate::{data::*};
    let config = config::Config::init(manifest_path);
    for path in config.input_files {
        let env = Env {
            current_dir: path.parent().unwrap().to_owned(),
        };
        let html = io::load_text_file(path);
        let mut html = Node::parse_string(html);
        html.eval(Rc::new(move |node: &mut Node| {
            node.tag()
                .map(|node_tag| {
                    for tag_macro in crate::macros::tag_macros(&env) {
                        if tag_macro.tag == node_tag {
                            (tag_macro.callback.0)(node);
                        }
                    }
                });
        }));
        let html_str = html.to_html_str(0);
        println!("{}", html_str);
    }
}

pub fn run() {
    match cli::Cli::from_args() {
        cli::Cli::Compile{manifest} => {
            build(&manifest)
        }
    }
}

