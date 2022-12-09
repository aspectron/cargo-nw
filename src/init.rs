use crate::prelude::*;
use async_std::path::PathBuf;
use serde::Serialize;
use async_std::fs;
use convert_case::{Case, Casing};

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
    name : String,
    title : String,
    folder : PathBuf,
}

impl Project {
    pub fn try_new(name: String, folder: PathBuf) -> Result<Project> {

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

    pub fn generate(&self) -> Result<()> {

        println!("{:?}", self);

        println!("TODO - init template project...");


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
