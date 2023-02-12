use std::collections::HashSet;

use crate::prelude::*;
use async_std::fs;
use async_std::path::{Path, PathBuf};
use console::style;
use convert_case::{Case, Casing};
use question::{Answer, Question};
use uuid::Uuid;

const DEFAULT_APPLICATION_ICON: &[u8] =
    include_bytes!("../resources/images/default-application-icon.png");
const MACOS_DISK_IMAGE_BACKGROUND: &[u8] =
    include_bytes!("../resources/images/macos-disk-image-background.png");
const INNOSETUP_WIZARD_SMALL_IMAGE: &[u8] =
    include_bytes!("../resources/images/innosetup-wizard-small.png");
const INNOSETUP_WIZARD_LARGE_IMAGE: &[u8] =
    include_bytes!("../resources/images/innosetup-wizard-large.png");
const TRAY_ICON: &[u8] = include_bytes!("../resources/images/tray-icon@2x.png");


mod generic_js {
    pub const NW_TOML: &str = include_str!("../resources/init/generic-rs/nw.toml");
    pub const GITIGNORE: &str = include_str!("../resources/init/generic-rs/.gitignore");
    pub const INDEX_JS: &str = include_str!("../resources/init/generic-rs/index.js");
    pub const INDEX_HTML: &str = include_str!("../resources/init/generic-rs/app/index.html");
    // const CARGO_TOML: &str = include_str!("../resources/init/generic-rs/Cargo.toml");
    // const LIB_RS: &str = include_str!("../resources/init/generic-rs/src/lib.rs");
    // const BUILD_SH: &str = include_str!("../resources/init/generic-rs/build.sh");
    // const BUILD_PS1: &str = include_str!("../resources/init/generic-rs/build.ps1");
}

mod generic_rs {
    pub const NW_TOML: &str = include_str!("../resources/init/generic-rs/nw.toml");
    pub const GITIGNORE: &str = include_str!("../resources/init/generic-rs/.gitignore");
    pub const INDEX_JS: &str = include_str!("../resources/init/generic-rs/index.js");
    pub const INDEX_HTML: &str = include_str!("../resources/init/generic-rs/app/index.html");
    pub const CARGO_TOML: &str = include_str!("../resources/init/generic-rs/Cargo.toml");
    pub const LIB_RS: &str = include_str!("../resources/init/generic-rs/src/lib.rs");
    pub const BUILD_SH: &str = include_str!("../resources/init/generic-rs/build.sh");
    pub const BUILD_PS1: &str = include_str!("../resources/init/generic-rs/build.ps1");
}

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

pub enum TemplateKind {
    GenericRs,
    GenericJs,
}

pub struct Options {
    pub manifest: bool,
    // pub js: bool,
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
}

impl Project {
    pub fn try_new(name: String, folder: PathBuf) -> Result<Project> {
        let name = name.to_case(Case::Kebab);
        let title = name.from_case(Case::Lower).to_case(Case::Title);
        let group = title.clone();
        let version = "0.1.0".to_string();
        let description = "...".to_string();
        let uuid = Uuid::new_v4();

        let project = Project {
            folder,
            name,
            title,
            group,
            version,
            description,
            uuid,
        };

        Ok(project)
    }

    pub async fn generate(&mut self, mut options: Options) -> Result<()> {
        if !options.force && Path::new("nw.toml").exists().await {
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
            options.manifest = true;
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

        match options.template_kind {
            TemplateKind::GenericJs => {
                self.generic_js(&options).await?;
            },
            TemplateKind::GenericRs => {
                self.generic_rs(&options).await?;
            }
        }
        Ok(())

    }

    async fn generic_js(&self, options: &Options) -> Result<()> {
        let tpl = self.tpl()?;
        let files = if options.manifest {
            [("nw.toml", tpl.transform(generic_js::NW_TOML))].to_vec()
        } else {
            let package = PackageJson {
                name: self.title.clone(),
                // main: "app/index.js".to_string(),
                main: "index.js".to_string(),
                version: Some(self.version.clone()),
                description: Some("".to_string()),
            };
            let package_json = serde_json::to_string_pretty(&package).unwrap();

            [
                (".gitignore", generic_js::GITIGNORE.to_string()),
                ("package.json", tpl.transform(&package_json)),
                // ("app/index.js", tpl.transform(INDEX_JS)),
                ("index.js", tpl.transform(generic_js::INDEX_JS)),
                ("app/index.html", tpl.transform(generic_js::INDEX_HTML)),
                // ("root/page2.html", tpl.transform(PAGE2_HTML)),
                // ("src/lib.rs", tpl.transform(LIB_RS)),
                ("nw.toml", tpl.transform(generic_js::NW_TOML)),
                // ("Cargo.toml", tpl.transform(CARGO_TOML)),
                // ("build", tpl.transform(BUILD_SH)),
                // ("build.ps1", tpl.transform(BUILD_PS1)),
            ]
            .to_vec()
        };

        let images = self.images();

        self.create_folders(&files,&images, options).await?;
        self.write_files(&files,&images, options).await?;

        cfg_if! {
            if #[cfg(not(target_os = "windows"))] {
                fs::set_permissions(Path::new("build"), std::os::unix::fs::PermissionsExt::from_mode(0o755)).await?;
            }
        }

        println!("Please run 'build' script to build the project");
        println!(
            "Following this, you can run 'nw .' or 'cargo nw run' to start run the application"
        );
        println!();

        Ok(())
    }

    async fn generic_rs(&self, options: &Options) -> Result<()> {
        let tpl = self.tpl()?;
        let files = if options.manifest {
            [("nw.toml", tpl.transform(generic_rs::NW_TOML))].to_vec()
        } else {
            let package = PackageJson {
                name: self.title.clone(),
                // main: "app/index.js".to_string(),
                main: "index.js".to_string(),
                version: Some(self.version.clone()),
                description: Some("".to_string()),
            };
            let package_json = serde_json::to_string_pretty(&package).unwrap();

            [
                (".gitignore", generic_rs::GITIGNORE.to_string()),
                ("package.json", tpl.transform(&package_json)),
                // ("app/index.js", tpl.transform(INDEX_JS)),
                ("index.js", tpl.transform(generic_rs::INDEX_JS)),
                ("app/index.html", tpl.transform(generic_rs::INDEX_HTML)),
                // ("root/page2.html", tpl.transform(PAGE2_HTML)),
                ("src/lib.rs", tpl.transform(generic_rs::LIB_RS)),
                ("nw.toml", tpl.transform(generic_rs::NW_TOML)),
                ("Cargo.toml", tpl.transform(generic_rs::CARGO_TOML)),
                ("build", tpl.transform(generic_rs::BUILD_SH)),
                ("build.ps1", tpl.transform(generic_rs::BUILD_PS1)),
            ]
            .to_vec()
        };

        let images = self.images();

        self.create_folders(&files,&images, options).await?;
        self.write_files(&files,&images, options).await?;

        cfg_if! {
            if #[cfg(not(target_os = "windows"))] {
                fs::set_permissions(Path::new("build"), std::os::unix::fs::PermissionsExt::from_mode(0o755)).await?;
            }
        }

        println!("Please run 'build' script to build the project");
        println!(
            "Following this, you can run 'nw .' or 'cargo nw run' to start run the application"
        );
        println!();

        Ok(())
    }

    async fn write_files(&self, files : &[(&str, String)], images : &[(&str,&[u8])], options: &Options) -> Result<()> {

        for (filename, content) in files.iter() {
            if !options.force && Path::new(filename).exists().await {
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
    async fn create_folders(&self, files : &[(&str, String)], images : &[(&str,&[u8])], _options: &Options) -> Result<()> {
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
