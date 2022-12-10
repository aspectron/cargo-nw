use async_std::fs::*;
use async_std::path::PathBuf;
use crate::prelude::*;
use regex::Regex;

#[derive(Debug, Clone, Deserialize)]
pub struct Manifest {
    pub application : Application,
    pub description : Description,
    pub package : Package,
    pub nwjs : NWJS,
    pub windows : Option<Windows>,
    pub firewall : Option<Firewall>,
    pub languages : Option<Languages>,
}

impl Manifest {

    pub async fn locate(location: Option<String>, manifest : Option<String>) -> Result<PathBuf> {
        let manifest = manifest.unwrap_or("nw.toml".to_string());
        let cwd = current_dir().await;
        if let Some(location) = location {
            let location = cwd.join(location).join(&manifest);
            if location.exists().await {
                Ok(location)
            } else {
                Err(format!("Unable to locate 'nw.toml' in '{}'", location.display()).into())
            }
        } else {
            let location = cwd.join(&manifest);
            if location.exists().await {
                Ok(location)
            } else {
                let location = search_upwards(&cwd.clone(), &manifest).await;
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

        manifest.sanity_checks()?;

        Ok(manifest)
    }
    
    pub fn sanity_checks(&self) -> Result<()> {

        let regex = Regex::new(r"^[^\s]*[a-z0-9-_]*$").unwrap();
        if !regex.is_match(&self.application.name) {
            return Err(format!("invalid application name '{}'", self.application.name).into());
        }

        Ok(())
    }
}


#[derive(Debug, Clone, Deserialize)]
pub struct Application {
    pub name: String,
    pub version: String,
    pub title: String,
    pub authors: Option<String>,
    pub organization: String,
    pub copyright: Option<String>,
    pub trademarks: Option<String>,
    pub url: Option<String>,
}


#[derive(Debug, Clone, Deserialize)]
pub struct Description {
    pub short: String,
    pub long: String,
}


#[derive(Debug, Clone, Deserialize)]
// #[allow(non_camel_case_types)]
pub enum Execute {
    #[serde(rename = "build")]
    Build { 
        cmd : String,
        folder : Option<String>,
        platform: Option<String>,
        arch: Option<String>,
        env : Option<Vec<String>>,
    },
    #[serde(rename = "build")]
    Pack {
        cmd : String,
        folder : Option<String>,
        platform: Option<String>,
        arch: Option<String>,
        env : Option<Vec<String>>,
    },
    #[serde(rename = "deploy")]
    Deploy {
        cmd : String,
        folder : Option<String>,
        platform: Option<String>,
        arch: Option<String>,
        env : Option<Vec<String>>,
    },
    #[serde(rename = "publish")]
    /// Esecution invoked when running `cargo nw publish`
    Publish {
        cmd : String,
        folder : Option<String>,
        platform: Option<String>,
        arch: Option<String>,
        env : Option<Vec<String>>,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub enum Build {
    WASM { 
        clean : Option<bool>,
        purge : Option<bool>,
        dev : Option<bool>,
        folder : Option<String>,
        args : Option<String>,
        env : Option<Vec<String>>,
    },
    NPM {
        clean : Option<bool>,
        dev : Option<bool>,
        args : Option<String>,
        env : Option<Vec<String>>,
    },
    #[serde(rename = "custom")]
    Custom {
        cmd : String,
        env : Option<Vec<String>>,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct Package {
    pub gitignore: Option<bool>,
    pub build: Option<Vec<Build>>,
    pub archive: Option<Archive>,
    pub signatures: Option<bool>,
    pub resources: Option<String>,
    pub root: Option<String>,
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    pub hidden: Option<bool>,
    pub execute: Option<Vec<Execute>>,
    // pub output: Option<String>,
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

// ~~~

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageJson {
    pub name : String,
    pub main : String,
    pub description : Option<String>,
    pub version : Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Archive {
    STORE,
    BZIP2,
    DEFLATE,
    ZSTD
}

impl Default for Archive {
    fn default() -> Self {
        Archive::ZSTD
    }
}

impl Into<zip::CompressionMethod> for Archive {
    fn into(self) -> zip::CompressionMethod {
        match self {
            Archive::STORE => zip::CompressionMethod::Stored,
            Archive::BZIP2 => zip::CompressionMethod::Bzip2,
            Archive::DEFLATE => zip::CompressionMethod::Deflated,
            Archive::ZSTD => zip::CompressionMethod::Zstd,
        }
    }
}
