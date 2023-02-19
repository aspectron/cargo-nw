use nw_sys::prelude::*;
use wasm_bindgen::prelude::*;
use workflow_log::{log_info, log_trace};
use workflow_nw::prelude::*;
use workflow_nw::result::Result;

/// Global application object created on application initialization.
static mut APP: Option<Arc<App>> = None;

/// Application struct wrapping `workflow_nw::Application` as an inner.
#[derive(Clone)]
pub struct App {
    pub inner: Arc<Application>,
}

impl App {

    /// Get access to the global application object
    pub fn global() -> Option<Arc<App>> {
        unsafe { APP.clone() }
    }

    /// Create a new application instance
    pub fn new() -> Result<Arc<Self>> {
        let app = Arc::new(Self {
            inner: Application::new()?,
        });

        unsafe {
            APP = Some(app.clone());
        };

        Ok(app)
    }

    /// Create a test page window
    fn create_window(&self) -> Result<()> {
        let options = window::Options::new()
            .title("Test page")
            .width(200)
            .height(200)
            .left(0);

        self.inner.create_window_with_callback(
            "/app/secondary.html",
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

    /// Create application menu
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

    /// Create application tray icon
    pub fn create_tray_icon(&self) -> Result<()> {
        let _tray = TrayMenuBuilder::new()
            .icon("resources/icons/tray-icon@2x.png")
            .icons_are_templates(false)
            .callback(|_| {
                window().alert_with_message("Tray Icon click")?;
                Ok(())
            })
            .build()?;
        Ok(())
    }

    /// Create application tray icon and tray menu
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

        let _tray = TrayMenuBuilder::new()
            .icon("resources/icons/tray-icon@2x.png")
            .icons_are_templates(false)
            .submenus(vec![submenu_1, menu_separator(), exit_menu])
            .build()?;

        Ok(())
    }

    /// Create a custom application context menu
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

/// Creates the application context menu
#[wasm_bindgen]
pub fn create_context_menu() -> Result<()> {
    if let Some(app) = App::global() {
        app.create_context_menu()?;
    } else {
        let is_nw = initialize_app()?;
        if !is_nw {
            log_info!("TODO: initialize web-app");
            return Ok(());
        }
        let app = App::global().expect("Unable to create app");
        app.create_context_menu()?;
    }
    Ok(())
}

/// Crteates the application instance
#[wasm_bindgen]
pub fn initialize_app() -> Result<bool> {
    let is_nw = is_nw();

    let _app = App::new()?;
    Ok(is_nw)
}

/// This function is called from the main `/index.js` file 
/// and creates the main application window containing
/// `index.html`
#[wasm_bindgen]
pub fn initialize() -> Result<()> {
    let is_nw = initialize_app()?;
    if !is_nw {
        log_info!("TODO: initialize web-app");
        return Ok(());
    }

    let app = App::global().expect("Unable to create app");

    app.inner.create_window_with_callback(
        "/app/index.html",
        &window::Options::new().new_instance(false),
        |_win: Window| -> std::result::Result<(), JsValue> {
            //app.create_context_menu()?;
            Ok(())
        },
    )?;

    let window = window::get();
    log_trace!("nw.Window.get(): {:?}", window);

    app.create_menu()?;
    app.create_tray_icon()?;
    app.create_tray_icon_with_menu()?;

    Ok(())
}
