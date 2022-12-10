use crate::prelude::*;
use async_std::{path::Path, fs};
use serde::{Serialize,Deserialize};

#[derive(Serialize, Deserialize)]
pub struct SnapData {
    name: String,
    version: String,
    summary: String,
    description: String,
    confinement: Confinement,
    architectures: Vec<String>,
    apps: Vec<App>,
    plugs: Vec<Plug>,
}

#[derive(Serialize, Deserialize)]
pub struct Confinement {
    value: String,
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
    pub fn new(ctx: &Context) -> SnapData {
        let snap = SnapData {
            name: ctx.manifest.application.title.clone(), 
            version: ctx.manifest.application.version.clone(),
            summary: ctx.manifest.description.short.clone(),
            description: ctx.manifest.description.long.clone(),
            confinement: Confinement {
                value: String::new(),
            },
            architectures: Vec::new(),
            apps: Vec::new(),
            plugs: Vec::new(),
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
    data: SnapData,
    ctx: Arc<Context>, 
}

impl Snap {
    pub fn new(ctx: &Arc<Context>) -> Snap {
        Snap {
            data: SnapData::new(&ctx),
            ctx : ctx.clone()
        }
    }

    pub async fn create(&self) -> Result<()> {

        self.data.store(&self.ctx.build_folder.join("snap.yaml")).await?;



        Ok(())
    }
}

