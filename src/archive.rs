use flate2::read::GzDecoder;
use tar::Archive;
use zip::result::ZipError;
use zip::write::FileOptions;
// use std::fs;
// use std::fs::DirEntry;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};
// use async_std::path::PathBuf;
// use async_std::path::Path;
use std::io::{Seek, Write};
// use crate::manifest::Archive;
use crate::prelude::*;
use console::style;

// pub async fn extract(file: &str, dir: &str) -> Result<()> {
pub async fn extract(file: &async_std::path::PathBuf, dir: &async_std::path::PathBuf) -> Result<()> {

    let file_str = file
        .clone()
        .into_os_string()
        .into_string()?;

    // println!("extracting file: {} to {}", file_str, dir_str);

    if file_str.ends_with(".tar.gz") || file_str.ends_with(".tgz") {
        extract_tar_gz(&file.into(),&dir.into())?;
    } else if file_str.ends_with(".zip") {
        extract_zip(&file.into(),&dir.into()).await?;
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
                std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode)).unwrap();
            }
        }
    }

    Ok(())

}


fn zip_folder<T>(
    nb_files : usize,
    filename : &str,
    _path: &Path,
    it: &mut dyn Iterator<Item = DirEntry>,
    prefix: &str,
    writer: T,
    method: zip::CompressionMethod,
) -> Result<()>
where
    T: Write + Seek,
{
    let mut count: usize = 0;
    let mut bytes: usize = 0;
    let filename = style(filename).cyan();

    let mut zip = zip::ZipWriter::new(writer);
    let options = FileOptions::default()
        .compression_method(method)
        .unix_permissions(0o755);
        
    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(Path::new(prefix)).unwrap();

        // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if path.is_file() {
            // println!("adding file {:?} as {:?} ...", path, name);
            #[allow(deprecated)]
            zip.start_file_from_path(name, options)?;
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&*buffer)?;
            bytes += buffer.len();
            buffer.clear();
            // zip.fl
        } else if !name.as_os_str().is_empty() {
            // Only if not root! Avoids path spec / warning
            // and mapname conversion failed error on unzip
            // println!("adding dir {:?} as {:?} ...", path, name);
            #[allow(deprecated)]
            zip.add_directory_from_path(name, options)?;
        }

        count += 1;
        let pos = count as f64 / nb_files as f64 * 100.0;
        let percent = style(format!("{:1.2}%",pos)).cyan();
        let size = style(format!("{:1.2} Mb",bytes as f64 / 1024.0 / 1024.0)).cyan();
        let files = style(format!("{count}/{nb_files} files")).cyan();
        log_state!("Compressing","... {filename}: {percent} - {files} - {size} ");
    }

    log_state_clear();
    zip.finish()?;

    Ok(())
}


pub fn compress_folder(
    src_dir: &async_std::path::Path,
    dst_file: &async_std::path::Path,
    // method: crate::manifest::Archive,
    method: crate::manifest::Archive,
    // method: zip::CompressionMethod,
) -> Result<()> { //zip::result::ZipResult<()> {
    if !Path::new(src_dir).is_dir() {
        return Err(ZipError::FileNotFound.into());
    }



    log!("Archive","compressing ({})", method.to_string());

    let method : zip::CompressionMethod = method.into();

    let path = Path::new(dst_file);
    let file = File::create(path).unwrap();

    let walkdir = WalkDir::new(src_dir);
    let it = walkdir.into_iter();
    let mut nb_files = 0;
    for _ in it {
        nb_files = nb_files+1;
    }

    let walkdir = WalkDir::new(src_dir);
    let it = walkdir.into_iter();

    zip_folder(
        nb_files,
        dst_file.file_name().unwrap().to_str().unwrap(),
        path,
        &mut it.filter_map(|e| e.ok()), 
        src_dir.to_str().unwrap(), 
        file,
        method
    )?;

    Ok(())
}