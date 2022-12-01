use async_std::path::Path;
use async_std::path::PathBuf;
use async_std::fs;
use fs_extra::dir;
use serde::Serialize;
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
    pub contents_folder : PathBuf,
    pub resources_folder : PathBuf,
    pub app_nw_folder : PathBuf,

}

impl MacOS {

    pub fn new(ctx: &Context) -> MacOS {
        // let app_root_folder : PathBuf = Path::new(&ctx.cargo_target_folder).join(&ctx.manifest.package.title).join("nw.app");
        
        MacOS {
            contents_folder: Path::new(&ctx.nwjs_root_folder).join("Contents"),
            resources_folder: Path::new(&ctx.nwjs_root_folder).join("Contents").join("Resources"),
            app_nw_folder: Path::new(&ctx.nwjs_root_folder).join("Contents").join("Resources").join("app.nw"),
            // app_root_folder: PathBuf::from("/Applications"),
        }
    }

    async fn copy_app_data(&self, ctx: &Context) -> Result<()> {

        let mut options = dir::CopyOptions::new();
        options.content_only = true;
        // options.skip_exist = true;
        
        println!("[macos] copying application data");
        dir::copy(
            &ctx.app_root_folder, 
            &self.app_nw_folder, 
            &options
        )?;


        Ok(())
    }

    async fn create_package_json(&self, ctx: &Context) -> Result<()> {
/*
{
    "name": "helloworld",
    "main": "index.js"
}
*/
        println!("[macos] creating package.json");

        let package_json = PackageJson {
            name : ctx.manifest.application.title.clone(),
            main : "index.js".to_string(),
        };

        let json = serde_json::to_string(&package_json).unwrap();

        fs::write(&self.app_nw_folder.join("package.json"), json).await?;


        Ok(())
    }
}

#[async_trait]
impl Installer for MacOS {
    async fn create(&self, ctx : &Context, installer_type: InstallerType) -> Result<()> {
        // println!("[macos] creating {:?} installer",installer_type);
        
        self.copy_app_data(ctx).await?;
        self.create_package_json(ctx).await?;
        
        println!("[macos] creating {:?} installer",installer_type);
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

// impl TryInto<MacOS> for Manifest {
//     type Error = Error;
//     fn try_into(self) -> Result<MacOS> {

//         let macos = MacOS { 
//             // dmg_app_name: self.package.title
//             app_root_folder
//         };

//         Ok(macos)
//     }
// }

// this.log("Setting up OSX icns...");
// let resourceFolder = this.options.resources || 'resources/setup'
// await this.copy(path.join(this.appFolder, resourceFolder, this.name+"-icon.icns"),path.join(NWJS_APP_RESOURCES,"app.icns"));
// await this.copy(path.join(this.appFolder, resourceFolder, this.name+"-icon.icns"),path.join(NWJS_APP_RESOURCES,"document.icns"));

// if(this.options.DMG){
//     this.log("Setting up background...");
//     fse.ensureDirSync(path.join(this.APP_DIR, '.background'));
//     await this.copy(path.join(this.appFolder, resourceFolder , this.name+"-dmg.png"),path.join(this.APP_DIR,'.background/'+this.name+'-dmg.png'));

//     this.log("Symlink for /Applications...");
//     fse.ensureSymlinkSync("/Applications", path.join(this.APP_DIR, "/Applications"), 'dir');
// }


// app_nwjs_renaming(){
//     return new Promise(async (resolve, reject) => {
//         let appname = this.DMG_APP_NAME.toLowerCase();
//         let plistFilePath = path.join(this.NWJS_APP_CONTENTS, 'info.plist');
//         let infoPlistContents = fse.readFileSync(plistFilePath)+"";
//         if(infoPlistContents){
//             infoPlistContents = infoPlistContents.replace(
//                 /<key>CFBundleDisplayName<\/key>([^<]*)<string>nwjs<\/string>/,
//                 `<key>CFBundleDisplayName</key>$1<string>${appname}</string>`
//             )
//             fse.writeFileSync(plistFilePath, infoPlistContents);
//         }

//         let helpersPath = path.join(this.NWJS_APP_CONTENTS, "Frameworks", "nwjs Framework.framework", "Versions", "Current", "Helpers")
//         let helpers = ["Helper", "Helper (GPU)", "Helper (Plugin)", "Helper (Renderer)"];

//         while(helpers.length) {
//             let helper = helpers.shift();
//             let src = `nwjs ${helper}`;
//             let dest = `${appname} ${helper}`;
//             let srcApp = `${src}.app`;
//             let destApp = `${dest}.app`;
//             let plistFilePath = path.join(helpersPath, srcApp, 'Contents', 'info.plist');
//             let infoPlistContents = fse.readFileSync(plistFilePath)+"";
//             if(infoPlistContents){
//                 infoPlistContents = infoPlistContents.replace(
//                     /<key>CFBundleDisplayName<\/key>([^<]*)<string>([^<]*)<\/string>/,
//                     `<key>CFBundleDisplayName</key>$1<string>${dest}</string>`
//                 )
//                 fse.writeFileSync(plistFilePath, infoPlistContents);
//             }
//             let helperPath = path.join(helpersPath, srcApp, 'Contents', 'MacOS');
//             await this.spawn('mv', [src, dest], { cwd : helperPath, stdio: 'inherit' });
//             await this.spawn('mv', [srcApp, destApp], { cwd : helpersPath, stdio: 'inherit' });
//         }

//         resolve();
//     });
// }



