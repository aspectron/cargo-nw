use std::env::current_dir;
use serde_derive::Deserialize;
use async_std::fs::*;
use crate::result::Result;
// use crate::repository::Repository;
// use crate::build::Build;
// use crate::run::Run;

#[derive(Debug, Clone, Deserialize)]
pub struct Manifest {
    // pub repository: Vec<Repository>,
    // pub build: Option<Vec<Build>>,
    // pub run: Option<Run>,
    pub package : Package,
    pub nwjs : Nwjs,
}

impl Manifest {
    // pub fn application_name(&self) -> String {
    //     match &self.emanate.name {
    //         Some(name) => name.clone(),
    //         None => {
    //             println!("WARNING: manifest is missing [emanate].name section");
    //             self.package.name.clone()
    //         }
    //     }
    // }

    // pub fn application_ident(&self) -> String {
    //     self.package.name.clone()
    // }


    pub async fn load() -> Result<Manifest> {
        let cwd = current_dir().unwrap();
    
        let nwjs_toml = read_to_string(cwd.clone().join("nwjs.toml")).await?;
        // println!("toml: {:#?}", toml);
        let manifest: Manifest = match toml::from_str(&nwjs_toml) {
            Ok(manifest) => manifest,
            Err(err) => {
                panic!("Error loading nwjs.toml: {}", err);
            }
        };    

        Ok(manifest)
    }
}

// #[derive(Debug, Clone, Deserialize)]
// pub struct PackageConfig {
//     pub name: String,
//     pub version: String,
//     pub authors: Vec<String>,
//     pub description: Option<String>,
//     // port: Option<u64>,
// }



// #[derive(Debug, Clone, Deserialize)]
// pub struct ProjectConfig {
// }



#[derive(Debug, Clone, Deserialize)]
pub struct Package {
    pub name: String,
    pub title: String,
    pub version: String,
    pub resources: String,
    // port: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Nwjs {
    pub version: String,
    pub ffmpeg: Option<bool>,
}

