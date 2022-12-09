use crate::prelude::*;
use async_std::path::Path;
use serde::{Serialize,Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Snap {
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

impl Snap {
    pub fn new(ctx: &Context) -> Snap {
        let snap = Snap {
            name: ctx.manifest.application.title.clone(), 
            version: ctx.manifest.application.version.clone(),
            summary: ctx.manifest.application.description.clone(),
            description: String::new(),
            confinement: Confinement {
                value: String::new(),
            },
            architectures: Vec::new(),
            apps: Vec::new(),
            plugs: Vec::new(),
        };
        
        snap
    }

    pub fn store(&self, file : &Path) -> Result<()> {
        let yaml = serde_yaml::to_string(self)?;
println!("YAML:{}",yaml);
        Ok(())
    }
}
