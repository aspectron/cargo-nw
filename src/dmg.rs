/*
use std::*;
use std::path::{Path, PathBuf};

use indicatif::ProgressBar;
use crate::context::Context;
use crate::error::*;
use crate::task::Task;
use crate::spawn::*;


pub struct DMG {
    pub application_name: String,
    pub application_ident: String,
    pub version: String,
    pub authors: String,
    pub resources : PathBuf,
    pub dmg_integration_file : PathBuf,
    pub dmg_target_file : PathBuf,
    osascript : PathBuf,
    application_folder : PathBuf,
    application_contents : PathBuf,
    application_resources : PathBuf,
    app_nw : PathBuf,
    
    // resources : PathBuf,
    // pub task : Task,
    // pub progress: ProgressBar,
}

*/

/*

impl DMG {

    pub fn new(task : &Task, ctx: &Context) -> DMG {
        // let filestem = ctx.filestem.clone();
        let application_name = ctx.manifest.application_name(); //emanate.name.unwrap().clone();
        let application_ident = ctx.manifest.application_ident(); //emanate.name.unwrap().clone();
        let authors = ctx.manifest.package.authors.join(", ");
        let version = ctx.manifest.package.version;
        let resources = ctx.resources;
        // let task = Task::from(task, "DMG");
        let dmg_integration_file = format!("{}.dmg",application_name);
        let dmg_integration_file = Path::new(&dmg_integration_file);
        let dmg_integration_file = ctx.integration.clone().join(dmg_integration_file);

        let dmg_target_file = format!("{}.dmg", ctx.filestem);
        let dmg_target_file = Path::new(&dmg_target_file);
        let dmg_target_file = ctx.setup.clone().join(dmg_target_file);

        let osascript = ctx.deps.clone().join("osa");
        let application_folder = ctx.integration.clone().join("app");
        let application_contents = application_folder.clone().join(application_name).join("Contents");
        let application_resources = application_contents.clone().join("Resources");;
        let app_nw = application_resources.clone().join("app.nw");;

        DMG {
            application_name,
            application_ident,
            dmg_integration_file,
            dmg_target_file,
            osascript,
            version,
            resources,
            authors,
            application_folder,
            application_contents,
            application_resources,
            app_nw,
            // task
            // progress : task.progress("DMG", 5)
        }
    }

    pub async fn execute(&self) {
        
        self.unmount()?;
        self.attach()?;
        self.configure()?;
        self.resources()?;
        self.sync()?;
        self.unmount()?;
        self.package()?;

        // dmg-unmount',
		// 		'dmg-create',
		// 		'dmg-attach',
		// 		'dmg-configure',
		// 		'dmg-nwjs-plists',
		// 		'dmg-sync',
		// 		'dmg-detach',
		// 		'dmg-package

    }

    pub async fn sync(&self,) -> Result<()> {
        spawn(
            "sync",
            &[],
            ProcessOutput::Pipe).await?;
        Ok(())
    }

    pub async fn unmount(&self,) -> Result<()> {
        spawn(
            "hdiutil",
            &[
                "detach",
                &format!("/Volumes/{}",self.application_name)
            ],
            ProcessOutput::Pipe).await?;
        Ok(())
    }


    pub async fn create(&self,) -> Result<()> {
        spawn(
            "hdiutil",
            &[
                "create",
                "-volname",
                &self.application_name,
                "-srcfolder",
                "./DMG",
                "-ov",
                "-fs",
                "HFS+",
                "-format",
                "UDRW",
                self.dmg_integration_file.to_str().unwrap(),
            ],
            ProcessOutput::Pipe).await?;
        Ok(())
    }

    pub async fn attach(&self,) -> Result<()> {
        spawn(
            "hdiutil",
            &[
                "attach",
                "-readwrite",
                "-noverify",
                "-noautoopen",
                self.dmg_integration_file.to_str().unwrap(),
            ],
            ProcessOutput::Pipe).await?;
        Ok(())
    }

    pub fn init_application(&self) -> Result<()> {


        if self.application_folder.exists() {
            fs::remove_dir_all(self.application_folder.clone()).expect(&format!("Error removing integration folder {}", integration.to_str().unwrap()));
        }
        // <app>/Contents/Resources/
        fs::create_dir_all(self.app_nw.clone()).ok();



        println!("Setting up OSX icns...");
        // let resource_folder = self.resource_folder; //res""; // this.options.resources || 'resources/setup'
        let from = self.resource_folder.clone().join(self.application_ident + "-icon.icns");
        let to = self.application_resources.clone().join("app.icns");
        fs::copy(from,to)?;
        let to = self.application_resources.clone().join("document.icns");
        fs::copy(from,to)?;
        

        // await this.copy(path.join(this.appFolder, resourceFolder, this.name+"-icon.icns"),
        // path.join(NWJS_APP_RESOURCES,"app.icns"));
        // await this.copy(path.join(this.appFolder, resourceFolder, this.name+"-icon.icns"),path.join(NWJS_APP_RESOURCES,"document.icns"));

        if(this.options.DMG){
            this.log("Setting up background...");
            fse.ensureDirSync(path.join(this.APP_DIR, '.background'));
            await this.copy(path.join(this.appFolder, resourceFolder , this.name+"-dmg.png"),path.join(this.APP_DIR,'.background/'+this.name+'-dmg.png'));

            this.log("Symlink for /Applications...");
            fse.ensureSymlinkSync("/Applications", path.join(this.APP_DIR, "/Applications"), 'dir');
      }


        Ok(())
    }

    pub async fn rename(&self) -> Result<()> {
        let appname = this.DMG_APP_NAME.toLowerCase();
        let plistFilePath = path.join(this.NWJS_APP_CONTENTS, 'info.plist');
        let infoPlistContents = fse.readFileSync(plistFilePath)+"";
        if(infoPlistContents){
            infoPlistContents = infoPlistContents.replace(
                /<key>CFBundleDisplayName<\/key>([^<]*)<string>nwjs<\/string>/,
                `<key>CFBundleDisplayName</key>$1<string>${appname}</string>`
            )
            fse.writeFileSync(plistFilePath, infoPlistContents);
        }

        let helpersPath = path.join(this.NWJS_APP_CONTENTS, "Frameworks", "nwjs Framework.framework", "Versions", "Current", "Helpers")
        let helpers = ["Helper", "Helper (GPU)", "Helper (Plugin)", "Helper (Renderer)"];

        while(helpers.length) {
            let helper = helpers.shift();
            let src = `nwjs ${helper}`;
            let dest = `${appname} ${helper}`;
            let srcApp = `${src}.app`;
            let destApp = `${dest}.app`;
            let plistFilePath = path.join(helpersPath, srcApp, 'Contents', 'info.plist');
            let infoPlistContents = fse.readFileSync(plistFilePath)+"";
            if(infoPlistContents){
                infoPlistContents = infoPlistContents.replace(
                    /<key>CFBundleDisplayName<\/key>([^<]*)<string>([^<]*)<\/string>/,
                    `<key>CFBundleDisplayName</key>$1<string>${dest}</string>`
                )
                fse.writeFileSync(plistFilePath, infoPlistContents);
            }
            let helperPath = path.join(helpersPath, srcApp, 'Contents', 'MacOS');
            await this.spawn('mv', [src, dest], { cwd : helperPath, stdio: 'inherit' });
            await this.spawn('mv', [srcApp, destApp], { cwd : helpersPath, stdio: 'inherit' });
        }
    }

    pub async fn configure(&self) -> Result<()> {

            let captionBarHeight = 48;
            let width = 485;
            let height = 330+captionBarHeight;
            let offsetX = 400;
            let offsetY = 100;
            let right = offsetX + width;
            let bottom = offsetY + height;
            let iconSize = 72;
            let iconY = 158; // Math.round(150 - iconSize / 2);
            let iconOffset = 100;
            let width_sub_iconOffset = width - iconOffset;
    
            let application_name = self.application_name.clone();
            let ident = "";

            let script = 
            format!(r#"
                tell application "Finder"
                    tell disk "{application_name}"
                        open
                        set current view of container window to icon view
                        set toolbar visible of container window to false
                        set statusbar visible of container window to false
                        set the bounds of container window to {{ {offsetX}, {offsetY}, {right}, {bottom} }}
                        set theViewOptions to the icon view options of container window
                        set arrangement of theViewOptions to not arranged
                        set icon size of theViewOptions to ${iconSize}
                        set background picture of theViewOptions to file ".background:{ident}-dmg.png"
                        set position of item "{application_name}.app" of container window to {{ {iconOffset}, {iconY} }}
                        set position of item "Applications" of container window to {{ {width_sub_iconOffset}, {iconY} }}
                        update without registering applications
                        delay 5
                        close
                    end tell
                end tell  
            "#);
    
            // // make new alias file at container window to POSIX file "/Applications" with properties {name:"Applications"}
    
            println!("Applying AppleScript configuration...");

            // let osa = Path::new("osa");
            fs::write(&self.osascript, &script)?;//.expect("Unable to write apple script file");

            spawn(
                "osascript",
                &[
                    self.osascript.to_str().unwrap(),
                ],
                ProcessOutput::Pipe
            ).await?;

            let application_name = &self.application_name;
            spawn(
                "chmod",
                &[
                    "-R",
                    "a+rw",
                    &format!("/Volumes/{application_name}/{application_name}.app")
                ],
                ProcessOutput::Pipe
            ).await?;
    
        Ok(())
    }


    pub async fn resources(&self) -> Result<()> {
    
        use chrono::Datelike;
        let current_date = chrono::Utc::now();
        let year = current_date.year();
        let authors = &self.authors;
        // dmg_nwjc_plists(callback) {

        let application_name = &self.application_name;
        let version = self.version.clone();

        let text =
            format!(r#"CFBundleDisplayName = "{application_name}";
CFBundleGetInfoString = "{application_name} ${version}, Copyright {year} {authors}, The Chromium Authors, NW.js contributors, Node.js. All rights reserved.";
CFBundleName = "{application_name}";
NSContactsUsageDescription = "Details from your contacts can help you fill out forms more quickly in {application_name}.";
NSHumanReadableCopyright = "Copyright {year} {authors}, The Chromium Authors, NW.js contributors, Node.js. All rights reserved.";
"#);

        let resource_folder = format!("/Volumes/{application_name}/{application_name}.app/Contents/Resources");
        let resource_folder = Path::new(&resource_folder);
        //   var resources = fs.readdirSync(resourceFolder);
        let resources = fs::read_dir(resource_folder)?;

        for resource in resources {
            let path = resource.unwrap().path();
            let path_str = path.to_str().unwrap();
            if path_str.ends_with(".lproj") {
                let target = resource_folder.clone().join(path).join(Path::new("InfoPlist.strings"));

                if !target.exists() {
                    panic!("Unable to locate: {}", target.to_str().unwrap());
                }

                fs::write(&target, &text)?;//.expect("Unable to write {}", );

                //path.join(resourceFolder,item,'InfoPlist.strings');
//                   
            }
        }

  
//           process.stdout.write("processing InfoPlist strings: ".green);
//           _.each(resources, (item) => {
//               if(item.match(/\.lproj$/ig)) {
//                   process.stdout.write(item.split('.').shift()+' ');
//                   var target = path.join(resourceFolder,item,'InfoPlist.strings');
//                   if(!fs.existsSync(target)) {
//                       this.log("\nUnable to locate:".red.bold,target.toString().magenta.bold);
//                       return;
//                   }
  
//                   fs.writeFileSync(target,text);
//               }
//           })
  
//           process.stdout.write('\n');
//           callback();
    


        Ok(())
    }

    pub async fn package(&self) -> Result<()> {

        if self.dmg_target_file.exists() {
            fs::remove_file(self.dmg_target_file.clone())?;
        }

        spawn(
            "hdiutil",
            &[

                "convert",
                "-format",
                "UDZO",
                "-imagekey",
                "zlib-level=9",
                "-o",
                self.dmg_target_file.to_str().unwrap(),
                self.dmg_integration_file.to_str().unwrap(),

            ],
            ProcessOutput::Pipe).await?;

            
        let hash = crate::hash::sha256_from_file(&self.dmg_target_file)?;
        let hash_file = self.dmg_target_file.clone().join(Path::new(".sha256sum"));
        fs::write(&hash_file, &hash)?;//.expect("Unable to write {}", );

		// let targetDMG = path.join(this.SETUP, this.targetDMG);
		// if(fs.existsSync(targetDMG))
		// 	await this.remove(targetDMG);
		
        // var args = ['convert', '-format', 'UDZO', '-imagekey', 'zlib-level=9','-o',`${targetDMG}`, `${this.DMG_APP_NAME}.dmg`];
		// this.log((`Converting and Compressing ${this.DMG_APP_NAME}.dmg to `).green+this.targetDMG.green.bold+"...".green, args);
		// return new Promise(async (resolve, reject)=>{
		// 	this.spawn('hdiutil', args, {cwd:this.SETUP,stdio:'inherit'}).then( async(code)=>{
		// 		if(code){
		// 			console.log('Error running hdutil - code: '+code)
		// 			return reject(code);
		// 		}


		// 		let hash = await this.utils.fileHash(targetDMG, 'sha1');
		// 		let hashFile = targetDMG+'.sha1sum';
		// 		fs.writeFileSync(hashFile, hash);
	
		// 		if(0)
		// 			this.log("...keeping old dmg for experiments".red.bold);
		// 		else
		// 			fs.unlinkSync(path.join(this.SETUP, this.DMG_APP_NAME+".dmg"));

		// 		resolve();
		// 	}, err=>{
		// 		console.log("Detaching:error", err);
		// 		reject(err);
		// 	})
		// })

        Ok(())
    }

}

// this.spawn('hdiutil',[`detach`,`/Volumes/${this.DMG_APP_NAME}`], {cwd:this.ROOT, stdio : 'inherit' })

*/