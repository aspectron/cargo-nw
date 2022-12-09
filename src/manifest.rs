// use serde::Deserialize;
use async_std::fs::*;
use async_std::path::PathBuf;
// use crate::result::Result;
use crate::prelude::*;
// use crate::repository::Repository;
// use crate::build::Build;
// use crate::run::Run;

#[derive(Debug, Clone, Deserialize)]
pub struct Manifest {
    // pub repository: Vec<Repository>,
    // pub build: Option<Vec<Build>>,
    // pub run: Option<Run>,
    pub application : Application,
    pub nwjs : NWJS,
    pub windows : Option<Windows>,
    pub firewall : Option<Firewall>,
    pub languages : Option<Languages>,

    pub build : Option<Vec<Build>>,
    pub deploy : Option<Vec<Deploy>>,
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

    pub async fn locate(location: Option<String>) -> Result<PathBuf> {
        let cwd = current_dir().await;
        if let Some(location) = location {
            let location = cwd.join(location).join("nw.toml");
            if location.exists().await {
                Ok(location)
            } else {
                Err(format!("Unable to locate 'nw.toml' in '{}'", location.display()).into())
            }
        } else {
            let location = cwd.join("nw.toml");
            if location.exists().await {
                Ok(location)
            } else {
                let location = search_upwards(&cwd.clone(), "nw.toml").await;
                location.ok_or(format!("Unable to locate 'nw.toml' in '{}'", cwd.display()).into())
            }
        }
    }
    
    pub async fn load(nwjs_toml : &PathBuf) -> Result<Manifest> {
        // let cwd = current_dir().unwrap();
    
        // let nwjs_toml = read_to_string(cwd.clone().join("nwjs.toml")).await?;
        let nwjs_toml = read_to_string(nwjs_toml).await?;
        // println!("toml: {:#?}", toml);
        let manifest: Manifest = match toml::from_str(&nwjs_toml) {
            Ok(manifest) => manifest,
            Err(err) => {
                return Err(format!("Error loading nwjs.toml: {}", err).into());
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
pub struct Application {
    pub name: String,
    pub title: String,
    pub version: String,
    pub description: String,
    pub organization: String,
    pub authors: Option<String>,
    pub copyright: Option<String>,
    pub trademarks: Option<String>,
    pub resources: Option<String>,
    pub url: Option<String>,
    pub root: Option<String>,
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NWJS {
    pub version: String,
    pub ffmpeg: Option<bool>,
    pub sdk: Option<bool>,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Deserialize)]
pub struct Windows {
    pub uuid: String,
    pub group: String,
    pub executable: Option<String>,
    pub run_on_startup: Option<String>,
    pub run_after_setup: Option<bool>,
    pub setup_icon: Option<String>,

    pub resources : Option<Vec<WindowsResourceString>>
    // ~

    // pub ProductName: Option<String>,
    // pub ProductVersion: Option<String>,
    // pub FileVersion: Option<String>,
    // pub FileDescription: Option<String>,
    // pub CompanyName: Option<String>,
    // pub LegalCopyright: Option<String>,
    // pub LegalTrademarks: Option<String>,
    // pub InternalName: Option<String>,
    // pub OriginalFilename: Option<String>,
    // pub PrivateBuild: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub enum WindowsResourceString {
    ProductName(String),
    ProductVersion(String),
    FileVersion(String),
    FileDescription(String),
    CompanyName(String),
    LegalCopyright(String),
    LegalTrademarks(String),
    InternalName(String),
    Custom { name : String, value : String },

}

#[derive(Debug, Clone, Deserialize)]
pub struct Firewall {
    pub ports: Option<Vec<String>>,
    pub rules: Option<Vec<String>>,
}


#[derive(Debug, Clone, Deserialize)]
pub struct Languages {
    pub languages: Option<Vec<String>>,
}


#[derive(Debug, Clone, Deserialize)]
pub struct Build {
    pub cmd: String,
    pub folder: Option<String>
}

#[derive(Debug, Clone, Deserialize)]
pub struct Deploy {
    pub cmd: String,
    pub folder: Option<String>
}

