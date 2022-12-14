use std::collections::HashSet;

use crate::prelude::*;
use async_std::path::{PathBuf, Path};
use async_std::fs;
use convert_case::{Case, Casing};
use question::{Answer, Question};
use console::style;
use uuid::Uuid;

const DEFAULT_APPLICATION_ICON: &[u8] = include_bytes!("../resources/default-application-icon.png");
const MACOS_DMG_BACKGROUND: &[u8] = include_bytes!("../resources/macos-dmg-background.png");
const INNOSETUP_55X58_IMAGE: &[u8] = include_bytes!("../resources/innosetup-55x58.bmp");
const INNOSETUP_164X314_IMAGE: &[u8] = include_bytes!("../resources/innosetup-164x314.bmp");


const INDEX_JS: &str = r###"
(async()=>{
    window.$$SNAKE = await import('/root/wasm/$NAME.js');
    // window.$$SNAKE = $$NAME;
    const wasm = await window.$$SNAKE.default('/root/wasm/$NAME_bg.wasm');
    //console.log("wasm", wasm, workflow)
    //$$SNAKE.init_console_panic_hook();
    //$$SNAKE.show_panic_hook_logs();
    window.$$SNAKE.initialize();
})();

"###;

const INDEX_HTML: &str = r###"
<!DOCTYPE html>
<html>
  <head>
    <title>Hello World!</title>
    <style>
        html{
            background-color:#FFF;
            height:100%;
            width:100%;
            margin:0px;
            padding:0px;
        }
        body{
            min-height:100px;
            height:100%;
            background-color:#cbcbcb;
            position: absolute;
            left:0px;
            right:0px;
            top:0px;
            bottom:0px;
            padding:15px;
            margin:0px;
        }
    </style>
  </head>
  <body>
    <h1>Hello World!</h1>
    <script>
      (async()=>{
        window.$$SNAKE = await import('/root/wasm/$NAME.js');
        const wasm = await window.$$SNAKE.default('/root/wasm/$NAME_bg.wasm');
        window.$$SNAKE.create_context_menu();
      })();
    </script>
  </body>
</html>
"###;

const PAGE2_HTML: &str = r###"
<!DOCTYPE html>
<html>
  <head>
    <!--title>new window test</title-->
  </head>
  <body>
    <h1>Window 2</h1>
    <script>
        console.log("nw", nw);
    </script>
  </body>
</html>
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
# exclude = ["resources/setup"]
exclude = [{ glob = ["{src/*,target/*,test/*,resources/setup/*,*.lock,*.toml,.git*}"] }]

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

[lib]
crate-type = ["cdylib", "rlib"]

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]

# Make sure to update crate versions before production use
# Use https://crates.io to get the latest versions

wasm-bindgen = "0.2.79"
js-sys = "0.3.56"
nw-sys={path="../nw-sys"}
workflow-log="*"
workflow-wasm = {path="../workflow-wasm"}
workflow-dom ={path="../workflow-dom"}
workflow-nw ={path="../workflow-nw"}

[dependencies.web-sys]
version = "0.3.60"
features = [
    'console',
    'Document',
    'Window',
    'HtmlElement',
    'CustomEvent',
    'MouseEvent'
]

"###;

const LIB_RS: &str = r###"


use wasm_bindgen::prelude::*;
use workflow_log::{log_trace, log_info};
use workflow_dom::utils::window;
use nw_sys::result::Result;
use nw_sys::prelude::*;
use workflow_nw::prelude::*;

static mut APP:Option<Arc<ExampleApp>> = None;

#[derive(Clone)]
pub struct ExampleApp{
    pub inner:Arc<App>
}


impl ExampleApp{
    fn new()->Result<Arc<Self>>{
        let app = Arc::new(Self{
            inner: App::new()?
        });

        unsafe{
            APP = Some(app.clone());
        };

        Ok(app)
    }

    fn create_window(&self)->Result<()>{
        let options = nw::window::Options::new()
            .title("Test page")
            .width(200)
            .height(200)
            .left(0);

        self.inner.create_window_with_callback(
            "/root/page2.html", 
            &options,
            |win:nw::Window|->std::result::Result<(), JsValue>{
                log_trace!("win: {:?}", win);
                log_trace!("win.x: {:?}", win.x());
                win.move_by(300, 0);
                win.set_x(100);
                win.set_y(100);

                log_trace!("win.title: {}", win.title());
                win.set_title("Another Window");
                log_trace!("win.set_title(\"Another Window\")");
                log_trace!("win.title: {}", win.title());

                Ok(())
            }
        )?;

        Ok(())
    }

    fn create_menu(&self)->Result<()>{

        let this = self.clone();
        let submenu_1 = MenuItemBuilder::new()
            .label("Create window")
            .key("8")
            .modifiers("ctrl")
            .callback(move |_|->std::result::Result<(), JsValue>{
                log_trace!("Create window : menu clicked");
                this.create_window()?;
                Ok(())
            }).build()?;
        
        let submenu_2 = MenuItemBuilder::new()
            .label("Say hello")
            .key("9")
            .modifiers("ctrl")
            .callback(move |_|->std::result::Result<(), JsValue>{
                window().alert_with_message("Hello")?;
                Ok(())
            }).build()?;
        
        let item = MenuItemBuilder::new()
            .label("Top Menu")
            .submenus(vec![submenu_1, menu_separator(), submenu_2])
            .build()?;

        
        MenubarBuilder::new("$TITLE")
            .mac_hide_edit(true)
            .mac_hide_window(true)
            .append(item)
            .build(true)?;
        
        Ok(())
    }

    pub fn create_context_menu(self:Arc<Self>)->Result<()>{

        let item_1 = MenuItemBuilder::new()
            .label("Sub Menu 1")
            .callback(move |_|->std::result::Result<(), JsValue>{
                window().alert_with_message("Context menu 1 clicked")?;
                Ok(())
            }).build()?;

        let item_2 = MenuItemBuilder::new()
            .label("Sub Menu 2")
            .callback(move |_|->std::result::Result<(), JsValue>{
                window().alert_with_message("Context menu 2 clicked")?;
                Ok(())
            }).build()?;


        self.inner.create_context_menu(vec![item_1, item_2])?;

        Ok(())
    }
}

fn app()->Option<Arc<ExampleApp>>{
    unsafe{APP.clone()}
}

#[wasm_bindgen]
pub fn create_context_menu()->Result<()>{
    if let Some(app) = app(){
        app.create_context_menu()?;
    }else{
        let is_nw = initialize_app()?;
        if !is_nw{
            log_info!("TODO: initialize web-app");
            return Ok(());
        }
        let app = app().expect("Unable to create app");
        app.create_context_menu()?;
    }
    Ok(())
}

#[wasm_bindgen]
pub fn initialize_app()->Result<bool>{
    let is_nw = nw::is_nw();

    let _app = ExampleApp::new()?;
    Ok(is_nw)
}

#[wasm_bindgen]
pub fn initialize()->Result<()>{
    let is_nw = initialize_app()?;
    if !is_nw{
        log_info!("TODO: initialize web-app");
        return Ok(());
    }

    let app = app().expect("Unable to create app");

    app.inner.create_window_with_callback(
        "/root/index.html",
        &nw::window::Options::new().new_instance(false),
        |_win:nw::Window|->std::result::Result<(), JsValue>{
            //app.create_context_menu()?;
            Ok(())
        }
    )?;

    let window = nw::Window::get();
    log_trace!("nw.Window.get(): {:?}", window);

    app.create_menu()?;
    
    Ok(())
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
                    version: Some(self.version.clone()),
                    description: Some("".to_string()),
                };
                let package_json = serde_json::to_string_pretty(&package).unwrap();
    
                [
                    ("package.json",tpl.transform(&package_json)),
                    ("root/index.js", tpl.transform(INDEX_JS)),
                    ("root/index.html", tpl.transform(INDEX_HTML)),
                    ("root/page2.html", tpl.transform(PAGE2_HTML)),
                    ("src/lib.rs", tpl.transform(LIB_RS)),
                    ("nw.toml",tpl.transform(NW_TOML)),
                    ("Cargo.toml", tpl.transform(CARGO_TOML)),
                    ("build", tpl.transform(BUILD_SH)),
                    ("build.ps1", tpl.transform(BUILD_PS1)),
                ].to_vec()
            };

//             const MACOS_DMG_BACKGROUND: &[u8] = include_bytes!("../resources/macos-dmg-background.png");
// const INNOSETUP_55x58_IMAGE: &[u8] = include_bytes!("../resources/innosetup-55x58.bmp");
// const INNOSETUP_164x314_IMAGE: &[u8] = include_bytes!("../resources/innosetup-164x314.bmp");

        let images = [
            ("resources/setup/application.png",DEFAULT_APPLICATION_ICON),
            ("resources/setup/document.png",DEFAULT_APPLICATION_ICON),
            ("resources/setup/macos-application.png",DEFAULT_APPLICATION_ICON),
            ("resources/setup/macos-dmg-background.png",MACOS_DMG_BACKGROUND),
            ("resources/setup/innosetup-55x58.png",INNOSETUP_55X58_IMAGE),
            ("resources/setup/innosetup-164x314.png",INNOSETUP_164X314_IMAGE),
        ];

        let folders: HashSet<&Path> = files
            .iter()
            .map(|(f,_)|f)
            .chain(images.iter().map(|(f,_)|f))
            .map(|path|Path::new(path).parent())
            .flatten()
            .collect();

        for folder in folders {
            fs::create_dir_all(folder).await?;
        }

        for (filename, content) in files.iter() {
            fs::write(filename,&content).await?;
        }

        for (filename,data) in images.iter() {
            fs::write(filename,data).await?;
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

