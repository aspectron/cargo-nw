use std::env::current_dir;
// use serde::Deserialize;
use async_std::fs::*;
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


    pub async fn load() -> Result<Manifest> {
        let cwd = current_dir().unwrap();
    
        let nwjs_toml = read_to_string(cwd.clone().join("nwjs.toml")).await?;
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
    // port: Option<u64>,
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
    pub folder: String
}

impl Build {
    pub async fn execute(&self) -> Result<&Self> {
        let cwd = std::env::current_dir()?;

        let argv : Vec<String> = self.cmd.split(" ").map(|s|s.to_string()).collect();
        let program = argv.first().expect("missing program in build config");
        let args = argv[1..].to_vec();
        cmd(program,args).dir(cwd.join(&self.folder)).run()?;

        Ok(self)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Deploy {
    pub cmd: String,
}

