use std::collections::HashMap;

use crate::prelude::*;
use async_std::{path::{Path, PathBuf}, fs};
use serde::{Serialize,Deserialize};

#[derive(Serialize, Deserialize)]
pub struct SnapData {
    pub name: String,
    pub version: String,
    pub summary: String,
    pub description: String,
    pub base : String,
    pub grade: Channel,
    pub confinement: Confinement,
    pub architectures: Vec<String>,
    // apps: Vec<App>,
    // plugs: Vec<Plug>,
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
pub struct App {
    name: String,
    command: String,
    plugs: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Plug {
    name: String,
    interface: String,
    attrs: Vec<Attr>,
}

#[derive(Serialize, Deserialize)]
pub struct Attr {
    key: String,
    value: String,
}

impl SnapData {
    pub fn new(ctx: &Context, target_file: &Path) -> SnapData {

        let name = ctx.manifest.application.name.clone();
        let parts = Parts::new(&[(name.as_str(), Part::new(target_file.to_str().unwrap(), Plugin::Nil))]);

        let snap = SnapData {
            name,//ctx.manifest.application.name.clone(), 
            version: ctx.manifest.application.version.clone(),
            summary: ctx.manifest.description.short.clone(),
            description: ctx.manifest.description.long.clone(),
            base: "core22".to_string(),
            grade: ctx.channel.clone(),
            confinement: ctx.confinement.clone(),
            // TODO
            architectures: vec!["amd64".to_string()],
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
}

impl Snap {
    pub fn new(ctx: &Arc<Context>, target_file : &Path) -> Snap {
        Snap {
            data: SnapData::new(&ctx, target_file),
            ctx : ctx.clone(),
        }
    }

    pub async fn create(&self) -> Result<()> {
        self.data.store(&self.ctx.build_folder.join("snapcraft.yaml")).await?;
        Ok(())
    }
    pub async fn build(&self) -> Result<()> {

        // let snap_file = format!("{}_{}_amd64",self.ctx.manifest.application.name,self.ctx.manifest.application.version);
        // let snap_target_file = format!("{}-{}-amd64.zip",self.ctx.manifest.application.name,self.ctx.manifest.application.version);
        // let snap_target_file = format!("{}_{}_amd64.zip",self.ctx.manifest.application.name,self.ctx.manifest.application.version);

        // fs::copy(
        //     self.ctx.output_folder.join(format!("{}.zip",self.ctx.app_snake_name)),
        //     self.ctx.build_folder.join(&snap_target_file)
        // ).await?;

        // let tar_filename = format!("{}.tar.gz",snap_file);
        // let tar_path = self.ctx.build_folder.join(tar_filename);
        // cmd("tar",["-czf",tar_path.to_str().unwrap(),self.ctx.output_folder.to_str().unwrap()]).run()?;
        // log_info!("Archiving","`{}`",tar_path.display());

        cmd!("snapcraft").dir(&self.ctx.build_folder).run()?;
        Ok(())
    }
    pub fn file_name(&self) -> Result<PathBuf> {

        let filename = format!("{}_{}_{}",
        // let filename = format!("{}-{}-{}",
            self.data.name,
            self.data.version,
            "amd64"
        );

        Ok(self.ctx.build_folder.join(filename))
    }
}

