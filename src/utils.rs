use flate2::read::GzDecoder;
use tar::Archive;
use async_std::fs;
use std::collections::HashSet;
use async_std::path::PathBuf;
use async_std::path::Path;
use globset::{Glob,GlobSet,GlobSetBuilder};
use globmatch;

use crate::prelude::*;

// pub async fn extract(file: &str, dir: &str) -> Result<()> {
pub async fn extract(file: &PathBuf, dir: &PathBuf) -> Result<()> {

    // let dir_str = dir
    //     .clone()
    //     .into_os_string()
    //     .into_string()?;

    let file_str = file
        .clone()
        .into_os_string()
        .into_string()?;

    // println!("extracting file: {} to {}", file_str, dir_str);

    if file_str.ends_with(".tar.gz") || file_str.ends_with(".tgz") {
        extract_tar_gz(file,dir)?;
    } else if file_str.ends_with(".zip") {
        extract_zip(file,dir).await?;
    } else {
        return Err(format!("extract(): unsupported file type: {}", file_str).into());
    }


    Ok(())
}

fn extract_tar_gz(file: &PathBuf, dir: &PathBuf) -> Result<()> {
    // https://rust-lang-nursery.github.io/rust-cookbook/compression/tar.html
    let tar_gz = std::fs::File::open(file)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive.unpack(dir)?;
    Ok(())
}

async fn extract_zip(file: &PathBuf, dir: &PathBuf) -> Result<()> {
    // let args: Vec<_> = std::env::args().collect();
    // if args.len() < 2 {
    //     println!("Usage: {} <filename>", args[0]);
    //     return 1;
    // }
    // let fname = std::path::Path::new(&*args[1]);
    let file_reader = std::fs::File::open(&file).unwrap();
    let mut archive = zip::ZipArchive::new(file_reader).unwrap();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(path) => std::path::Path::new(dir).join(path), //.to_owned(),
            None => continue,
        };

        // {
        //     let comment = file.comment();
        //     if !comment.is_empty() {
        //         println!("File {} comment: {}", i, comment);
        //     }
        // }

        if (*file.name()).ends_with('/') {
            // println!("File {} extracted to \"{}\"", i, outpath.display());
            std::fs::create_dir_all(&outpath).unwrap();
        } else {
            // println!(
            //     "File {} extracted to \"{}\" ({} bytes)",
            //     i,
            //     outpath.display(),
            //     file.size()
            // );
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p).unwrap();
                }
            }
            let mut outfile = std::fs::File::create(&outpath).unwrap();
            std::io::copy(&mut file, &mut outfile).unwrap();
        }

        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                std::fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).unwrap();
            }
        }
    }

    Ok(())

}

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

pub async fn copy_folder_with_glob_walk(case_sensitive : bool, src_folder: &Path, dest_folder: &Path, include_patterns: &[&str], exclude_patterns: &[&str]) -> Result<()> {

    let mut include = include_patterns.to_vec();
    if include.is_empty() {
        include.push("**");
    }

    let mut exclude_globs = globset::GlobSetBuilder::new();
    for pattern in exclude_patterns {
        exclude_globs.add(Glob::new(pattern)?);
    }
    let exclude_globs = exclude_globs.build()?;

    let mut folders = HashSet::new();
    let mut files = HashSet::new();
    for pattern in include {
        let builder = globmatch::Builder::new( pattern)
            .case_sensitive(case_sensitive)
            .build(src_folder)?;

        for path in builder.into_iter().flatten() {
            let relative = path.strip_prefix(src_folder).unwrap().to_path_buf();

            if is_hidden(&relative) {
                continue;
            }

            if !exclude_globs.is_empty() && exclude_globs.is_match(&relative) {
                continue;
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
    // case_sensitive : bool, 
    src_folder: &Path, 
    dest_folder: &Path, 
    include_patterns: &[&str], 
    exclude_patterns: &[&str],
) -> Result<()> {

    let ctx = GlobCtx::try_new(src_folder, include_patterns, exclude_patterns)?;
    if ctx.include(src_folder) {
        copy_folder_recurse(src_folder, dest_folder, &ctx)?;
    }

    Ok(())
}

pub struct GlobCtx {
    root : PathBuf,
    include : GlobSet,
    exclude : GlobSet,
}

impl GlobCtx {
    pub fn try_new(
        base : &Path, 
        include_patterns: &[&str], 
        exclude_patterns: &[&str]
    ) -> Result<GlobCtx> {

        let mut include_globs = GlobSetBuilder::new();
        let mut exclude_globs = GlobSetBuilder::new();

        if include_patterns.is_empty() {
            include_globs.add(Glob::new("**/*")?);
        } else {
            for pattern in include_patterns.iter() {
                include_globs.add(Glob::new(pattern)?);
            }
        }
    
        for pattern in exclude_patterns.iter() {
            exclude_globs.add(Glob::new(pattern)?);
        }
    
        let include = include_globs.build()?;
        let exclude = exclude_globs.build()?;
        let root = base.to_path_buf();
        let ctx = GlobCtx {
            root : root.to_path_buf(),
            include,
            exclude
        };

        Ok(ctx)
    }

    pub fn include(&self, path : &Path)
    -> bool 
    // where P : AsRef<std::path::Path>
    {
        if is_hidden(path) {
            return false;
        }

        let path = path.strip_prefix(&self.root).unwrap();

        if !self.include.is_empty() &&  !self.include.is_match(path) {
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

pub async fn execute(ctx : &Context, cmd : &str, folder : &Option<String>) -> Result<()> {

    let folder = if let Some(folder) = folder {
        ctx.app_root_folder.join(folder)
    } else {
        ctx.app_root_folder.clone()
    };

    let argv : Vec<String> = cmd.split(" ").map(|s|s.to_string()).collect();
    let program = argv.first().expect("missing program in build config");
    let args = argv[1..].to_vec();

    if let Err(e) = duct::cmd(program,args).dir(&folder).run() {
        println!("Error while executing: '{}'", cmd);
        Err(e.into())
    } else {
        Ok(())
    }
}

