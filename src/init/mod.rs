mod generic_js;
mod generic_rs;

use crate::prelude::*;
use async_std::fs;
use async_std::path::{Path, PathBuf};
use console::style;
use convert_case::{Case, Casing};
use question::{Answer, Question};
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use uuid::Uuid;

const DEFAULT_APPLICATION_ICON: &[u8] =
    include_bytes!("../../resources/images/default-application-icon.png");
const MACOS_DISK_IMAGE_BACKGROUND: &[u8] =
    include_bytes!("../../resources/images/macos-disk-image-background.png");
const INNOSETUP_WIZARD_SMALL_IMAGE: &[u8] =
    include_bytes!("../../resources/images/innosetup-wizard-small.png");
const INNOSETUP_WIZARD_LARGE_IMAGE: &[u8] =
    include_bytes!("../../resources/images/innosetup-wizard-large.png");
const TRAY_ICON: &[u8] = include_bytes!("../../resources/images/tray-icon@2x.png");

// const PAGE2_HTML: &str = r###"
// <!DOCTYPE html>
// <html>
//     <head>
//         <!--title>new window test</title-->
//     </head>
//     <body>
//         <h1>$TITLE (Window 2)</h1>
//         <script>
//             console.log("nw", nw);
//         </script>
//     </body>
// </html>
// "###;

#[derive(Debug)]
pub enum TemplateKind {
    GenericRs,
    GenericJs,
}

#[derive(Debug)]
pub struct Options {
    pub manifest: bool,
    pub template_kind: TemplateKind,
    pub force: bool,
}

#[derive(Debug)]
pub struct Project {
    pub folder: PathBuf,
    pub name: String,
    pub title: String,
    pub group: String,
    pub version: String,
    pub description: String,
    pub uuid: Uuid,
    pub options: Options,
    pub manifest: AtomicBool,
}

impl Project {
    pub fn try_new(name: String, folder: PathBuf, options: Options) -> Result<Project> {
        let name = name.to_case(Case::Kebab);
        let title = name.from_case(Case::Lower).to_case(Case::Title);
        let group = title.clone();
        let version = "0.1.0".to_string();
        let description = "...".to_string();
        let uuid = Uuid::new_v4();
        let manifest = AtomicBool::new(options.manifest);

        let project = Project {
            folder,
            name,
            title,
            group,
            version,
            description,
            uuid,
            options,
            manifest,
        };

        Ok(project)
    }

    pub async fn generate(&mut self) -> Result<()> {
        if !self.options.force && Path::new("nw.toml").exists().await {
            return Err("existing nw.toml found ...aborting (use --force to re-create)".into());
        }

        if Path::new("package.json").exists().await {
            let package_json = PackageJson::try_load("package.json")?;
            self.name = package_json.name.to_lowercase().replace(' ', "-");
            self.title = package_json.name;
            if let Some(version) = package_json.version {
                self.version = version;
            }
            if let Some(description) = package_json.description {
                self.description = description;
            }

            log_info!("Project", "detected existing 'package.json' manifest");
            log_info!(
                "Project",
                "name: '{}' title: '{}' version: '{}'",
                self.name,
                self.title,
                self.version
            );
            self.manifest.store(true, Ordering::SeqCst);
        } else {
            let name = Question::new(&format!(
                "Project name [default:'{}']:",
                style(&self.name).yellow()
            ))
            .ask();
            if let Some(Answer::RESPONSE(name)) = name {
                if !name.is_empty() {
                    if name.contains(' ') {
                        println!(
                            "{}",
                            style("\nError: project name can not contain spaces\n").red()
                        );
                        std::process::exit(1);
                    }

                    let name = name.to_case(Case::Kebab);
                    if name != self.name {
                        self.title = name
                            .replace('-', " ")
                            .from_case(Case::Kebab)
                            .to_case(Case::Title);
                    }

                    self.name = name;
                }
            }
            let title = Question::new(&format!(
                "Project title [default:'{}']:",
                style(&self.title).yellow()
            ))
            .ask();
            if let Some(Answer::RESPONSE(title)) = title {
                if !title.is_empty() {
                    self.title = title;
                }
            }
        }

        println!();
        log_info!("Init", "creating '{}'", self.name);
        println!();

        let manifest = self.manifest.load(Ordering::SeqCst);
        match self.options.template_kind {
            TemplateKind::GenericJs => {
                generic_js::generate(self, manifest).await?;
            }
            TemplateKind::GenericRs => {
                generic_rs::generate(self, manifest).await?;
            }
        }
        Ok(())
    }

    async fn write_files(&self, files: &[(&str, String)], images: &[(&str, &[u8])]) -> Result<()> {
        for (filename, content) in files.iter() {
            if !self.options.force && Path::new(filename).exists().await {
                log_warn!(
                    "Init",
                    "WARNING: file already exists! `{}` (use --force to overwrite) skipping...",
                    filename
                );
            }
            fs::write(filename, &content).await?;
        }

        for (filename, data) in images.iter() {
            fs::write(filename, data).await?;
        }

        Ok(())
    }
    async fn create_folders(
        &self,
        files: &[(&str, String)],
        images: &[(&str, &[u8])],
    ) -> Result<()> {
        let folders: HashSet<&Path> = files
            .iter()
            .map(|(f, _)| f)
            .chain(images.iter().map(|(f, _)| f))
            .filter_map(|path| Path::new(path).parent())
            .collect();

        for folder in folders {
            fs::create_dir_all(folder).await?;
        }

        Ok(())
    }

    fn images(&self) -> Vec<(&'static str, &'static [u8])> {
        vec![
            ("resources/setup/application.png", DEFAULT_APPLICATION_ICON),
            ("resources/setup/document.png", DEFAULT_APPLICATION_ICON),
            (
                "resources/setup/macos-application.png",
                DEFAULT_APPLICATION_ICON,
            ),
            (
                "resources/setup/macos-dmg-background.png",
                MACOS_DISK_IMAGE_BACKGROUND,
            ),
            (
                "resources/setup/innosetup-wizard-small.png",
                INNOSETUP_WIZARD_SMALL_IMAGE,
            ),
            (
                "resources/setup/innosetup-wizard-large.png",
                INNOSETUP_WIZARD_LARGE_IMAGE,
            ),
            ("resources/icons/tray-icon@2x.png", TRAY_ICON),
        ]
    }

    fn tpl(&self) -> Result<Tpl> {
        let tpl: Tpl = [
            ("NAME", self.name.clone()),
            (
                "SNAKE",
                self.name.from_case(Case::Kebab).to_case(Case::Snake),
            ),
            ("TITLE", self.title.clone()),
            ("UUID", self.uuid.to_string()),
            ("VERSION", self.version.to_string()),
            ("DESCRIPTION", self.description.to_string()),
        ]
        .as_slice()
        .try_into()?;

        Ok(tpl)
    }

    // async fn create_package_json(&self, ctx: &Context) -> Result<()> {
    //     log!("MacOS","creating package.json");
    //     let package_json = PackageJson {
    //         name : ctx.manifest.application.title.clone(),
    //         main : "index.js".to_string(),
    //     };
    //     let json = serde_json::to_string(&package_json).unwrap();
    //     fs::write(&self.folder.join("package.json"), json).await?;
    //     Ok(())
    // }
}
