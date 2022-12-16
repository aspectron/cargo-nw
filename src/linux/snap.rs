use std::collections::HashMap;

use crate::prelude::*;
use async_std::{path::{Path, PathBuf}, fs};
use serde::{Serialize,Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(non_camel_case_types)]
pub enum SnapArchitecture {
    amd64,
    x86,
    aarch64,
}

impl From<Architecture> for SnapArchitecture {
    fn from(arch: Architecture) -> Self {
        match arch {
            Architecture::ia32 => SnapArchitecture::x86,
            Architecture::x64 => SnapArchitecture::amd64,
            Architecture::aarch64 => SnapArchitecture::aarch64,
        }
    }
}

impl ToString for SnapArchitecture {
    fn to_string(&self) -> String {
        match self {
            SnapArchitecture::amd64 => "amd64",
            SnapArchitecture::x86 => "x86",
            SnapArchitecture::aarch64 => "aarch64",
        }.to_string()
    }
}

#[derive(Serialize, Deserialize)]
pub struct SnapData {
    pub name: String,
    pub version: String,
    pub summary: String,
    pub description: String,
    pub base : String,
    pub grade: Channel,
    pub confinement: Confinement,
    pub architectures: Vec<SnapArchitecture>,
    // apps: Vec<App>,
    // plugs: Vec<Plug>,
    pub apps: Apps,
    pub parts: Parts,

}


#[derive(Serialize, Deserialize)]
pub struct Parts(HashMap<String, Part>);

impl Parts {
    pub fn new(parts : &[(&str, Part)]) -> Parts {
        let mut list = HashMap::new();
        for (src, part) in parts {
            list.insert(src.to_string(), part.clone());
        }
        Parts(list)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Plugin {
    #[serde(rename = "nil")]
    Nil
}
impl Default for Plugin {
    fn default() -> Plugin {
        Plugin::Nil
    }
}
impl ToString for Plugin {
    fn to_string(&self) -> String {
        match self {
            Plugin::Nil => "nil".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Part {
    source : String,
    plugin : Plugin,
}

impl Part {
    pub fn new(source : &str, plugin : Plugin) -> Part {
        Part {
            source : source.to_string(),
            plugin : plugin,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Apps(HashMap<String,App>);

impl Apps {
    pub fn new(apps: &[(&str, App)]) -> Self {
        let mut list = HashMap::new();
        for (name,app) in apps.iter() {
            list.insert(name.to_string(), app.clone());
        }

        Apps(list)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "app")]
pub struct App {
    // name: String,
    command: String,
    // plugs: Vec<String>,
}

impl App {
    pub fn new(command: &str) -> App {
        App {
            // name: name.to_string(),
            command: command.to_string(),
        }
    }
}

// #[derive(Serialize, Deserialize)]
// pub struct Plug {
//     name: String,
//     interface: String,
//     attrs: Vec<Attr>,
// }

// #[derive(Serialize, Deserialize)]
// pub struct Attr {
//     key: String,
//     value: String,
// }

impl SnapData {
    pub fn new(ctx: &Context, target_file: &str) -> SnapData {

        let name = ctx.manifest.application.name.clone();
        let parts = Parts::new(&[
            (name.as_str(), Part::new(target_file, Plugin::Nil))
        ]);
        let apps = Apps::new(&[
            (name.as_str(), App::new(&format!("./{}",name)))
        ]);

        let snap = SnapData {
            name,//ctx.manifest.application.name.clone(), 
            version: ctx.manifest.application.version.clone(),
            summary: ctx.manifest.description.short.clone(),
            description: ctx.manifest.description.long.clone(),
            base: "core22".to_string(),
            grade: ctx.channel.clone(),
            confinement: ctx.confinement.clone(),
            // TODO
            architectures: vec![ctx.arch.clone().into()],
            apps,
            parts,
            // apps: Vec::new(),
            // plugs: Vec::new(),
        };
        
        snap
    }

    pub async fn store(&self, file : &Path) -> Result<()> {
        let yaml = serde_yaml::to_string(self)?;
        fs::write(file,yaml).await?;
        Ok(())
    }
}


pub struct Snap {
    pub data: SnapData,
    pub ctx: Arc<Context>,
    pub archive_filepath : PathBuf,
    pub archive_filename : String,
}

impl Snap {
    pub fn try_new(ctx: &Arc<Context>, archive_filepath : &Path) -> Result<Snap> {

        let archive_filename = archive_filepath.file_name().unwrap().to_str().unwrap();

        // let archive_filename = 
        let snap = Snap {
            data: SnapData::new(&ctx, archive_filename),
            archive_filepath : archive_filepath.to_path_buf(),
            archive_filename : archive_filename.to_string(),
            ctx : ctx.clone(),
        };

        Ok(snap)
    }

    pub async fn create(&self) -> Result<()> {
        self.data.store(&self.ctx.build_folder.join("snapcraft.yaml")).await?;
        Ok(())
    }
    pub async fn build(&self) -> Result<PathBuf> {

        std::fs::copy(&self.archive_filepath, self.ctx.build_folder.join(&self.archive_filename))?;

        log_info!("SNAP","generating ...");

        cmd!("snapcraft").dir(&self.ctx.build_folder).run()?;

        let snap_filename = format!("{}_{}_{}.snap",
        // let filename = format!("{}-{}-{}",
            self.data.name,
            self.data.version,
            "amd64"
        );

        Ok(self.ctx.build_folder.join(snap_filename))

    }
}

