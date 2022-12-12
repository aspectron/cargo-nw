use async_std::{fs, path::Path};
use console::style;
use std::{time::Instant, collections::HashSet};
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

        // println!("{:#?}", self.ctx.manifest);
        // return Ok(());

        if targets.len() == 0 {
            return Err("no build targets selected".into());
        }

        self.ctx.clean().await?;
        self.ctx.deps.ensure().await?;
        self.ctx.ensure_folders().await?;
        self.process_dependencies().await?;

        // return Ok(());

        if let Some(builds) = &self.ctx.manifest.package.build {

            log_info!("Build","building...");
            println!("");

            for build in builds.iter() {
                match build {
                    Build::WASM {
                        clean,
                        purge,
                        outdir,
                        args,
                        dev,
                        env,
                    } => {

                        // invoke cargo clean
                        if clean.unwrap_or(false) {
                            log_info!("WasmPack","cargo clean");
                            cmd!("cargo clean").dir(&self.ctx.app_root_folder).run()?;
                        }

                        // delete the entire target folder
                        if purge.unwrap_or(false) && self.ctx.cargo_target_folder.exists().await {
                            log_info!("WasmPack","purging target folder");
                            fs::remove_dir_all(&self.ctx.cargo_target_folder).await?;
                        }

                        let outdir = outdir.clone().unwrap_or("root/wasm".to_string());
                        let name = &self.ctx.manifest.application.name;
                        let mut argv = vec!["wasmpack","build"];
                        if dev.unwrap_or(false) {
                            argv.push("--dev");
                        }
                        argv.extend_from_slice(&["--target","web","--out-name",name.as_str(),"--out-dir",outdir.as_str()]);
                        if let Some(args) = args {
                            argv.extend(args.split(" ").collect::<Vec<_>>());
                        }

                        log_info!("WasmPack","building WASM target");
                        self.ctx.execute(
                            &argv.into(),
                            &self.ctx.app_root_folder,
                            env,
                            &None,
                            &None,
                            None
                        ).await?;
                    },
                    Build::NPM {
                        clean,
                        clean_package_lock,
                        args,
                        dev,
                        env,
                    } => {

                        let node_modules_folder = self.ctx.app_root_folder.join("node_modules");
                        if clean.unwrap_or(false) && node_modules_folder.exists().await {
                            log_info!("NPM","removing node_modules folder");
                            fs::remove_dir_all(&node_modules_folder).await?;
                        }
                        let package_lock_file = self.ctx.app_root_folder.join("package-lock.json");
                        if clean_package_lock.unwrap_or(false) && package_lock_file.exists().await {
                            log_info!("NPM","removing NPM package lock");
                            fs::remove_file(&package_lock_file).await?;
                        }

                        let mut argv = vec!["npm","install"];
                        if !dev.unwrap_or(false) {
                            argv.extend_from_slice(&["--omit","dev"]);
                        }
                        if let Some(args) = args {
                            argv.extend(args.split(" ").collect::<Vec<_>>());
                        }
                        log_info!("NPM","installing");
                        // let argv = argv.iter().map(|s|s.to_string()).collect();
                        self.ctx.execute(
                            &argv.into(),
                            &self.ctx.app_root_folder,
                            env,
                            &None,
                            &None,
                            None
                        ).await?;
                    },
                    Build::Custom(ec) => {
                        log_info!("Build","executing `{}`",ec.display(Some(&self.ctx.tpl())));
                        self.ctx.execute_with_context(ec,None, None).await?;
                    }
                }
            }

            println!("");
        }

        let ts_start = Instant::now();
        log_info!("Build","building {} version {}",style(&self.ctx.manifest.application.title).cyan(),style(&self.ctx.manifest.application.version).cyan());
        let target_list = targets.iter().map(|v|v.to_string()).collect::<Vec<String>>().join(", ");
        log_info!("Build","installer type: {}",style(format!("{}", target_list)).cyan());


        if let Some(actions) = &self.ctx.manifest.package.execute {
            for action in actions {
                if let Execute::Build(ec) = action {
                    log_info!("Build","executing `{}`",ec.display(Some(&self.ctx.tpl())));
                    self.ctx.execute_with_context(ec,None,None).await?;
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
            return Err(Error::Warning("build produced no output".into()));
        }

        let duration = ts_start.elapsed();

        let files: Vec<(_,_)> = files.iter().map(|f|(f.file_name().unwrap().to_owned(),f)).collect();

        if self.ctx.manifest.package.signatures.unwrap_or(false) {
            log_info!("Build","generating signatures (SHA)");
            for (_,path) in files.iter() {
                generate_sha256sum(&path).await?;
            }
        }

        for (_file,path) in files.iter() {
            let package_size = (std::fs::metadata(&path)?.len() as f64) / 1024.0 / 1024.0;
            log_info!("Package","{} - {}", style(path.to_str().unwrap()).cyan(),style(format!("{:.2}Mb", package_size)).cyan());
        }

        log_info!("Finished","build completed in {:.0}s", duration.as_millis() as f64/1000.0);

        if let Some(actions) = &self.ctx.manifest.package.execute {
            for action in actions {
                if let Execute::Deploy(ec) = action {
                    log_info!("Deploy","executing `{}`",ec.display(Some(&self.ctx.tpl())));
                    self.ctx.execute_with_context(ec,None, None).await?;

                    // let argv: Vec<String> = cmd.split(" ").map(|s|s.to_string()).collect();
                    // log_info!("Build","executing deploy action '{}'",argv.first().unwrap());
                    // execute(&self.ctx,argv,env,folder,platform,arch).await?;
                }
            }
        }

        println!("");

        Ok(())
    }

    async fn process_dependencies(&self) -> Result<()> {
        fs::create_dir_all(&self.ctx.dependencies_folder).await?;
        for dep in self.ctx.manifest.dependencies.iter() {
            self.process_dependency(dep).await?;
        }

        Ok(())
    }

    async fn process_dependency(&self, dep : &Dependency) -> Result<()> {

        // let cwd = &self.ctx.dependencies_folder;

        // ^ TODO
        // ^  GIT CLONE OR PULL
        // ^  CHECK IF SUMMARY MATCHES
        // ^  CHECK IF FILES EXIST
        // ^  RUN BUILD IF NOT
        // ^  COPY FILES

        // let files = dep.copy.from.map()

        let mut name = dep.name.clone();

        let (mut rebuild, dep_build_folder, status) = if let Some(git) = &dep.git {

            let repo = Path::new(&git.url).file_name().unwrap().to_str().unwrap();
            let repo_folder = self.ctx.dependencies_folder.join(repo);
            let status_file = self.ctx.dependencies_folder.join(format!("{repo}.status"));
            name = name.or(Some(repo.to_string()));
            
            if repo_folder.is_dir().await {
                log_info!("Git", "pulling `{}`", name.as_ref().unwrap());
                cmd("git",["pull"]).dir(&repo_folder).run()?;
            } else {
                let args = if let Some(branch) = &git.branch {
                    log_info!("Git", "cloning `{}` ({})", name.as_ref().unwrap(),branch);
                    vec!["clone","-b",branch.as_str(),&git.url]
                } else {
                    log_info!("Git", "cloning `{}`", name.as_ref().unwrap());
                    vec!["clone",&git.url]
                };
                cmd("git",args).dir(&self.ctx.dependencies_folder).run()?;
            }

            let status_data = cmd("git",["show","--summary"]).dir(&repo_folder).read()?;
            if status_file.exists().await {
                let last_status_data = fs::read_to_string(&status_file).await?;
                let rebuild = status_data != last_status_data;
                (rebuild, repo_folder, Some((status_file, status_data)))
            } else {
                (true, repo_folder, Some((status_file, status_data)))
            }
        } else {
            (true, self.ctx.dependencies_folder.clone(), None)
        };

        let mut files = Vec::new();

        // if !rebuild {
        //     'outer: 
        let mut to_folders = HashSet::new();
        if let Ok(tpl) = self.ctx.tpl.lock() {
            for copy in dep.copy.iter() {
                for file in &copy.from {
                    let from = dep_build_folder.join(tpl.transform(file));
                    let to_folder = self.ctx.app_root_folder.join(tpl.transform(&copy.to));
                    to_folders.insert(to_folder.clone());
                    let to = to_folder.join(from.file_name().unwrap());
                    files.push((from,to));
                }
            }
        }

        for (file,_) in files.iter() {
            if !file.exists().await {
                rebuild = true;    
                break;
            }
        }

        let name = name.map(|s| s.to_string()).unwrap_or_else(|| "...".into());
        if rebuild {
            log_info!("Dependency", "building `{}`",name);
            for ec in dep.run.iter() {
                self.ctx.execute_with_context(ec, Some(dep_build_folder.as_path()),None).await?;
            }
        } else {
            log_info!("Dependency", "skipping `{}` (build is up to date)",name);
        }

        for folder in to_folders.iter() {
            fs::create_dir_all(folder).await?;
        }

        for (from,to) in files.iter() {
            fs::copy(from,to)
                .await?;
                // .expect(&format!(
                //     "error copying `{}` to `{}`",
                //     from.display(),
                //     to.display()
                // ));
        }

        if let Some((status_file,status_data)) = status {
            fs::write(status_file, status_data).await?;
        }


        Ok(())
    }
}

// enum Status {
//     Rebuild
// }