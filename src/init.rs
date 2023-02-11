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

const GITIGNORE: &str = include_str!("../resources/init/generic-rs/.gitignore");
const INDEX_JS: &str = include_str!("../resources/init/generic-rs/index.js");
const INDEX_HTML: &str = include_str!("../resources/init/generic-rs/app/index.html");
const NW_TOML: &str = include_str!("../resources/init/generic-rs/nw.toml");
const CARGO_TOML: &str = include_str!("../resources/init/generic-rs/Cargo.toml");
const LIB_RS: &str = include_str!("../resources/init/generic-rs/src/lib.rs");
const BUILD_SH: &str = include_str!("../resources/init/generic-rs/build.sh");
const BUILD_PS1: &str = include_str!("../resources/init/generic-rs/build.ps1");

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

pub struct Options {
    pub manifest: bool,
    pub js: bool,
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

        // println!("{self:?}");

        let tpl = self.tpl()?;
        let files = if options.manifest {
            [("nw.toml", tpl.transform(NW_TOML))].to_vec()
        } else {
            let package = PackageJson {
                name: self.title.clone(),
                main: "app/index.js".to_string(),
                version: Some(self.version.clone()),
                description: Some("".to_string()),
            };
            let package_json = serde_json::to_string_pretty(&package).unwrap();

            [
                (".gitignore", GITIGNORE.to_string()),
                ("package.json", tpl.transform(&package_json)),
                // ("app/index.js", tpl.transform(INDEX_JS)),
                ("index.js", tpl.transform(INDEX_JS)),
                ("app/index.html", tpl.transform(INDEX_HTML)),
                // ("root/page2.html", tpl.transform(PAGE2_HTML)),
                ("src/lib.rs", tpl.transform(LIB_RS)),
                ("nw.toml", tpl.transform(NW_TOML)),
                ("Cargo.toml", tpl.transform(CARGO_TOML)),
                ("build", tpl.transform(BUILD_SH)),
                ("build.ps1", tpl.transform(BUILD_PS1)),
            ]
            .to_vec()
        };

        //             const MACOS_DMG_BACKGROUND: &[u8] = include_bytes!("../resources/macos-dmg-background.png");
        // const INNOSETUP_55x58_IMAGE: &[u8] = include_bytes!("../resources/innosetup-55x58.bmp");
        // const INNOSETUP_164x314_IMAGE: &[u8] = include_bytes!("../resources/innosetup-164x314.bmp");

        let images = [
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
        ];

        let folders: HashSet<&Path> = files
            .iter()
            .map(|(f, _)| f)
            .chain(images.iter().map(|(f, _)| f))
            .filter_map(|path| Path::new(path).parent())
            .collect();

        for folder in folders {
            fs::create_dir_all(folder).await?;
        }

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
