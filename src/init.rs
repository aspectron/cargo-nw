use std::collections::HashSet;

use crate::prelude::*;
use async_std::path::{PathBuf, Path};
use async_std::fs;
use convert_case::{Case, Casing};
use question::{Answer, Question};
use console::style;
use uuid::Uuid;


const INDEX_JS: &str = r###"
(async()=>{
    window.$$SNAKE = await import('../wasm/$NAME.js');
    // window.$$SNAKE = $$NAME;
    const wasm = await window.$$SNAKE.default('/wasm/$NAME_bg.wasm');
    //console.log("wasm", wasm, workflow)
    //$$SNAKE.init_console_panic_hook();
    //$$SNAKE.show_panic_hook_logs();
    window.$$SNAKE.initialize();
})();

"###;

const INDEX_HTML: &str = r###"

(async()=>{
    let $$SNAKE = await import('../wasm/$NAME.js');
    const wasm = await $$NAME.default('/wasm/$NAME_bg.wasm'); 
    $$SNAKE.run();
    // console.log("create_context_menu", $$SNAKE.create_context_menu())
})();
"###;

const NW_TOML: &str = r###"

# nw.toml - for additional properties please see https://github.com/aspectron/cargo-nw

[application]
name = "$NAME"
version = "$VERSION"
title = "$TITLE"
organization = "Your Organization Name"

[description]
short = "..."
long = """
$DESCRIPTION
""""

[package]
# root = ""
# resources = "resources/setup"
exclude = ["resources/setup"]

[node-webkit]
version = "0.71.0"
ffmpeg = false

[dmg]
# window = ["0,0,300,300"]
# icon = ["0,0"]
# applications = ["0,0"]

[windows]
uuid = "$UUID"
group = "$GROUP"
# run_on_startup = "everyone"
run_after_setup = true

# [languages]
# languages = ["english"]

# [firewall]
# application = "in:out"

"###;

const CARGO_TOML: &str = r###"
[package]
name = "$NAME"
version = "$VERSION"
edition = "2021"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]

# Make sure to update crate versions before production use
# Use https://crates.io to get the latest versions

wasm_bindgen = "*"
nw-sys = "*"
workflow-log = "*"
workflow-wasm = "*"
workflow-panic-hook = "*"

"###;

const LIB_RS: &str = r###"
use wasm_bindgen::prelude::*;
use workflow_log::log_trace;
use workflow_wasm::listener::Listener;
use nw_sys::result::Result;
use nw_sys::prelude::*;
use nw_sys::utils;

static mut APP:Option<Arc<ExampleApp>> = None;
#[derive(Clone)]
pub struct ExampleApp {
    // pub win_listeners:Arc<Mutex<Vec<Listener<nw::Window>>>>,
    // pub menu_listeners:Arc<Mutex<Vec<Listener<JsValue>>>>,
    // pub listeners:Arc<Mutex<Vec<Listener<web_sys::MouseEvent>>>>
}

"###;


const BUILD_SH: &str = r###"
if [ "$1" = "--dev" ]; then
    wasm-pack build --dev --target web --out-name $NAME --out-dir root/wasm
else
    wasm-pack build --target web --out-name $NAME --out-dir root/wasm
fi
"###;

const BUILD_PS1: &str = r###"
if ($args.Contains("--dev")) {
    & "wasm-pack build --dev --target web --out-name $NAME --out-dir root/wasm"
} else {
    & "wasm-pack build --target web --out-name $NAME --out-dir root/wasm"
}
"###;


pub struct Options {
    pub manifest: bool,
    pub js : bool,
    pub force : bool,
}

#[derive(Debug)]
pub struct Project {
    pub folder : PathBuf,
    pub name : String,
    pub title : String,
    pub group : String,
    pub version : String,
    pub description : String,
    pub uuid : Uuid,
}

impl Project {
    pub fn try_new(name: String, folder: PathBuf) -> Result<Project> {

        let name = name.to_case(Case::Kebab);
        let title = name.from_case(Case::Lower).to_case(Case::Title);
        let group = title.clone();
        let version = format!("0.1.0");
        let description = format!("...");
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

        if !options.force &&  Path::new("nw.toml").exists().await {
            return Err("existing nw.toml found ...aborting (use --force to re-create)".into());
        }

        if Path::new("package.json").exists().await {

            let text = fs::read_to_string("package.json").await?;
            let package_json: PackageJson = serde_json::from_str(&text)?;

            self.name = package_json.name.to_lowercase().replace(" ","-");
            self.title = package_json.name;
            if let Some(version) = package_json.version {
                self.version = version.to_string();
            }
            if let Some(description) = package_json.description {
                self.description = description.to_string();
            }

            log_info!("Project","detected existing 'package.json' manifest");
            log_info!("Project","name: '{}' title: '{}' version: '{}'", self.name, self.title, self.version);
            options.manifest = true;

        } else {

            let name = Question::new(&format!("Project name [default:'{}']:",style(&self.name).yellow())).ask();
            if let Some(Answer::RESPONSE(name)) = name {
                if !name.is_empty() {
                    if name.contains(" ") {
                        println!("{}",style("\nError: project name can not contain spaces\n").red());
                        std::process::exit(1);
                    }

                    let name = name.to_case(Case::Kebab);
                    if name != self.name {
                        // self.title = name.from_case(Case::Kebab).to_case(Case::Camel);
                        self.title = name.replace("-"," ").from_case(Case::Kebab).to_case(Case::Title);
                    }

                    self.name = name;
                }
            }
            let title = Question::new(&format!("Project title [default:'{}']:",style(&self.title).yellow())).ask();
            if let Some(Answer::RESPONSE(title)) = title {
                if !title.is_empty() {
                    self.title = title;
                }
            }
        }

        println!("");
        log_info!("Init","creating '{}'",self.name);
        println!("");

        println!("{:?}", self);

        let tpl = self.tpl()?;
        let files = 
            if options.manifest {
                [
                    ("nw.toml",tpl.transform(NW_TOML)),
                ].to_vec()
            } else {

                let package = PackageJson {
                    name : self.title.clone(),
                    main : "root/index.js".to_string(),
                    version: None,
                    description: None,
                };
                let package_json = serde_json::to_string(&package).unwrap();
    
                [
                    ("root/package.json",tpl.transform(&package_json)),
                    ("root/index.js", tpl.transform(INDEX_JS)),
                    ("root/index.html", tpl.transform(INDEX_HTML)),
                    ("src/lib.rs", tpl.transform(LIB_RS)),
                    ("nw.toml",tpl.transform(NW_TOML)),
                    ("Cargo.toml", tpl.transform(CARGO_TOML)),
                    ("build", tpl.transform(BUILD_SH)),
                    ("build.ps1", tpl.transform(BUILD_PS1)),
                ].to_vec()
            };

        let folders: HashSet<&Path> = files
            .iter()
            .map(|(file,_)|Path::new(file).parent())
            .flatten()
            .collect();

        for folder in folders {
            fs::create_dir_all(folder).await?;
        }

        for (filename, content) in files.iter() {
            fs::write(filename,&content).await?;
        }

        cfg_if! {
            if #[cfg(not(target_os = "windows"))] {
                fs::set_permissions(Path::new("build"), std::os::unix::fs::PermissionsExt::from_mode(0o755)).await?;
            }
        }

        println!("Please run 'build' script to build the project");
        println!("Following this, you can run 'nw .' to start run the application");
        println!("");

        Ok(())
    }

    fn tpl(&self) -> Result<Tpl> {

        let tpl : Tpl = [
            ("$NAME",self.name.clone()),
            ("$SNAKE",self.name.from_case(Case::Kebab).to_case(Case::Snake)),
            ("$TITLE",self.title.clone()),
            ("$UUID",self.uuid.to_string()),
            ("$VERSION",self.version.to_string()),
            ("$DESCRIPTION",self.description.to_string()),
        ].as_slice().try_into()?;

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

