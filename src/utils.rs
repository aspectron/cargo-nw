use std::collections::HashSet;
use async_std::path::PathBuf;
use async_std::path::Path;
use globset::{Glob,GlobSet,GlobSetBuilder};
use globmatch;
use regex::Regex;
use crate::prelude::*;

pub async fn search_upwards(folder: &PathBuf, filename: &str) -> Option<PathBuf> {
    let mut folder = folder.clone();
    // let mut cur_dir = env::current_dir().unwrap();

    loop {
        // Check if the file exists in the current directory
        let file_path = folder.join(filename);
        if file_path.is_file().await {
            return Some(file_path);
        }

        // Move up to the parent directory
        if let Some(parent) = folder.parent() {
            folder = parent.to_path_buf();
        } else {
            // We've reached the root directory without finding the file, so return None
            return None;
        }
    }
}

pub async fn current_dir() -> PathBuf {
    std::env::current_dir().unwrap().into()
}

pub async fn copy_folder_with_glob_walk(
    case_sensitive : bool,
    src_folder: &Path,
    dest_folder: &Path,
    include_patterns: Option<Vec<String>>,
    exclude_patterns: Option<Vec<String>>
) -> Result<()> {

    let include = match include_patterns {
        Some(patterns) if patterns.len() != 0 => {
            patterns
        },
        _ => {
            vec!["**".to_string()]
        }
    };
    
    let exclude_globs = match exclude_patterns {
        Some(patterns) if patterns.len()!= 0 => {
            let mut exclude_globs = globset::GlobSetBuilder::new();
            for pattern in patterns {
                exclude_globs.add(Glob::new(&pattern)?);
            }
            Some(exclude_globs.build()?)
        },
        _ => { None }
    };

    let mut folders = HashSet::new();
    let mut files = HashSet::new();
    for pattern in include {
        let builder = globmatch::Builder::new( &pattern)
            .case_sensitive(case_sensitive)
            .build(src_folder)?;

        for path in builder.into_iter().flatten() {
            let relative = path.strip_prefix(src_folder).unwrap().to_path_buf();

            if is_hidden(&relative) {
                continue;
            }

            if let Some(globs) = &exclude_globs {
                if globs.is_match(&relative) {
                    continue
                }
            }

            if path.is_dir() {
                folders.insert(relative);
            } else {
                files.insert(relative);
            }
        }
    }

    for folder in folders {
        std::fs::create_dir_all(dest_folder.join(folder))?; 
    }

    for file in files {
        std::fs::copy(src_folder.join(&file),dest_folder.join(&file))?;
    }

    Ok(())
}

pub async fn copy_folder_with_glob_filters(
    src_folder: &Path, 
    dest_folder: &Path, 
    include_patterns: Vec<String>, 
    exclude_patterns: Vec<String>,
    hidden : bool,
) -> Result<()> {

    let ctx = GlobCtx::try_new(
        src_folder,
        include_patterns,
        exclude_patterns,
        hidden,
    )?;

    if ctx.include(src_folder) {
        copy_folder_recurse(src_folder, dest_folder, &ctx)?;
    }

    Ok(())
}

pub struct GlobCtx {
    prefix_len : usize,
    include : GlobSet,
    exclude : GlobSet,
    hidden : bool,
}

impl GlobCtx {
    pub fn try_new(
        base : &Path, 
        include_patterns: Vec<String>, 
        exclude_patterns: Vec<String>,
        hidden: bool,
    ) -> Result<GlobCtx> {

        let mut include_globs = GlobSetBuilder::new();
        let mut exclude_globs = GlobSetBuilder::new();

        for pattern in include_patterns.iter() {
            include_globs.add(Glob::new(pattern)?);
        }
        for pattern in exclude_patterns.iter() {
            exclude_globs.add(Glob::new(pattern)?);
        }
    
        let include = include_globs.build()?;
        let exclude = exclude_globs.build()?;
        let prefix_len = base.to_path_buf().to_string_lossy().len();
        let ctx = GlobCtx {
            prefix_len,
            include,
            exclude,
            hidden,
        };

        Ok(ctx)
    }

    pub fn include(&self, path : &Path)
    -> bool 
    {
        if !self.hidden && is_hidden(path) {
            return false;
        }
        let path = path.to_string_lossy();
        let path = path.split_at(self.prefix_len).1;
        if !self.include.is_empty() && !self.include.is_match(path) {
            return false;
        }
        if !self.exclude.is_empty() && self.exclude.is_match(path) {
            return false;
        }
        true
    }
}

pub fn is_hidden<P>(path: P) -> bool
where
    P: AsRef<std::path::Path>,
{
    let is_hidden = path
        .as_ref()
        .file_name()
        .unwrap_or_else(|| path.as_ref().as_os_str())
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false);
    is_hidden
}

pub fn copy_folder_recurse(src_folder: &Path, dest_folder: &Path, ctx : &GlobCtx) -> Result<()> {
    std::fs::create_dir_all(dest_folder)?; 

    let entries = std::fs::read_dir(src_folder)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let src = src_folder.join(&path);
            if ctx.include(&src) {
                let dest = dest_folder.join(path.file_name().unwrap());
                std::fs::copy(src, dest)?;
            }
        }
        else if path.is_dir() {
            let dir_name = path.file_name().unwrap();
            let src_path = src_folder.join(dir_name);
            let dest_path = dest_folder.join(dir_name);

            if ctx.include(&src_path) {
                copy_folder_recurse(&src_path, &dest_path, &ctx)?;
            }
        }
    }

    Ok(())
}

pub async fn find_file(folder: &Path,files: &[&str]) -> Result<PathBuf> {

    for file in files {
        let path = folder.join(file);
        if path.exists().await {
            return Ok(path);
        }
    }
    return Err(format!("Unable to locate any of the files: {}", files.join(", ")).into())
}

pub fn get_env_defs(strings: &Vec<String>) -> Result<Vec<(String, String)>> {
    let regex = Regex::new(r"([^=]+?)=(.+)").unwrap();

    let mut parsed_strings = Vec::new();

    for string in strings {
        let captures = regex.captures(&string).unwrap();
        if captures.len() != 2 {
            return Err(format!("Error parsing the environment string: '{string}'").into());
        }
        let a = captures[1].to_string();
        let b = captures[2].to_string();

        parsed_strings.push((a, b));
    }

    Ok(parsed_strings)
}

pub enum ExecArgs {
    String(String),
    Argv(Vec<String>),
}

impl ExecArgs {
    pub fn try_new(cmd: &Option<String>, argv : &Option<Vec<String>>) -> Result<ExecArgs> {
        if cmd.is_some() && argv.is_some() {
            Err(
                format!("cmd and argv can not be present at the same time: '{:?}' and '{:?}' ",
                    cmd.as_ref().unwrap(),
                    argv.as_ref().unwrap()
                ).into()
            )
        } else if let Some(cmd) = cmd {
            Ok(ExecArgs::String(cmd.clone()))
        } else if let Some(argv) = argv{
            Ok(ExecArgs::Argv(argv.clone()))
        } else {
            Err(format!("ExecArgs::try_new() cmd or argv must be present").into())
        }
    }

    pub fn get(&self, tpl: Option<&Tpl>) -> Vec<String> {
        match self {
            ExecArgs::String(cmd) => {
                tpl
                .map(|tpl|tpl.transform(&cmd))
                .unwrap_or(cmd.clone())
                .split(" ")
                .map(|s|s.to_string())
                .collect::<Vec<String>>()
            },
            ExecArgs::Argv(argv) => {
                tpl
                .map(|tpl|
                    argv
                    .into_iter()
                    .map(|v|
                        tpl
                        .transform(&v)
                    )
                    .collect()
                ).unwrap_or(argv.clone())
            },
        }
    }
}

impl From<&[&str]> for ExecArgs {
    fn from(args: &[&str]) -> Self {
        ExecArgs::Argv(args.iter().map(|s|s.to_string()).collect())
    }
}

impl From<Vec<&str>> for ExecArgs {
    fn from(args: Vec<&str>) -> Self {
        ExecArgs::Argv(args.iter().map(|s|s.to_string()).collect())
    }
}
