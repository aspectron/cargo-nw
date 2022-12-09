use crate::prelude::*;
use async_std::path::PathBuf;
use serde::Serialize;
use async_std::fs;
use convert_case::{Case, Casing};
use question::{Answer, Question};
use console::style;

#[derive(Debug, Clone, Serialize)]
pub struct PackageJson {
    name : String,
    main : String,
}

// const PACKAGE_JSON: &str = r###"
// "###;

const INDEX_JS: &str = r###"
(async()=>{
    let $$NAME = await import('../$NAME/$NAME.js');
    window.$$NAME = $$NAME;
    const wasm = await $$NAME.default('/$NAME/$NAME_bg.wasm');
    //console.log("wasm", wasm, workflow)
    //$$NAME.init_console_panic_hook();
    //$$NAME.show_panic_hook_logs();
    $$NAME.initialize();
})();

"###;

const INDEX_HTML: &str = r###"

(async()=>{
    let $$NAME = await import('../$NAME/$NAME.js');
    const wasm = await $$NAME.default('/$NAME/$NAME_bg.wasm'); 
    $$NAME.run();
    // console.log("create_context_menu", $$NAME.create_context_menu())
})();
"###;

const NW_TOML: &str = r###"

# nw.toml - for additional properties please see https://example.com

[application]
name = "$NAME"
version = "0.1.0"
title = "$TITLE"
description = "$TITLE"
# root = "root"
# resources = "resources/setup"
# organization = "Your Organization Name"

[nwjs]
version = "0.70.1"
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
edition = "$RUST_EDITION"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
workflow-log = "*"
workflow-panic-hook = "*"

"###;

const LIB_RS: &str = r###"

"###;

#[derive(Debug)]
pub struct Project {
    pub name : String,
    pub title : String,
    pub folder : PathBuf,
}

impl Project {
    pub fn try_new(name: String, folder: PathBuf) -> Result<Project> {

        let name = name.to_case(Case::Kebab);
        let title = name.from_case(Case::Lower).to_case(Case::Title);

        let nw_toml = folder.join("nw.toml");
        let package_json = folder.join("package.json");
        let index_js = folder.join("index.js");
        let index_html = folder.join("index.html");

        let project = Project {
            name,
            title,
            folder
        };

        Ok(project)
    }

    pub async fn generate(&mut self) -> Result<()> {

        // println!("TODO - init template project...");

        let name = Question::new(&format!("Project name [default:'{}']:",style(&self.name).yellow())).ask();

        if let Some(Answer::RESPONSE(name)) = name {

            if !name.is_empty() {
                // name.chars().all(char::is_alphanumeric)
                // name.chars().all(char::is_alphanumeric)

                if name.contains(" ") {
                    println!("{}",style("\nError: project name can not contain spaces\n").red());
                    std::process::exit(1);
                }

                // let name = name.to_lowercase().to_case(Case::Kebab);
                let name = name.to_case(Case::Kebab);

                if name != self.name {
                    // self.title = name.from_case(Case::Kebab).to_case(Case::Camel);
                    self.title = name.from_case(Case::Kebab).to_case(Case::Title);
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

        println!("");
        log!("Init","creating project...");
        println!("");
        // println!("name:{:?} title:{:?}",self.name,self.title);

        println!("{:?}", self);

        let package = PackageJson {
            name : self.title.clone(),
            main : "index.js".to_string()
        };
        let package_json = serde_json::to_string(&package).unwrap();

        let files = [
            ("nw.toml",NW_TOML),
            ("package.json",&package_json),
            ("index.js", INDEX_JS),
            ("Cargo.toml", CARGO_TOML),
            ("libr.rs", CARGO_TOML),
            // ("",),
            // ("",),
            // ("",),
            // ("",),
            // ("",),

        ];

        for (filename, content) in files.iter() {
            fs::write(filename,content).await?;
        }

        // let answer = Question::new("Continue?")
        // .default(Answer::YES)
        // .show_defaults()
        // .confirm();

        // ^ TODO - create files

        Ok(())
    }

    async fn create_package_json(&self, ctx: &Context) -> Result<()> {
        log!("MacOS","creating package.json");

        let package_json = PackageJson {
            name : ctx.manifest.application.title.clone(),
            main : "index.js".to_string(),
        };

        let json = serde_json::to_string(&package_json).unwrap();
        fs::write(&self.folder.join("package.json"), json).await?;
        Ok(())
    }


}
