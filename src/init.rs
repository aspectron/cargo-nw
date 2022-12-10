use crate::prelude::*;
use async_std::path::PathBuf;
use serde::Serialize;
use async_std::fs;
use convert_case::{Case, Casing};
use question::{Answer, Question};
use console::style;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct PackageJson {
    name : String,
    main : String,
}

// const PACKAGE_JSON: &str = r###"
// "###;

const INDEX_JS: &str = r###"
(async()=>{
    window.$$SNAKE = await import('../$NAME/$NAME.js');
    // window.$$SNAKE = $$NAME;
    const wasm = await window.$$SNAKE.default('/$NAME/$NAME_bg.wasm');
    //console.log("wasm", wasm, workflow)
    //$$SNAKE.init_console_panic_hook();
    //$$SNAKE.show_panic_hook_logs();
    window.$$SNAKE.initialize();
})();

"###;

const INDEX_HTML: &str = r###"

(async()=>{
    let $$SNAKE = await import('../$NAME/$NAME.js');
    const wasm = await $$NAME.default('/$NAME/$NAME_bg.wasm'); 
    $$SNAKE.run();
    // console.log("create_context_menu", $$SNAKE.create_context_menu())
})();
"###;

const NW_TOML: &str = r###"

# nw.toml - for additional properties please see https://example.com

[application]
name = "$NAME"
version = "0.1.0"
title = "$TITLE"
summary = "..."
description = """
...
""""

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
edition = "2021"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
workflow-log = "*"
workflow-panic-hook = "*"

"###;

const LIB_RS: &str = r###"

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
    pub uuid : Uuid,
}

impl Project {
    pub fn try_new(name: String, folder: PathBuf) -> Result<Project> {

        let name = name.to_case(Case::Kebab);
        let title = name.from_case(Case::Lower).to_case(Case::Title);
        let group = title.clone();
        let version = format!("0.1.0");
        let uuid = Uuid::new_v4();

        let project = Project {
            folder,
            name,
            title,
            group,
            version,
            uuid,
        };

        Ok(project)
    }

    pub async fn generate(&mut self, _options: Options) -> Result<()> {

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
        log!("Init","creating '{}'",self.name);
        println!("");

        println!("{:?}", self);

        let package = PackageJson {
            name : self.title.clone(),
            main : "index.js".to_string()
        };
        let package_json = serde_json::to_string(&package).unwrap();

        let tpl = self.tpl()?;

        let files = [
            ("nw.toml",tpl.transform(NW_TOML)),
            ("package.json",tpl.transform(&package_json)),
            ("index.js", tpl.transform(INDEX_JS)),
            ("Cargo.toml", tpl.transform(CARGO_TOML)),
            ("lib.rs", tpl.transform(CARGO_TOML)),
        ];

        for (filename, content) in files.iter() {
            fs::write(filename,&content).await?;
        }

        Ok(())
    }

    fn tpl(&self) -> Result<Tpl> {

        let tpl : Tpl = [
            ("$NAME",self.name.clone()),
            ("$SNAKE",self.name.from_case(Case::Kebab).to_case(Case::Snake)),
            ("$TITLE",self.title.clone()),
            ("$UUID",self.uuid.to_string()),
            ("$VERSION",self.version.to_string()),
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

