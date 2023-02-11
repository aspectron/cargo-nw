use nw_sys::prelude::*;
use nw_sys::result::Result;
use wasm_bindgen::prelude::*;
use workflow_dom::utils::window;
use workflow_log::{log_info, log_trace};
use workflow_nw::prelude::*;

static mut APP: Option<Arc<ExampleApp>> = None;

#[derive(Clone)]
pub struct ExampleApp {
    pub inner: Arc<Application>,
}

impl ExampleApp {
    fn new() -> Result<Arc<Self>> {
        let app = Arc::new(Self {
            inner: Application::new()?,
        });

        unsafe {
            APP = Some(app.clone());
        };

        Ok(app)
    }

    fn create_window(&self) -> Result<()> {
        let options = window::Options::new()
            .title("Test page")
            .width(200)
            .height(200)
            .left(0);

        self.inner.create_window_with_callback(
            "/root/page2.html",
            &options,
            |win: Window| -> std::result::Result<(), JsValue> {
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
            },
        )?;

        Ok(())
    }

    fn create_menu(&self) -> Result<()> {
        let this = self.clone();
        let submenu_1 = MenuItemBuilder::new()
            .label("Create window")
            .key("8")
            .modifiers("ctrl")
            .callback(move |_| -> std::result::Result<(), JsValue> {
                log_trace!("Create window : menu clicked");
                this.create_window()?;
                Ok(())
            })
            .build()?;

        let submenu_2 = MenuItemBuilder::new()
            .label("Say hello")
            .key("9")
            .modifiers("ctrl")
            .callback(move |_| -> std::result::Result<(), JsValue> {
                window().alert_with_message("Hello")?;
                Ok(())
            })
            .build()?;

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

    pub fn create_tray_icon(&self) -> Result<()> {
        let _tray = TrayIconBuilder::new()
            .icon("resources/icons/tray-icon@2x.png")
            .icons_are_templates(false)
            .callback(|_| {
                window().alert_with_message("Tray Icon click")?;
                Ok(())
            })
            .build()?;
        Ok(())
    }

    pub fn create_tray_icon_with_menu(&self) -> Result<()> {
        let submenu_1 = MenuItemBuilder::new()
            .label("Say hi")
            .key("6")
            .modifiers("ctrl")
            .callback(move |_| -> std::result::Result<(), JsValue> {
                window().alert_with_message("hi")?;
                Ok(())
            })
            .build()?;

        let exit_menu = MenuItemBuilder::new()
            .label("Exit")
            .callback(move |_| -> std::result::Result<(), JsValue> {
                window().alert_with_message("TODO: Exit")?;
                Ok(())
            })
            .build()?;

        let _tray = TrayIconBuilder::new()
            .icon("resources/icons/tray-icon@2x.png")
            .icons_are_templates(false)
            .submenus(vec![submenu_1, menu_separator(), exit_menu])
            .build()?;

        Ok(())
    }

    pub fn create_context_menu(self: Arc<Self>) -> Result<()> {
        let item_1 = MenuItemBuilder::new()
            .label("Sub Menu 1")
            .callback(move |_| -> std::result::Result<(), JsValue> {
                window().alert_with_message("Context menu 1 clicked")?;
                Ok(())
            })
            .build()?;

        let item_2 = MenuItemBuilder::new()
            .label("Sub Menu 2")
            .callback(move |_| -> std::result::Result<(), JsValue> {
                window().alert_with_message("Context menu 2 clicked")?;
                Ok(())
            })
            .build()?;

        self.inner.create_context_menu(vec![item_1, item_2])?;

        Ok(())
    }
}

fn app() -> Option<Arc<ExampleApp>> {
    unsafe { APP.clone() }
}

#[wasm_bindgen]
pub fn create_context_menu() -> Result<()> {
    if let Some(app) = app() {
        app.create_context_menu()?;
    } else {
        let is_nw = initialize_app()?;
        if !is_nw {
            log_info!("TODO: initialize web-app");
            return Ok(());
        }
        let app = app().expect("Unable to create app");
        app.create_context_menu()?;
    }
    Ok(())
}

#[wasm_bindgen]
pub fn initialize_app() -> Result<bool> {
    let is_nw = is_nw();

    let _app = ExampleApp::new()?;
    Ok(is_nw)
}

#[wasm_bindgen]
pub fn initialize() -> Result<()> {
    let is_nw = initialize_app()?;
    if !is_nw {
        log_info!("TODO: initialize web-app");
        return Ok(());
    }

    let app = app().expect("Unable to create app");

    app.inner.create_window_with_callback(
        "/root/index.html",
        &window::Options::new().new_instance(false),
        |_win: Window| -> std::result::Result<(), JsValue> {
            //app.create_context_menu()?;
            Ok(())
        },
    )?;

    let window = Window::get();
    log_trace!("nw.Window.get(): {:?}", window);

    app.create_menu()?;
    app.create_tray_icon()?;
    app.create_tray_icon_with_menu()?;

    Ok(())
}
