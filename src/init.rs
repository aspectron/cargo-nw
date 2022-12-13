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
      //console.log("nw", nw);

      (async()=>{
        window.$$SNAKE = await import('/root/wasm/$NAME.js');
        // window.$$SNAKE = $$NAME;
        const wasm = await window.$$SNAKE.default('/root/wasm/$NAME_bg.wasm');
        window.$$SNAKE.initialize_app();
        console.log("create_context_menu", window.$$SNAKE.create_context_menu())
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
use workflow_log::log_trace;
use workflow_wasm::listener::Listener;
use workflow_dom::utils::window;
use nw_sys::result::Result;
use nw_sys::prelude::*;
use nw_sys::utils;

static mut APP:Option<Arc<ExampleApp>> = None;


#[derive(Clone)]
pub struct ExampleApp{
    pub win_listeners:Arc<Mutex<Vec<Listener<nw::Window>>>>,
    pub menu_listeners:Arc<Mutex<Vec<Listener<JsValue>>>>,
    pub listeners:Arc<Mutex<Vec<Listener<web_sys::MouseEvent>>>>
}


impl ExampleApp{
    fn new()->Arc<Self>{
        let app = Arc::new(Self{
            win_listeners:Arc::new(Mutex::new(vec![])),
            menu_listeners:Arc::new(Mutex::new(vec![])),
            listeners:Arc::new(Mutex::new(vec![]))
        });

        unsafe{
            APP = Some(app.clone());
        };

        app
    }

    fn create_window(&self)->Result<()>{
        let options = nw::window::Options::new()
            .title("Test page")
            .width(200)
            .height(200)
            .left(0);

        let listener = Listener::new(|win:nw::Window|->std::result::Result<(), JsValue>{
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
        });

        nw::Window::open_with_options_and_callback("/root/page2.html", &options, listener.into_js());

        log_trace!("nw.Window.open(\"/root/page2.html\", {})", options);

        self.win_listeners.lock()?.push(listener);

        Ok(())
    }

    fn create_menu(&self)->Result<()>{
        let submenus = nw::Menu::new();
        let this = self.clone();
        let listener = Listener::new(move |_|->std::result::Result<(), JsValue>{
            log_trace!("Create window : menu clicked");
            this.create_window()?;
            Ok(())
        });

        
        let submenus_clone = submenus.clone();
        let listener3 = Listener::new(move |_|->std::result::Result<(), JsValue>{
            log_trace!("Menu 5 clicked");
            Ok(())
        });
        let listener3_clone = listener3.clone();
        let listener2 = Listener::new(move |_|->std::result::Result<(), JsValue>{
            let menu_item = &submenus_clone.items()[2];
            let menu_item_4 = &submenus_clone.items()[3];
            let menu_item_5 = &submenus_clone.items()[4];
            log_trace!("Menu 3 is checked: {:?}", menu_item.checked());
            log_trace!("Menu 3 key: {:?}", menu_item.key());
            menu_item.set_key("0");
            log_trace!("Menu 3 key after set_key(0): {:?}", menu_item.key());
            log_trace!("Menu 4 key: {:?}", menu_item_4.key());
            log_trace!("Menu 4 is enabled: {:?}", menu_item_4.enabled());
            menu_item_4.set_enabled(false);
            log_trace!("Menu 4 is enabled after set_enabled(false): {:?}", menu_item_4.enabled());
            menu_item_5.set_click(listener3_clone.into_js());

            log_trace!("Menu 5 submenu: {:?}", menu_item_5.submenu());
            let menu_options = nw::menu_item::Options::new()
                .label("Sub Menu 1");
            let sub_menu_item_1 = nw::MenuItem::new(&menu_options);
            let submenu = nw::Menu::new();
            submenu.append(&sub_menu_item_1);
            menu_item_5.set_submenu(&submenu);
            log_trace!("Menu 5 submenu: {:?}", menu_item_5.submenu());
            Ok(())
        });

        let menu_options = nw::menu_item::Options::new()
            .label("Create window")
            .key("8")
            .modifiers("ctrl")
            .click(listener.into_js());
        let menu_item_1 = nw::MenuItem::new(&menu_options);

        let menu_item_2 = nw::MenuItem::new(&nw::menu_item::Type::Separator.into());
        
        let menu_options:nw::menu_item::Options = nw::menu_item::Options::new()
            .set_type(nw::menu_item::Type::Checkbox)    
            .label("Menu 3: Update menus")
            .key("9")
            .modifiers("cmd+shift")
            .click(listener2.into_js());
        let menu_item_3 = nw::MenuItem::new(&menu_options);

        let menu_options = nw::menu_item::Options::new()
            .set_type(nw::menu_item::Type::Checkbox)
            .label("Menu 4")
            .click(listener2.into_js());
        let menu_item_4 = nw::MenuItem::new(&menu_options);

        let menu_options = nw::menu_item::Options::new()
            .label("Menu 5");
        let menu_item_5 = nw::MenuItem::new(&menu_options);


        self.menu_listeners.lock()?.push(listener);
        self.menu_listeners.lock()?.push(listener2);
        self.menu_listeners.lock()?.push(listener3);
        
        submenus.append(&menu_item_1);
        submenus.append(&menu_item_2);
        submenus.append(&menu_item_3);
        submenus.append(&menu_item_4);
        submenus.append(&menu_item_5);

        
        let menu_options = nw::menu_item::Options::new()
            .label("Top Menu")
            .submenu(&submenus);

        let menubar = nw::Menu::new_with_options(&nw::menu::Type::Menubar.into());
        let mac_options = nw::menu::MacOptions::new()
            .hide_edit(true)
            .hide_window(true);
        menubar.create_mac_builtin_with_options("$TITLE", &mac_options);
        menubar.append(&nw::MenuItem::new(&menu_options));
        
        let window = nw::Window::get();
        window.set_menu(&menubar);
        //window.remove_menu();

        Ok(())
    }

    pub fn create_context_menu(self:Arc<Self>)->Result<()>{
        let win = nw::Window::get();
        let dom_win = win.window();
        //log_trace!("dom_win: {}, {:?}", win.title(), dom_win);

        let body = utils::body(Some(dom_win));
        //log_trace!("body.inner_html: {:?}", body.inner_html());
        let this = self.clone();
        let listener = Listener::new(move |ev:web_sys::MouseEvent|->std::result::Result<(), JsValue>{
            ev.prevent_default();
            //let x = win.x() + ev.x();
            //let y = win.y() + ev.y();
            log_trace!("win :::: x:{}, y:{}", win.x(), win.y());
            log_trace!("contextmenu :::: x:{}, y:{}", ev.x(), ev.y());

            let menu_listener = Listener::new(move |_|->std::result::Result<(), JsValue>{
                log_trace!("Context menu clicked");
                window().alert_with_message("Context menu clicked")?;
                Ok(())
            });
            
            let menu_options = nw::menu_item::Options::new()
                .label("Sub Menu 1")
                .click(menu_listener.into_js());
            let sub_menu_item_1 = nw::MenuItem::new(&menu_options);

            let menu_options = nw::menu_item::Options::new()
                .label("Sub Menu 2")
                .click(menu_listener.into_js());

            let sub_menu_item_2 = nw::MenuItem::new(&menu_options);
            let popup_menu = nw::Menu::new();
            popup_menu.append(&sub_menu_item_1);
            popup_menu.append(&sub_menu_item_2);
            popup_menu.popup(ev.x(), ev.y());

            this.menu_listeners.lock().unwrap().push(menu_listener);

            
            Ok(())
        });

        body.add_event_listener_with_callback("contextmenu", listener.into_js())?;
        self.listeners.lock()?.push(listener);
        

        Ok(())
    }
}

fn app()->Arc<ExampleApp>{
    unsafe{APP.clone().unwrap()}
}

#[wasm_bindgen]
pub fn create_context_menu()->Result<()>{
    app().create_context_menu()?;
    Ok(())
}

#[wasm_bindgen]
pub fn initialize_app()->Result<()>{
    let nw = nw::try_nw().expect("NW Object not found");
    log_trace!("nw: {:?}", nw);

    let _app = ExampleApp::new();
    Ok(())
}

#[wasm_bindgen]
pub fn initialize()->Result<()>{
    let nw = nw::try_nw().expect("NW Object not found");
    log_trace!("nw: {:?}", nw);

    let app = ExampleApp::new();

    let listener = Listener::new(|_win:nw::Window|->std::result::Result<(), JsValue>{
        //app.create_context_menu()?;
        Ok(())
    });
    let options = nw::window::Options::new()
        .new_instance(false);
    nw::Window::open_with_options_and_callback("/root/index.html", &options, listener.into_js());
    log_trace!("nw.Window.open(\"/root/index.html\")");

    app.win_listeners.lock()?.push(listener);

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

