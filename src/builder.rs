use async_std::fs;
use console::style;
use std::time::Instant;
use crate::prelude::*;

pub struct Builder {
    pub ctx : Arc<Context>
}

impl Builder {
    pub fn new(ctx: Arc<Context>) -> Self {
        Builder {
            ctx
        }
    }


    pub async fn execute(&self, targets: TargetSet) -> Result<()> {

        println!("{:#?}", self.ctx.manifest);
        // return Ok(());

        if let Some(builds) = &self.ctx.manifest.package.build {
            let cwd = self.ctx.app_root_folder.to_str().unwrap().to_string();

            for build in builds.iter() {
            match build {
                Build::WASM {
                    clean,
                    purge,
                    folder,
                    args,
                    dev,
                    env,
                } => {

                    // invoke cargo clean
                    if clean.unwrap_or(false) {
                        execute(&self.ctx,vec!["cargo".to_string(),"clean".to_string()],&None,&Some(cwd.clone()),&None,&None).await?;
                    }

                    // delete the entire target folder
                    if purge.unwrap_or(false) && self.ctx.cargo_target_folder.exists().await {
                        fs::remove_dir_all(&self.ctx.cargo_target_folder).await?;
                    }

                    let outdir = folder.clone().unwrap_or("root/wasm".to_string());
                    let name = &self.ctx.manifest.application.name;
                    let mut argv = vec!["wasmpack","build"];
                    if dev.unwrap_or(false) {
                        argv.push("--dev");
                    }
                    argv.extend_from_slice(&["--target","web","--out-name",name.as_str(),"--out-dir",outdir.as_str()]);
                    if let Some(args) = args {
                        argv.extend(args.split(" ").collect::<Vec<_>>());
                    }
                    let argv = argv.iter().map(|s|s.to_string()).collect();

                    execute(&self.ctx,argv,env,&Some(cwd.clone()),&None,&None).await?;
                },
                Build::NPM {
                    clean,
                    args,
                    dev,
                    env,
                } => {

                    let node_modules_folder = self.ctx.app_root_folder.join("node_modules");
                    if clean.unwrap_or(false) && node_modules_folder.exists().await {
                        fs::remove_dir_all(&node_modules_folder).await?;
                    }

                    let mut argv = vec!["npm","install"];
                    if !dev.unwrap_or(false) {
                        argv.extend_from_slice(&["--omit","dev"]);
                    }
                    if let Some(args) = args {
                        argv.extend(args.split(" ").collect::<Vec<_>>());
                    }
                    let argv = argv.iter().map(|s|s.to_string()).collect();
                    execute(&self.ctx,argv,env,&Some(cwd.clone()),&None,&None).await?;
                },
                Build::Custom {
                    cmd,
                    env,
                } => {
                    let argv : Vec<String> = cmd.split(" ").map(|s|s.to_string()).collect();
                    // let program = argv.first().expect("missing program in build config");
                    // let args = argv[1..].to_vec();
                    execute(&self.ctx,argv,env,&Some(cwd.clone()),&None,&None).await?;

                }
            }
            }

        }

        // return Ok(());

        let ts_start = Instant::now();
        log!("Build","Building {} Version {}",style(&self.ctx.manifest.application.title).cyan(),style(&self.ctx.manifest.application.version).cyan());
        log!("Build","Installer type: {}",style(format!("{:?}", targets)).cyan());

        self.ctx.clean().await?;
        self.ctx.deps.ensure().await?;
        self.ctx.ensure_folders().await?;

        if let Some(actions) = &self.ctx.manifest.package.execute {
            log!("Build","Executing build actions");
            for action in actions {
                if let Execute::Build {cmd, env, folder, platform, arch } = action {
                    let argv = cmd.split(" ").map(|s|s.to_string()).collect();
                    execute(&self.ctx,argv,env,folder,platform,arch).await?;
                }
            }
        }

        cfg_if! {
            if #[cfg(target_os = "windows")] {
                let installer = Box::new(crate::windows::Windows::new(self.ctx.clone()));
            } else if #[cfg(target_os = "macos")] {
                let installer = Box::new(crate::macos::MacOS::new(self.ctx.clone()));
                // Box::new(windows::Windows::new(self.ctx.clone()))
            } else if #[cfg(target_os = "linux")] {
                let installer = Box::new(crate::linux::Linux::new(self.ctx.clone()));
            }
        }

        let files = installer.create(targets).await?;

        if files.is_empty() {
            panic!("Build produced no output");
        }

        let duration = ts_start.elapsed();

        log!("Build","generating signatures");
        for file in files.iter() {
            generate_sha256sum(&file).await?;
        }

        for file in files.iter() {
            log!("Package","{}", style(file.to_str().unwrap()).cyan());
        }

        let packages = if files.len() > 1 { "packages" } else { "package" };
        log!("Finished","build of ({} {}) completed in{:.2}s", files.len(), packages, duration.as_millis() as f64/1000.0);

        if let Some(actions) = &self.ctx.manifest.package.execute {
            log!("Build","Executing deploy actions");
            for action in actions {
                if let Execute::Deploy { cmd, env, folder, platform, arch } = action {
                    let argv = cmd.split(" ").map(|s|s.to_string()).collect();
                    execute(&self.ctx,argv,env,folder,platform,arch).await?;
                }
            }
        }

        println!("");

        Ok(())
    }

}