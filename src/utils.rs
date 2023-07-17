use crate::prelude::*;
use async_std::path::Path;
use async_std::path::PathBuf;
use regex::Regex;

pub async fn search_upwards(folder: &PathBuf, filename: &str) -> Option<PathBuf> {
    let mut folder = folder.clone();

    loop {
        let file_path = folder.join(filename);
        if file_path.is_file().await {
            return Some(sanitize(&file_path));
        }

        if let Some(parent) = folder.parent() {
            folder = sanitize(parent);
        } else {
            return None;
        }
    }
}

pub async fn current_dir() -> PathBuf {
    std::env::current_dir().unwrap().into()
}

// pub async fn find_file(folder: &Path,files: &[&str]) -> Result<PathBuf> {
pub async fn find_file(folder: &Path, files: &[String]) -> Result<PathBuf> {
    for file in files {
        let path = folder.join(file);
        if let Ok(path) = path.canonicalize().await {
            if path.is_file().await {
                return Ok(sanitize(path));
            }
        }
    }
    Err(format!(
        "Unable to locate any of the files: {} \nfrom {:?} directory",
        files.join(", "),
        folder.to_str().unwrap_or("")
    )
    .into())
}

pub fn get_env_defs(strings: &Vec<String>) -> Result<Vec<(String, String)>> {
    let regex = Regex::new(r"([^=]+?)=(.+)").unwrap();

    let mut parsed_strings = Vec::new();

    for string in strings {
        let captures = regex.captures(string).unwrap();
        if captures.len() != 2 {
            return Err(format!("Error parsing the environment string: '{string}'").into());
        }
        let a = captures[1].to_string();
        let b = captures[2].to_string();

        parsed_strings.push((a, b));
    }

    Ok(parsed_strings)
}


pub fn normalize<P>(path: P) -> Result<PathBuf>
where
    P: AsRef<Path>,
{
    cfg_if!{
        if #[cfg(platform = "windows")] {
            normalize_with_separator(path.as_ref(), "\\")
        } else {
            normalize_with_separator(path.as_ref(), "/")
        }
    }
}

pub fn normalize_with_separator(path: &Path, separator: &str) -> Result<PathBuf> {
    let mut result = PathBuf::new();

    for component in path.components() {
        if let Some(c) = component.as_os_str().to_str() {
            if c == "." {
                continue;
            } else if c == ".." {
                result.pop();
            } else {
                result.push(c);
            }
        } else {
            return Err(Error::InvalidPath(path.to_string_lossy().to_string()));
        }
    }

    if !result.is_absolute() {
        result = separator.parse::<PathBuf>().unwrap().join(&result);
    }

    Ok(sanitize(&result))
}

pub fn sanitize<P : AsRef<Path>>(path: P) -> PathBuf {
    let p = path.as_ref().to_string_lossy().to_string();
    PathBuf::from(p.replace("\\\\?\\", ""))
}
