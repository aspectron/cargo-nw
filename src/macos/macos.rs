use cfg_if::cfg_if;
use async_std::path::Path;
use async_std::path::PathBuf;
// use async_std::path::Path;
// use async_std::path::PathBuf;
use async_std::fs;
use async_std::task::sleep;
use fs_extra::dir;
use image::imageops::FilterType;
use serde::Serialize;
use image::GenericImageView;
use duct::cmd;
use regex::Regex;
use chrono::Datelike;
use console::style;
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

    pub ctx : Arc<Context>,
}

#[async_trait]
impl Installer for MacOS {
    async fn create(&self, installer_type: InstallerType) -> Result<Vec<PathBuf>> {
        // println!("[macos] creating {:?} installer",installer_type);

        self.copy_nwjs_bundle().await?;
        self.copy_app_data().await?;
        self.rename_app_bundle(&self.app_contents_folder).await?;
        self.generate_resource_strings(&self.app_contents_folder).await?;
        // self._create_package_json(ctx).await?;
        self.generate_icons().await?;

        // self.rename_plist(ctx).await?;

        log!("MacOS","creating {:?} installer",installer_type);
        // Ok(())


        match installer_type {
            InstallerType::Archive => {
                Ok(vec![])
            },
            InstallerType::DMG => {
                let dmg_file = self.create_dmg().await?;
                Ok(vec![dmg_file.into()])
            },
            _ => {
                Err(format!("Unsupported installer type: {:?}", installer_type).into())
            }
        }
    }
}

impl MacOS {

    pub fn new(ctx: Arc<Context>) -> MacOS {
        // let app_root_folder : PathBuf = Path::new(&ctx.cargo_target_folder).join(&ctx.manifest.package.title).join("nw.app");

        let nwjs_root_folder = ctx.build_folder.join(format!("{}.app", &ctx.manifest.application.title));
        MacOS {
            // app_root_folder : 
            app_contents_folder: nwjs_root_folder.join("Contents"),
            app_resources_folder: nwjs_root_folder.join("Contents").join("Resources"),
            app_nw_folder: nwjs_root_folder.join("Contents").join("Resources").join("app.nw"),
            nwjs_root_folder,
            ctx : ctx.clone(),
            // ctx : ctx.clone(),
            // app_root_folder: PathBuf::from("/Applications"),
        }
    }

    async fn copy_nwjs_bundle(&self) -> Result<()>{
        let mut options = dir::CopyOptions::new();
        options.content_only = true;
        options.skip_exist = true;
        
        log!("Integrating","NWJS binaries");
        dir::copy(
            Path::new(&self.ctx.deps.nwjs.target).join("nwjs.app"), 
            &self.nwjs_root_folder, 
            &options
        )?;

        Ok(())
    }

    async fn copy_app_data(&self) -> Result<()> {

        let mut options = dir::CopyOptions::new();
        options.content_only = true;
        options.overwrite = true;
        // options.skip_exist = true;
        
        log!("Integrating","application data");
        dir::copy(
            &self.ctx.app_root_folder, 
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

    

    async fn generate_icons(&self) -> Result<()> {

        log!("MacOS","generating application icons");
        
        let app_icon = self.ctx.setup_resources_folder.join("app.png");
        // generate_icns_sips(ctx,&app_icon, &self.app_resources_folder.join("app.icns")).await?;
        self.generate_icns_internal(&app_icon, &self.app_resources_folder.join("app.icns")).await?;
        log!("MacOS","generating document icons");
        let document_icon = self.ctx.setup_resources_folder.join("document.png");
        // generate_icns_sips(ctx,&document_icon, &self.app_resources_folder.join("document.icns")).await?;
        self.generate_icns_internal(&document_icon, &self.app_resources_folder.join("document.icns")).await?;

        // let dmg_background = ctx.app_resources_folder.join("dmg.png");



        Ok(())
    }


}


impl MacOS {

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

    async fn generate_icns_internal(&self, png: &PathBuf, icns: &PathBuf) -> Result<()> {

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

        let iconset_folder = self.ctx.cargo_target_folder.join("icns.iconset");
        if !std::path::Path::new(&iconset_folder).exists() {
            std::fs::create_dir_all(&iconset_folder)?;
        }

        cfg_if! {
            if #[cfg(debug_assertions)] {
                let resize_filter_type = FilterType::Triangle;
            } else {
                let resize_filter_type = FilterType::Lanczos3;
            }
        }

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
            .dir(&self.ctx.cargo_target_folder)
            .run()?;

        std::fs::remove_dir_all(iconset_folder)?;

        Ok(())
    }


    async fn plist_bundle_rename(&self, plist_file: &PathBuf, name: &str, version : Option<&str>) -> Result<()> {

        let mut text = fs::read_to_string(plist_file).await?;

        let regex = Regex::new(r"<key>CFBundleDisplayName</key>([^<]*)<string>([^<]*)</string>").unwrap();
        let replace = format!("<key>CFBundleDisplayName</key>$1<string>{name}</string>");
        text = regex.replace(&text,replace).to_string();
        
        if let Some(version) = version {
            let regex = Regex::new(r"<key>CFBundleShortVersionString</key>([^<]*)<string>([^<]*)</string>").unwrap();
            let replace = format!("<key>CFBundleShortVersionString</key>$1<string>{version}</string>");
            text = regex.replace(&text,replace).to_string();
        }

        fs::write(plist_file, text).await?;

        Ok(())
    }

    async fn generate_resource_strings(&self, app_contents_folder: &PathBuf) -> Result<()> {


        let app_title = &self.ctx.manifest.application.title;
        // let authors = if let Some(authors) = &manifest.application.authors { authors.as_str() } else { "" };
        let version = &self.ctx.manifest.application.version;
        let year = format!("{}", chrono::Utc::now().year());

        let copyright = 
        if let Some(copyright) = &self.ctx.manifest.application.copyright {
            copyright.to_string()
        } else if let Some(authors) = &self.ctx.manifest.application.authors {
            format!("Copyright {year} {authors}")
        } else {
            format!("Copyright {year} {app_title} developers")
        };
        
        let _resource_text = format!("\
    CFBundleDisplayName = \"{app_title}\";\n\
    CFBundleGetInfoString = \"{app_title} {version}, {copyright}, The Chromium Authors, NW.js contributors, Node.js. All rights reserved.\";\n\
    CFBundleName = \"{app_title}\";\n\
    NSHumanReadableCopyright = \"{copyright}, The Chromium Authors, NW.js contributors, Node.js. All rights reserved.\";\n\
    ");
    // CFBundleGetInfoString = \"nwjs 107.0.5304.88, Copyright 2022 The Chromium Authors, NW.js contributors, Node.js. All rights reserved.\";\n\

    let resource_text = format!("\
    CFBundleGetInfoString = \"{app_title} {version} {copyright}, Copyright 2022 The Chromium Authors, NW.js contributors, Node.js. All rights reserved.\";\n\
    NSBluetoothAlwaysUsageDescription = \"Once Chromium has access, websites will be able to ask you for access.\";\n\
    NSBluetoothPeripheralUsageDescription = \"Once Chromium has access, websites will be able to ask you for access.\";\n\
    NSCameraUsageDescription = \"Once Chromium has access, websites will be able to ask you for access.\";\n\
    NSHumanReadableCopyright = \"{copyright}, Copyright 2022 The Chromium Authors, NW.js contributors, Node.js. All rights reserved.\";\n\
    NSLocationUsageDescription = \"Once Chromium has access, websites will be able to ask you for access.\";\n\
    NSMicrophoneUsageDescription = \"Once Chromium has access, websites will be able to ask you for access.\";\n\
    ");


    // FIXME setup the contact usage string...
    // NSContactsUsageDescription = \"Details from your contacts can help you fill out forms more quickly in ${app_title}.\";\n\

        let resources_folder = app_contents_folder.join("Resources");
        let paths = std::fs::read_dir(&resources_folder).expect(&format!("unable to iterate {:?}", &resources_folder));
        for file in paths {
            if let Ok(entry) = file {
                if entry.file_name().into_string().unwrap().ends_with(".lproj") {
                    fs::write(entry.path().join("InfoPlist.strings"), &resource_text).await?;
                }
            }
        }

        Ok(())
    }

    async fn rename_app_bundle(&self, app_contents_folder: &PathBuf) -> Result<()> {

        // let app_name = app_name.to_lowercase();
        log!("MacOS","renaming application bundle");

        let plist_file = app_contents_folder.join("info.plist");
        // println!("plist: {:?}", plist_file);
        self.plist_bundle_rename(
            &plist_file, 
            &self.ctx.manifest.application.title,
            Some(&self.ctx.manifest.application.version)
        ).await?;

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
            plist_bundle_rename(&plist_file, &app_name, None).await?;

            let helper_path = helpers_root.join(&src_app).join("Contents").join("MacOS");

            // println!("\nrenaming: {:?} to {:?}", helper_path.join(&src),helper_path.join(&dest));
            fs::rename(helper_path.join(&src),helper_path.join(&dest)).await?;
            // println!("\nrenaming: {:?} to {:?}", helpers_root.join(&src_app),helpers_root.join(&dest_app));
            fs::rename(helpers_root.join(&src_app),helpers_root.join(&dest_app)).await?;
        }
    */    
        Ok(())
    }


    async fn osa_script(&self) -> Result<()> {

        // let caption_bar_height = 48;
        let caption_bar_height = 59;
        let window_width = 485;
        let window_height = 330+caption_bar_height;//+400;
        let window_l = 200;
        let window_t = 200;
        let window_r = window_l + window_width;
        let window_b = window_t + window_height;
        let icon_size = 72;
        let icon_t = 158; // Math.round(150 - iconSize / 2);
        let icon_l = 100;

        let apps_icon_t = icon_t;
        let apps_icon_l = window_width - 100;

        let app_name = &self.ctx.manifest.application.name;
        let app_title = &self.ctx.manifest.application.title;

        // set the bounds of container window to {{{window_t}, {window_l}, {window_r}, {window_b}}}\n\
        
        let script = 
        format!("\
            tell application \"Finder\"\n\
                tell disk \"{app_title}\"\n\
                    open\n\
                    delay 1\n\
                    set current view of container window to icon view\n\
                    set toolbar visible of container window to false\n\
                    set statusbar visible of container window to false\n\
                    #set the size of container window to {{{window_width}, {window_height}}}\n\
                    #set the bounds of container window to {{100, 100, 500, 500}}\n\
                    #set the bounds of container window to {{{window_l}, {window_t}, {window_width}, {window_height}}}\n\
                    set the bounds of container window to {{{window_l}, {window_t}, {window_r}, {window_b}}}\n\
                    #set the bounds of container window to {{{window_t}, {window_l}, {window_r}, {window_b}}}\n\
                    #set position of container window to {{{window_t}, {window_l}}}\n\
                    #delay 1\n\
                    #delay 1\n\
                    #set position of container window to {{{window_t}, {window_l}}}\n\
                    #set size of container window to {{{window_width}, {window_height}}}\n\
                    set theViewOptions to the icon view options of container window\n\
                    set arrangement of theViewOptions to not arranged\n\
                    set icon size of theViewOptions to {icon_size}\n\
                    set background picture of theViewOptions to file \".background:{app_name}.png\"\n\
                    set position of item \"{app_title}.app\" of container window to {{{icon_l}, {icon_t}}}\n\
                    set position of item \"Applications\" of container window to {{{apps_icon_l}, {apps_icon_t}}}\n\
                    update without registering applications\n\
                    #delay 5\n\
                    delay 10\n\
                    close\n\
                end tell\n\
            end tell\n\
        ");

        // make new alias file at container window to POSIX file "/Applications" with properties {name:"Applications"}

        // this.log("Applying AppleScript configuration...".green);

        let osa_script_file = self.ctx.build_folder.join("osa");
        fs::write(&osa_script_file, script).await?;

        // fs.writeFileSync(path.join(this.DEPS,'osa'), script);

        //   this.spawn('osascript',"osa".split(" "), { cwd : this.DEPS, stdio : 'inherit' })

        cmd!(
            "osascript",
            osa_script_file.to_str().unwrap()
        ).stdout_null().run()?;

        // TODO - cleanup script
        // std::fs::remove_file(osa_script_file)?;
    // panic!("aborting...");

        Ok(())
    }

    async fn copy_dmg_files(&self, mountpoint: &PathBuf) -> Result<()> {
        let background_folder = mountpoint.join(".background");
        std::fs::create_dir_all(&background_folder)?;

        let from = self.ctx.setup_resources_folder.join("background.png");
        let to = background_folder.join(format!("{}.png", self.ctx.manifest.application.name));
        fs::copy(from,to).await?;

        // let applications_symlink = Path::new(mountpoint)
        std::os::unix::fs::symlink("/Applications",mountpoint.join("Applications"))?;

        Ok(())
    }

    async fn configure_dmg_icon(&self, mountpoint: &PathBuf) -> Result<()> {
        let icns = self.app_resources_folder.join("app.icns");
        let volume_icns = mountpoint.join(".VolumeIcon.icns");
        println!("volume_icns: {}", volume_icns.to_str().unwrap());
        std::fs::copy(icns,&volume_icns)?;
        cmd!("setfile","-c","icnC", volume_icns).stdout_null().run()?;
        cmd!("setfile","-a","C", mountpoint).stdout_null().run()?;
        Ok(())
    }

    async fn create_dmg(
        &self,
    ) -> Result<PathBuf> {

        let filename = 
        format!("{}-{}-{}-{}",
            self.ctx.manifest.application.name,
            self.ctx.manifest.application.version,
            self.ctx.platform,
            self.ctx.arch,
        );
        let build_dmg_file = &self.ctx.build_folder.join(format!("{filename}.build.dmg"));
        let output_dmg_file = &self.ctx.output_folder.join(format!("{filename}.dmg"));

        let volume_name = &self.ctx.manifest.application.title;
        let mountpoint = PathBuf::from(format!("/Volumes/{volume_name}"));

        if std::path::Path::new(&mountpoint).exists() {
            log!("DMG","{}",style("detaching existing DMG image").yellow());
            cmd!(
                "hdiutil",
                "detach",&mountpoint
            ).stdout_null().run()?;
        }

        if std::path::Path::new(build_dmg_file).exists() {
            std::fs::remove_file(build_dmg_file)?;
        }

        if std::path::Path::new(output_dmg_file).exists() {
            std::fs::remove_file(output_dmg_file)?;
        }

        log!("DMG","creating (UDRW HFS+)");
        cmd!(
            "hdiutil",
            "create",
            "-volname", volume_name,
            "-srcfolder", &self.nwjs_root_folder,
            "-ov",
            "-fs","HFS+",
            "-format","UDRW",
            build_dmg_file
        ).stdout_null().run()?;

        // println!("vvv: {:?}", vvv);

        log!("DMG","attaching");
        cmd!(
            "hdiutil", 
            "attach",
            "-readwrite",
            "-noverify",
            "-noautoopen",
            build_dmg_file,
        ).stdout_null().run()?;

        log!("DMG","configuring");
        self.copy_dmg_files(&mountpoint).await?;
        self.osa_script().await?;
        self.configure_dmg_icon(&mountpoint).await?;
    // cmd!("").read()?;

        log!("DMG","sync");
        cmd!("sync").stdout_null().run()?;
        sleep(std::time::Duration::from_millis(1000)).await;
        cmd!("sync").stdout_null().run()?;

        log!("DMG","detaching");
        cmd!(
            "hdiutil",
            "detach",&mountpoint
        ).stdout_null().run()?;

        log!("DMG","compressing (UDZO)");
        cmd!(
            "hdiutil",
            "convert",
            "-format", "UDZO",
            "-imagekey",
            "zlib-level=9",
            "-o",output_dmg_file, 
            build_dmg_file
        ).stdout_null().run()?;

        let dmg_size = std::fs::metadata(output_dmg_file)?.len() as f64;
        log!("DMG","resulting DMG size: {:.2}Mb", dmg_size/1024.0/1024.0);

        Ok(output_dmg_file.clone())

    }

}