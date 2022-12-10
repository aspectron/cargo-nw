use async_std::fs::*;
use async_std::path::PathBuf;
use crate::prelude::*;

#[derive(Debug, Clone, Deserialize)]
pub struct Manifest {
    pub application : Application,
    pub nwjs : NWJS,
    pub windows : Option<Windows>,
    pub firewall : Option<Firewall>,
    pub languages : Option<Languages>,

    // pub build : Option<Vec<Build>>,
    // pub deploy : Option<Vec<Deploy>>,
}

impl Manifest {

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
        let nwjs_toml = read_to_string(nwjs_toml).await?;
        let manifest: Manifest = match toml::from_str(&nwjs_toml) {
            Ok(manifest) => manifest,
            Err(err) => {
                return Err(format!("Error loading nwjs.toml: {}", err).into());
            }
        };    

        Ok(manifest)
    }
}

#[derive(Debug, Clone, Deserialize)]
// #[allow(non_camel_case_types)]
pub enum Execute {
    #[serde(rename = "build")]
    Build { cmd : String, folder : Option<String>, platform: Option<String>, arch: Option<String> },
    #[serde(rename = "build")]
    Pack { cmd : String, folder : Option<String>, platform: Option<String>, arch: Option<String>  },
    #[serde(rename = "deploy")]
    Deploy { cmd : String, folder : Option<String>, platform: Option<String>, arch: Option<String>  },

}


#[derive(Debug, Clone, Deserialize)]
pub struct Application {
    pub name: String,
    pub version: String,
    pub title: String,
    pub summary: Option<String>,
    pub description: String,
    pub authors: Option<String>,
    pub organization: String,
    pub copyright: Option<String>,
    pub trademarks: Option<String>,
    pub resources: Option<String>,
    pub url: Option<String>,
    pub root: Option<String>,
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    pub execute: Option<Vec<Execute>>,
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

