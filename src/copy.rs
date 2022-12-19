use std::collections::HashSet;
use async_std::path::Path;
use async_std::path::PathBuf;
use globset::{Glob,GlobSet};
use walkdir::WalkDir;
use regex::RegexSet;
// use ignore::Walk;
use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct CopyOptions {
    pub hidden : bool,
    // pub case_sensitive: bool,
    pub flatten : bool,
    // pub rename : Option<String>,
}

impl From<Copy> for CopyOptions {
    fn from(options: Copy) -> Self {
        CopyOptions {
            hidden: options.hidden.unwrap_or(false),
            // case_sensitive: options.case_sensitive.unwrap_or(false),
            flatten: options.flatten.unwrap_or(false),
            // rename : options.rename.clone(),
        }
    }
}

impl Default for CopyOptions {
    fn default() -> Self {
        CopyOptions {
            hidden: false,
            // case_sensitive: false,
            flatten: false,
            // rename: None,
        }
    }
}

impl CopyOptions {
    pub fn new(hidden : bool) -> Self {
        CopyOptions {
            hidden,
            // case_sensitive : false,
            flatten: false,
            // rename : None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Filter {
    Glob(GlobSet),
    Regex(RegexSet)
}

impl Filter {
    pub fn try_glob(tpl: &Tpl, glob_list: &Vec<String>) -> Result<Self> {
        let mut builder = globset::GlobSetBuilder::new();
        for pattern in glob_list {
            builder.add(Glob::new(&tpl.transform(pattern))?);
        }
        Ok(Filter::Glob(builder.build()?))
    }

    pub fn try_regex(tpl: &Tpl, regex_list: &Vec<String>) -> Result<Self> {
        Ok(Filter::Regex(
            RegexSet::new(
                regex_list
                .iter()
                .map(|s|tpl.transform(s))
                .collect::<Vec<_>>()
            )?
        ))
    }
}

impl TryFrom<(&Tpl,&CopyFilter)> for Filter {
    type Error = Error;
    fn try_from((tpl,cf):(&Tpl,&CopyFilter)) -> Result<Filter> {
        match cf {
            CopyFilter::Glob(glob_list) => {
                Filter::try_glob(tpl, glob_list)
            },
            CopyFilter::Regex(regex_list) => {
                Filter::try_regex(tpl, regex_list)
            }
        }
    }
}

impl TryFrom<(&Tpl,&Copy)> for Filter {
    type Error = Error;
    fn try_from((tpl,copy):(&Tpl,&Copy)) -> Result<Filter> {

        match (&copy.glob,&copy.regex) {
            (Some(glob), None) => {
                Filter::try_glob(tpl,&glob)
            },
            (None, Some(regex)) => {
                Filter::try_regex(tpl,&regex)
            },
            _ => {
                Err(format!("copy directive must have one 'glob' or 'regex' property: {:?}", copy).into())
            }
        }
    }
}

impl Filter {
    pub fn is_match(&self, text: &str) -> bool {
        match self {
            Filter::Glob(glob) => glob.is_match(text),
            Filter::Regex(regex) => regex.is_match(text)
        }
    }
}

#[derive(Debug, Clone)]
pub struct Filters {
    pub include : Option<Vec<Filter>>,
    pub exclude : Option<Vec<Filter>>
}

impl TryFrom<(Option<Filter>,Option<Filter>)> for Filters {
    type Error = Error;
    fn try_from((include,exclude):(Option<Filter>,Option<Filter>)) -> Result<Filters> {

        let include = if let Some(include) = include {
            Some(vec![include])
        } else { None };
        let exclude = if let Some(exclude) = exclude {
            Some(vec![exclude])
        } else { None };

        Ok(Filters { include, exclude })
    }
}

impl TryFrom<(&Tpl,&Copy)> for Filters {
    type Error = Error;
    fn try_from((tpl,copy):(&Tpl,&Copy)) -> Result<Filters> {
        let filter: Filter = (tpl,copy).try_into()?;
        Ok(Filters { include: Some(vec![filter]), exclude: None })
    }
}

impl TryFrom<(&Tpl,&Option<Vec<CopyFilter>>,&Option<Vec<CopyFilter>>)> for Filters {
    type Error = Error;
    fn try_from((tpl,include,exclude):(&Tpl,&Option<Vec<CopyFilter>>,&Option<Vec<CopyFilter>>)) -> Result<Filters> {

        let include = if let Some(include) = include {
            let mut include_filters = Vec::new();
            for filter in include {
                include_filters.push((tpl,filter).try_into()?);
            }
            Some(include_filters)
        } else { None };
        let exclude = if let Some(exclude) = exclude {
            let mut exclude_filters = Vec::new();
            for filter in exclude {
                exclude_filters.push((tpl,filter).try_into()?);
            }
            Some(exclude_filters)
        } else { None };

        Ok(Filters { include, exclude })
    }
}

impl Filters {
    pub fn is_match(&self, text: &str) -> bool {
        let include = if let Some(include) = &self.include {
            include
            .iter()
            .find(|f|f.is_match(text))
            .is_some()
        } else {
            true
        };
        
        let exclude = if let Some(exclude) = &self.exclude {
            exclude
                .iter()
                .find(|f|f.is_match(text))
                .is_some()
        } else {
            false
        };

        include && !exclude
    }
}

// fn as_absolute(path: &std::path::Path) -> String {
//     ["/",path.to_str().unwrap()].join("")
// }

// pub fn get_destination(to : &str) -> (String,String) {

// }

pub struct Rename {
    pub stem : Option<String>,
    pub extension : Option<String>
}

impl Rename {
    pub fn try_from(dest: &Path) -> Option<Rename> {

        let stem = dest.file_stem();
        let extension = dest.extension();
    
        let stem = if stem.map(|s|s.to_str().unwrap() == "*").unwrap_or(false) {
            None
        } else {
            Some(stem.unwrap().to_str().unwrap().to_string())
        };

        let extension = if extension.map(|s|s.to_str().unwrap() == "*").unwrap_or(false) {
            None
        } else {
            Some(extension.unwrap().to_str().unwrap().to_string())
        };

        if stem.is_some() && extension.is_some() {
            Some(Rename { stem, extension })
        } else {
            None
        }
    }

    pub fn transform(&self, path: &mut PathBuf) {

        let stem = if let Some(stem) = &self.stem {
            Some(stem.clone())
        } else {
            path
                .file_stem()
                .map(|s|
                    s.to_str()
                    .unwrap()
                    .to_string()
                )
                // .unwrap()
                // .to_string()
        };

        let extension = if let Some(extension) = &self.extension {
            Some(extension.clone())
        } else {
            path
                .extension()
                .map(|s|
                    s.to_str()
                    .unwrap()
                    .to_string()
                )
                // .unwrap()
                // .to_string()
        };

        let filename = [stem,extension].iter().flatten().cloned().collect::<Vec<String>>().join(".");
        path.set_file_name(filename);
    }
}

pub async fn copy_folder_with_filters(
    src_folder: &Path,
    dest_folder: &Path,
    // to : Option<String>,
    filters: Filters,
    options: CopyOptions,
) -> Result<()> {

    // let list = WalkDir::new(src_folder)
    let list = WalkDir::new(src_folder)
        .into_iter()
        .flatten()
        .filter_map(|entry|{
            let path = entry.path();
            let relative = path.strip_prefix(src_folder).unwrap();

            if !options.hidden && is_hidden(relative) {
                return None;
            }
            // if filters.is_match(&as_absolute(relative)) && path.is_file() {
            if filters.is_match(relative.to_str().unwrap()) && path.is_file() {
                Some(Path::new(relative).to_path_buf())
            } else {
                None
            }
        });

    let rename = Rename::try_from(dest_folder);

    if options.flatten {
        let files: Vec::<_> = list.collect();
        if files.len() != 0 {
            std::fs::create_dir_all(dest_folder)?;
        }

        for file in files {
            let mut to_file = dest_folder.join(&file.file_name().unwrap());
            if let Some(rename) = &rename {
                rename.transform(&mut to_file);
            }
            log_trace!("Copy","`{}` to `{}`",to_file.display(), dest_folder.display());
            std::fs::copy(
                src_folder.join(&file),
                to_file
            )?;
        }

    } else {
        let mut folders = HashSet::new();
        let list: Vec::<_> = list.collect();
        for path in list.iter() {
            let folder = path.parent().unwrap();
            folders.insert(folder.to_path_buf());
        }
    
        for folder in folders {
            std::fs::create_dir_all(dest_folder.join(folder))?; 
        }
    
        for file in list {
            let mut to_file = dest_folder.join(&file);
            if let Some(rename) = &rename {
                rename.transform(&mut to_file);
            }
            // println!("+{}",file.display());
            log_trace!("Copy","`{}` to `{}`",file.display(), to_file.display());
            std::fs::copy(src_folder.join(&file),to_file)?;
        }
    }

    Ok(())
}

pub fn is_hidden(path: &std::path::Path) -> bool {
    let is_hidden = path
        .components()
        .find(|f|f.as_os_str().to_string_lossy().starts_with("."))
        .is_some();

    is_hidden
}

pub async fn copy(tpl: &Tpl, copy: &Copy, src_folder: &Path, target_folder: &Path) -> Result<()> 
{

    if let Some(file) = &copy.file {
        if copy.glob.is_some() || copy.regex.is_some() || copy.flatten.is_some() {
            return Err(format!("other options can not be present if `copy.file` is declared").into());
        }
        let from = tpl.transform(&file);
        let to = tpl.transform(&copy.to);

        std::fs::copy(from, to)?;
    } else {
        
        let to_folder = target_folder.join(tpl.transform(&copy.to));
        let mut options = CopyOptions::default();
        options.hidden = copy.hidden.unwrap_or(false);
        options.flatten = true;
        copy_folder_with_filters(
            &src_folder,
            // &dep_build_folder,
            // &target_folder,
            &to_folder,
            (tpl,copy).try_into()?,
            options,
        ).await?;
    }



    Ok(())
}