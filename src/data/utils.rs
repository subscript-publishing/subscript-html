use std::path::{PathBuf, Path};
use std::convert::AsRef;

pub fn is_inline_tag(tag: &str) -> bool {
    if (tag == "a") {return true}
    if (tag == "abbr") {return true}
    if (tag == "audio") {return true}
    if (tag == "b") {return true}
    if (tag == "bdo") {return true}
    if (tag == "br") {return true}
    if (tag == "button") {return true}
    if (tag == "canvas") {return true}
    if (tag == "cite") {return true}
    if (tag == "code") {return true}
    if (tag == "command") {return true}
    if (tag == "data") {return true}
    if (tag == "datalist") {return true}
    if (tag == "dfn") {return true}
    if (tag == "em") {return true}
    if (tag == "embed") {return true}
    if (tag == "i") {return true}
    if (tag == "iframe") {return true}
    if (tag == "img") {return true}
    if (tag == "input") {return true}
    if (tag == "kbd") {return true}
    if (tag == "keygen") {return true}
    if (tag == "label") {return true}
    if (tag == "mark") {return true}
    if (tag == "math") {return true}
    if (tag == "meter") {return true}
    if (tag == "noscript") {return true}
    if (tag == "object") {return true}
    if (tag == "output") {return true}
    if (tag == "picture") {return true}
    if (tag == "progress") {return true}
    if (tag == "q") {return true}
    if (tag == "ruby") {return true}
    if (tag == "samp") {return true}
    if (tag == "script") {return true}
    if (tag == "select") {return true}
    if (tag == "small") {return true}
    if (tag == "span") {return true}
    if (tag == "strong") {return true}
    if (tag == "sub") {return true}
    if (tag == "sup") {return true}
    if (tag == "svg") {return true}
    if (tag == "textarea") {return true}
    if (tag == "time") {return true}
    if (tag == "var") {return true}
    if (tag == "video") {return true}
    if (tag == "wbr") {return true}
    false
}

pub fn is_header_tag(tag: &str) -> bool {
    tag == "h1" ||
    tag == "h2" ||
    tag == "h3" ||
    tag == "h4" ||
    tag == "h5" ||
    tag == "h6"
}

pub fn lookup_hash<H: std::hash::Hash>(data: &H) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish()
}

pub fn cache_file_dep(
    ctx: &crate::frontend::Env,
    input_path: &PathBuf,
) -> Option<String> {
    let src_ext = input_path
        .extension()
        .and_then(|x| x.to_str().map(|x| x.to_owned()))
        .map(|x| format!(".{}", x))?;
    let ref input_path = ctx.current_dir.join(input_path);
    if let Ok(binary) = crate::frontend::io::try_load_binary_file(input_path) {
        let uid = lookup_hash(&binary);
        let file_name = format!("{}{}", uid, src_ext);
        let output_file_path = ctx.output_dir
            .join("ss-data")
            .join(&PathBuf::from(file_name));
        let parent_dir = output_file_path.parent().unwrap();
        if !parent_dir.exists() {
            std::fs::create_dir_all(&parent_dir).unwrap();
        }
        if !output_file_path.exists() {
            std::fs::write(&output_file_path, binary).unwrap();
        }
        let target_path = output_file_path
            .strip_prefix(&ctx.output_dir)
            .map(|x| x.to_owned())
            .unwrap_or(output_file_path);
        let target_path = target_path.to_str().unwrap();
        if let Some(base_url) = ctx.base_url.clone() {
            let base_url = base_url
                .strip_suffix("/")
                .map(|x| x.to_owned())
                .unwrap_or(base_url);
            Some(format!("{}/{}", base_url, target_path))
        } else {
            Some(format!(
                "/{}",
                target_path.to_owned()
            ))
        }
    } else {
        eprintln!(
            "[warning] ignoring asset: {:?}",
            input_path
        );
        Some(input_path.to_str().unwrap().to_owned())
    }
}

pub fn cache_file_dep_without_normalizing(
    ctx: &crate::frontend::Env,
    input_path: &PathBuf,
) -> Option<String> {
    let src_ext = input_path
        .extension()
        .and_then(|x| x.to_str().map(|x| x.to_owned()))
        .map(|x| format!(".{}", x))?;
    if let Ok(binary) = crate::frontend::io::try_load_binary_file(input_path) {
        let uid = lookup_hash(&binary);
        let file_name = format!("{}{}", uid, src_ext);
        let output_file_path = ctx.output_dir
            .join("ss-data")
            .join(&PathBuf::from(file_name));
        let parent_dir = output_file_path.parent().unwrap();
        if !parent_dir.exists() {
            std::fs::create_dir_all(&parent_dir).unwrap();
        }
        if !output_file_path.exists() {
            std::fs::write(&output_file_path, binary).unwrap();
        }
        let target_path = output_file_path
            .strip_prefix(&ctx.output_dir)
            .map(|x| x.to_owned())
            .unwrap_or(output_file_path);
        let target_path = target_path.to_str().unwrap();
        if let Some(base_url) = ctx.base_url.clone() {
            let base_url = base_url
                .strip_suffix("/")
                .map(|x| x.to_owned())
                .unwrap_or(base_url);
            Some(format!("{}/{}", base_url, target_path))
        } else {
            Some(format!(
                "/{}",
                target_path.to_owned()
            ))
        }
    } else {
        eprintln!(
            "[warning] ignoring asset: {:?}",
            input_path
        );
        Some(input_path.to_str().unwrap().to_owned())
    }
}

