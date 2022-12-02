use async_std::path::Path;
use async_std::path::PathBuf;
use async_std::fs;
use fs_extra::dir;
use image::imageops::FilterType;
use serde::Serialize;
use image::GenericImageView;
use duct::cmd;
use regex::Regex;
// use duct::cmd;
use crate::prelude::*;


#[derive(Debug, Clone, Serialize)]
pub struct PackageJson {
    name : String,
    main : String,
}

pub struct MacOS {

    // pub dmg_app_name : String,
    // pub app_root_folder : PathBuf,
    pub nwjs_root_folder : PathBuf,
    pub app_contents_folder : PathBuf,
    pub app_resources_folder : PathBuf,
    pub app_nw_folder : PathBuf,

}

impl MacOS {

    pub fn new(ctx: &Context) -> MacOS {
        // let app_root_folder : PathBuf = Path::new(&ctx.cargo_target_folder).join(&ctx.manifest.package.title).join("nw.app");

        let nwjs_root_folder = ctx.build_folder.join(format!("{}.app", &ctx.manifest.application.title));
        MacOS {
            // app_root_folder : 
            app_contents_folder: nwjs_root_folder.join("Contents"),
            app_resources_folder: nwjs_root_folder.join("Contents").join("Resources"),
            app_nw_folder: nwjs_root_folder.join("Contents").join("Resources").join("app.nw"),
            nwjs_root_folder,
            // app_root_folder: PathBuf::from("/Applications"),
        }
    }

    async fn copy_nwjs_bundle(&self, ctx: &Context) -> Result<()>{
        let mut options = dir::CopyOptions::new();
        options.content_only = true;
        options.skip_exist = true;
        
        log!("Integrating","NWJS binaries");
        dir::copy(
            Path::new(&ctx.deps.nwjs.target).join("nwjs.app"), 
            &self.nwjs_root_folder, 
            &options
        )?;

        Ok(())
    }

    async fn copy_app_data(&self, ctx: &Context) -> Result<()> {

        let mut options = dir::CopyOptions::new();
        options.content_only = true;
        options.overwrite = true;
        // options.skip_exist = true;
        
        log!("Integrating","application data");
        dir::copy(
            &ctx.app_root_folder, 
            &self.app_nw_folder, 
            &options
        )?;


        Ok(())
    }

    async fn _create_package_json(&self, ctx: &Context) -> Result<()> {
        log!("MacOS","creating package.json");

        let package_json = PackageJson {
            name : ctx.manifest.application.title.clone(),
            main : "index.js".to_string(),
        };

        let json = serde_json::to_string(&package_json).unwrap();
        fs::write(&self.app_nw_folder.join("package.json"), json).await?;
        Ok(())
    }

    

    async fn generate_icons(&self, ctx: &Context) -> Result<()> {

        log!("MacOS","generating application icons");
        
        let app_icon = ctx.setup_resources_folder.join("app.png");
        // generate_icns_sips(ctx,&app_icon, &self.app_resources_folder.join("app.icns")).await?;
        generate_icns_internal(ctx,&app_icon, &self.app_resources_folder.join("app.icns")).await?;
        log!("MacOS","generating document icons");
        let document_icon = ctx.setup_resources_folder.join("document.png");
        // generate_icns_sips(ctx,&document_icon, &self.app_resources_folder.join("document.icns")).await?;
        generate_icns_internal(ctx,&document_icon, &self.app_resources_folder.join("document.icns")).await?;

        // let dmg_background = ctx.app_resources_folder.join("dmg.png");



        Ok(())
    }


}

#[async_trait]
impl Installer for MacOS {
    async fn create(&self, ctx : &Context, installer_type: InstallerType) -> Result<()> {
        // println!("[macos] creating {:?} installer",installer_type);

        self.copy_nwjs_bundle(ctx).await?;
        
        self.copy_app_data(ctx).await?;
        rename_app_bundle(&self.app_contents_folder, &ctx.manifest.application.title).await?;
        // self._create_package_json(ctx).await?;
        self.generate_icons(ctx).await?;

        // self.rename_plist(ctx).await?;

        log!("MacOS","creating {:?} installer",installer_type);
        // Ok(())


        match installer_type {
            InstallerType::Archive => {
                Ok(())
            },
            InstallerType::DMG => {

                Ok(())
            },
            _ => {
                Err(format!("Unsupported installer type: {:?}", installer_type).into())
            }
        }
    }
}

async fn _generate_icns_sips(ctx: &Context, png: &PathBuf, icns: &PathBuf) -> Result<()> {

    let iconset_folder = ctx.cargo_target_folder.join("icns.iconset");
    if !std::path::Path::new(&iconset_folder).exists() {
        std::fs::create_dir_all(&iconset_folder)?;
    }

    let sizes = vec![512,256,128,64,32,16];
    for size in sizes {
        let raw = size*2;
        let name = format!("icon_{}x{}@2.png", size, size);
        // println!("[icns] {}", name);
        cmd!("sips","-z",format!("{raw}"),format!("{raw}"),png,"--out",&iconset_folder.join(name))//.run()?;
        .stdin_null().read()?;

        let name = format!("icon_{}x{}.png", size, size);
        // println!("[icns] {}", name);
        cmd!("sips","-z",format!("{size}"),format!("{size}"),png,"--out",&iconset_folder.join(name))//.run()?;
        .stdin_null().read()?;
    }

    // println!("[icns] creating icns");
    cmd!("iconutil","-c","icns","--output",icns,"icns.iconset")
    .dir(&ctx.cargo_target_folder)
    .run()?;

    std::fs::remove_dir_all(iconset_folder)?;

    Ok(())
}

async fn generate_icns_internal(ctx: &Context, png: &PathBuf, icns: &PathBuf) -> Result<()> {

    let mut src = image::open(png)
        .expect(&format!("Unable to open {:?}", png));

    // The dimensions method returns the images width and height.
    let dimensions = src.dimensions();
    if dimensions.0 != 1024 || dimensions.1 != 1024 {
        println!("");
        println!("WARNING: {}", png.clone().file_name().unwrap().to_str().unwrap());
        println!("         ^^^ icon dimensions are {}x{}; must be 1024x1024", dimensions.0,dimensions.1);
        println!("");
    }
    // println!("dimensions {:?}", src.dimensions());

    // The color method returns the image's `ColorType`.
    // println!("{:?}", img.color());
    // Write the contents of this image to the Writer in PNG format.
    // img.save("test.png").unwrap();

    let iconset_folder = ctx.cargo_target_folder.join("icns.iconset");
    if !std::path::Path::new(&iconset_folder).exists() {
        std::fs::create_dir_all(&iconset_folder)?;
    }

    let resize_filter_type = FilterType::Lanczos3;
    let sizes = vec![512,256,128,64,32,16];
    for size in sizes {
        let dest = src.resize(size*2,size*2,resize_filter_type);
        let name = format!("icon_{}x{}@2.png", size, size);
        // log!("icns","{}", name);
        dest.save(iconset_folder.join(name)).unwrap();
        let dest = src.resize(size,size,resize_filter_type);
        let name = format!("icon_{}x{}.png", size, size);
        // println!("[icns] {}", name);
        dest.save(iconset_folder.join(name)).unwrap();
        src = dest;
    }

    // iconutil -c icns kdx-icon.iconset
    cmd!("iconutil","-c","icns","--output",icns,"icns.iconset")
    .dir(&ctx.cargo_target_folder)
    .run()?;

    std::fs::remove_dir_all(iconset_folder)?;

    Ok(())
}


async fn plist_bundle_rename(plist_file: &PathBuf, bundle_name: &str) -> Result<()> {

    let regex = Regex::new(r"<key>CFBundleDisplayName</key>([^<]*)<string>([^<]*)</string>").unwrap();
    let replace = format!("<key>CFBundleDisplayName</key>$1<string>{bundle_name}</string>");

    let text = fs::read_to_string(plist_file).await?;
    let replaced = regex.replace(&text,replace);
    fs::write(plist_file, replaced.to_string()).await?;

    Ok(())
}

async fn rename_app_bundle(app_contents_folder: &PathBuf, app_name: &str) -> Result<()> {

    // let app_name = app_name.to_lowercase();
    log!("MacOS","renaming application bundle");

    let plist_file = app_contents_folder.join("info.plist");
    // println!("plist: {:?}", plist_file);
    plist_bundle_rename(&plist_file, &app_name).await?;

/* 

    log!("MacOS","renaming framework helpers");

    let frameworks_folder = app_contents_folder
        .join("Frameworks")
        .join("nwjs Framework.framework")
        .join("Versions");
    let framework_version = fs::read_to_string(frameworks_folder.join("Current")).await?;

    let helpers_root = app_contents_folder
        .join("Frameworks")
        .join("nwjs Framework.framework")
        .join("Versions")
        .join(framework_version)
        .join("Helpers");

    let helpers = ["Helper", "Helper (GPU)", "Helper (Plugin)", "Helper (Renderer)", "Helper (Alerts)"];

    for helper in helpers {
        let src = format!("nwjs {helper}");
        let dest = format!("{app_name} {helper}");
        let src_app = format!("{src}.app");
        let dest_app = format!("{dest}.app");
            
        let plist_file = helpers_root.join(&src_app).join("Contents").join("info.plist");
        // println!("plist: {:?}", plist_file);
        plist_bundle_rename(&plist_file, &app_name).await?;

        let helper_path = helpers_root.join(&src_app).join("Contents").join("MacOS");

        // println!("\nrenaming: {:?} to {:?}", helper_path.join(&src),helper_path.join(&dest));
        fs::rename(helper_path.join(&src),helper_path.join(&dest)).await?;
        // println!("\nrenaming: {:?} to {:?}", helpers_root.join(&src_app),helpers_root.join(&dest_app));
        fs::rename(helpers_root.join(&src_app),helpers_root.join(&dest_app)).await?;
    }
*/    
    Ok(())
}


