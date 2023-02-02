use std::collections::{HashMap, HashSet};

use crate::prelude::*;
use async_std::{
    fs,
    path::{Path, PathBuf},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(non_camel_case_types)]
pub enum SnapArchitecture {
    amd64,
    i386,
    aarch64,
}

impl From<Architecture> for SnapArchitecture {
    fn from(arch: Architecture) -> Self {
        match arch {
            Architecture::ia32 => SnapArchitecture::i386,
            Architecture::x64 => SnapArchitecture::amd64,
            Architecture::aarch64 => SnapArchitecture::aarch64,
        }
    }
}

impl ToString for SnapArchitecture {
    fn to_string(&self) -> String {
        match self {
            SnapArchitecture::amd64 => "amd64",
            SnapArchitecture::i386 => "i386",
            SnapArchitecture::aarch64 => "aarch64",
        }
        .to_string()
    }
}

#[derive(Serialize, Deserialize)]
pub struct Parts(HashMap<String, Part>);

impl Parts {
    pub fn new(parts: &[(&str, Part)]) -> Parts {
        let mut list = HashMap::new();
        for (src, part) in parts {
            list.insert(src.to_string(), part.clone());
        }
        Parts(list)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Plugin {
    #[serde(rename = "dump")]
    Dump,
    #[serde(rename = "nil")]
    Nil,
}
impl Default for Plugin {
    fn default() -> Plugin {
        Plugin::Nil
    }
}
impl ToString for Plugin {
    fn to_string(&self) -> String {
        match self {
            Plugin::Dump => "dump".to_string(),
            Plugin::Nil => "nil".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Part {
    plugin: Plugin,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "stage-packages")]
    stage_packages: Option<HashSet<String>>,
}

impl Part {
    pub fn new_with_source(source: &str, plugin: Plugin) -> Part {
        Part {
            source: Some(source.to_string()),
            plugin,
            stage_packages: None,
        }
    }

    pub fn dependencies(packages: Option<HashSet<String>>) -> Part {
        let mut list = vec![
            "libnspr4",
            "libnss3",
            "libx11-6",
            "libxext6",
            "libwayland-client0",
            // "libatomic1-amd64-cross",
            "libatomic1",
            "libxcomposite1",
            "libxdamage1",
            "libxfixes3",
            "libxrandr2",
            "libasound2",
            "libatk1.0-0",
            "libatspi2.0-0",
            "libcairo2",
            "libcups2",
            "libgbm1",
            "libpango1.0-0",
            // needed for strict
            "libxkbcommon0",
            "libx11-xcb1",
            "libgl1",
            "mesa-utils",
            "libgl1-mesa-glx",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect::<HashSet<String>>();

        if let Some(packages) = packages {
            list.extend(packages);
        }

        Part {
            plugin: Plugin::Nil,
            source: None,
            stage_packages: Some(list),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Apps(HashMap<String, App>);

impl Apps {
    pub fn new(apps: &[(&str, App)]) -> Self {
        let mut list = HashMap::new();
        for (name, app) in apps.iter() {
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
    desktop: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    plugs: Option<HashSet<String>>,
}

impl App {
    pub fn new(folder: &str, name: &str, interfaces: Option<HashSet<String>>) -> App {
        let mut plugs = vec![
            "browser-support",
            "network",
            "network-bind",
            // ~
            "opengl",
            "x11",
            "upower-observe",
            // ~
            // "gsettings",
            // "desktop", //?
            // "desktop-legacy", //?
            // ~
            // "removable-media",
            // "personal-files",
            // "optical-drive",
            // "personal-files",
            // "home",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect::<HashSet<String>>();

        if let Some(interfaces) = interfaces {
            plugs.extend(interfaces);
        }

        App {
            command: format!("./{folder}/{name}"),
            desktop: format!("./{folder}/{name}.desktop"),
            plugs: Some(plugs),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SnapData {
    pub name: String,
    pub title: String,
    pub version: String,
    pub summary: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    // #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    pub base: String,
    pub grade: Channel,
    pub confinement: Confinement,
    pub architectures: Vec<SnapArchitecture>,
    pub apps: Apps,
    pub parts: Parts,
}

impl SnapData {
    pub fn new(ctx: &Context, target_file: &str) -> SnapData {
        let user_snap = ctx.manifest.snap.clone().unwrap_or_default();

        let name = ctx.manifest.application.name.clone();
        let parts = Parts::new(&[
            (
                name.as_str(),
                Part::new_with_source(target_file, Plugin::Dump),
            ),
            (
                "dependencies",
                Part::dependencies(user_snap.packages.clone()),
            ),
        ]);

        let apps = Apps::new(&[(
            name.as_str(),
            App::new(&ctx.app_snake_name, &name, user_snap.interfaces.clone()),
        )]);

        let snap = SnapData {
            name: name.clone(),
            title: ctx.manifest.application.title.clone(),
            version: ctx.manifest.application.version.clone(),
            summary: ctx.manifest.description.short.clone(),
            description: ctx.manifest.description.long.clone(),
            website: ctx.manifest.application.url.clone(),
            icon: format!("./{}/{}.png", ctx.app_snake_name, name),
            license: ctx.manifest.application.license.clone(),
            base: user_snap.base.unwrap_or("core22".to_string()),
            grade: ctx.channel.clone(),
            confinement: ctx.confinement.clone(),
            architectures: vec![ctx.arch.clone().into()],
            apps,
            parts,
        };

        snap
    }

    pub async fn store(&self, file: &Path) -> Result<()> {
        let yaml = serde_yaml::to_string(self)?;
        fs::write(file, yaml).await?;
        Ok(())
    }
}

pub struct Snap {
    pub data: SnapData,
    pub ctx: Arc<Context>,
    pub archive_filepath: PathBuf,
    pub archive_filename: String,
}

impl Snap {
    pub fn try_new(ctx: &Arc<Context>, archive_filepath: &Path) -> Result<Snap> {
        let archive_filename = archive_filepath.file_name().unwrap().to_str().unwrap();

        // let archive_filename =
        let snap = Snap {
            data: SnapData::new(ctx, archive_filename),
            archive_filepath: archive_filepath.to_path_buf(),
            archive_filename: archive_filename.to_string(),
            ctx: ctx.clone(),
        };

        Ok(snap)
    }

    pub async fn create(&self) -> Result<()> {
        self.data
            .store(&self.ctx.build_folder.join("snapcraft.yaml"))
            .await?;
        Ok(())
    }
    pub async fn build(&self) -> Result<PathBuf> {
        std::fs::copy(
            &self.archive_filepath,
            self.ctx.build_folder.join(&self.archive_filename),
        )?;

        log_info!("Snap", "generating ...");

        cmd!("snapcraft").dir(&self.ctx.build_folder).run()?;

        let snap_filename = format!(
            "{}_{}_{}.snap",
            // let filename = format!("{}-{}-{}",
            self.data.name,
            self.ctx.manifest.application.version,
            "amd64"
        );

        fs::rename(
            self.ctx.build_folder.join(&snap_filename),
            self.ctx.output_folder.join(&snap_filename),
        )
        .await?;

        Ok(self.ctx.output_folder.join(&snap_filename))
    }
}
