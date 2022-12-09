use flate2::read::GzDecoder;
use tar::Archive;
use std::fs;
use std::io;
// use async_std::path::Path;
use async_std::path::PathBuf;

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
        extract_zip(file,dir)?;
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

fn extract_zip(file: &PathBuf, dir: &PathBuf) -> Result<()> {
    // let args: Vec<_> = std::env::args().collect();
    // if args.len() < 2 {
    //     println!("Usage: {} <filename>", args[0]);
    //     return 1;
    // }
    // let fname = std::path::Path::new(&*args[1]);
    let file_reader = fs::File::open(&file).unwrap();
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
            fs::create_dir_all(&outpath).unwrap();
        } else {
            // println!(
            //     "File {} extracted to \"{}\" ({} bytes)",
            //     i,
            //     outpath.display(),
            //     file.size()
            // );
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p).unwrap();
                }
            }
            let mut outfile = fs::File::create(&outpath).unwrap();
            io::copy(&mut file, &mut outfile).unwrap();
        }

        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).unwrap();
            }
        }
    }

    Ok(())
    // 0
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


// pub fn get_parent_folder_name(path: &PathBuf) -> Option<PathBuf> {
//     path.parent()
//         .and_then(|parent| parent.file_name())
//         // .and_then(|file_name| file_name.to_str())
//         // .map(|file_name| file_name.to_string())
// }