use duct::cmd;


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



