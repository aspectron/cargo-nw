use std::{sync::Arc, str::FromStr, env};

use async_std::path::PathBuf;
// use crate::manifest::*;
// use crate::result::Result;
use clap::{Parser,Subcommand};
#[allow(unused_imports)]
use duct::cmd;

pub mod error;
pub mod result;
pub mod manifest;
// pub mod dmg;
pub mod builder;
pub mod utils;
pub mod platform;
pub mod context;
pub mod deps;
pub mod prelude;
pub mod installer;
pub mod log;
pub mod init;
pub mod signatures;
pub mod tpl;

// #[cfg(target_os = "macos")]
pub mod macos;
// #[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "windows")]
pub mod windows;

use prelude::*;
// mod repository;
// mod build;
// mod run;

#[derive(Debug, Parser)]
#[clap(name = "cargo")]
#[clap(bin_name = "cargo")]
#[clap(
    // setting = AppSettings::SubcommandRequiredElseHelp,
    setting = clap::AppSettings::DeriveDisplayOrder,
    dont_collapse_args_in_usage = true,
)]
enum Cmd {
    #[clap(name = "nwjs")]
    #[clap(about, author, version)]
    #[clap(
        setting = clap::AppSettings::DeriveDisplayOrder,
    )]
    Args(Args),
}


#[derive(Debug, clap::Args)]
struct Args {
    /// Location of the nw.toml manifest file
    #[clap(name = "location")]
    location: Option<String>,
    /// Action to execute (build,clean,init)
    #[clap(subcommand)]
    action : Action,
}

#[derive(Subcommand, Debug)]
enum Action {
    /// Build NWJS package
    Build {
        /// Package using NWJS SDK edition
        #[clap(short, long)]
        sdk : Option<bool>,
        /// Package target (for multi-target output)
        #[clap(short, long)]
        target : Option<Vec<Target>>,
        /// Target platform architecture (x64,ia32,aarch64)
        #[clap(short, long)]
        arch : Option<Architecture>,
        /// Package target
        #[clap(subcommand)]
        default: Option<Target>
    },
    /// Clean intermediate build folders
    Clean { 
        /// Clean only downloaded dependency packages
        #[clap(long)]
        deps : bool,
        /// Clean dependencies and build folders
        #[clap(long)]
        all : bool,
    },
    /// Create NW package template
    Init {
        /// The name of the project
        #[clap(name = "name")]
        name: Option<String>,
        /// JavaScript-only (Do not generate WASM stubs)
        #[clap(long)]
        js : bool,
        /// Create nw.toml manifest only
        #[clap(long)]
        manifest : bool,
        /// Force overwrite existing project files
        #[clap(long)]
        force : bool,
        
    }
}


// cfg_if! {
//     if #[cfg(target_os = "windows")] {

//     } else if #[cfg(target_os = "macos")] {
//     }
// }

impl FromStr for Target {
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err>
    {
        match s {
            "dmg" => Ok(Target::DMG),
            #[cfg(target_os = "macos")]
            "archive" => Ok(Target::Archive),
            #[cfg(target_os = "windows")]
            "innosetup" => Ok(Target::InnoSetup),
            _ => Err(format!("Unsupported target: {}", s).into()),
        }
    }
}


pub async fn async_main() -> Result<()> {
    
    // let cwd = std::env::current_dir()?;
    let args = Cmd::parse();
    let Cmd::Args(Args { action, location }) = args;
    // let action = match args { Cmd::Args(args) => args.action };
    // println!("action: {:?}", action);

    
    let platform = Platform::default();
    // let arch = Architecture::default();
    
    
    match action {
        Action::Build {
            arch,
            sdk,
            // target,
            // archive,
            target,
            default
        } => {

            let mut targets = TargetSet::new();
            if let Some(target) = target {
                targets.extend(target);
            }

            if let Some(default) = default {
                targets.insert(default);
            }

            let options = Options {
                sdk : sdk.unwrap_or(false),
            };

            let arch = arch.unwrap_or_default();
            let ctx = Arc::new(Context::create(
                location,
                platform,
                arch,
                options
            ).await?);

            println!("... executing ...");

            // println!("build context: {:#?}", ctx);

            // return Ok(());

            let build = Builder::new(ctx);
            build.execute(targets).await?;
        },
        Action::Clean { 
            all, 
            deps 
        } => {
            let deps = deps || all;

            // let ctx = Context::create(platform,arch,manifest,project_root,Options::default()).await?;
            let ctx = Arc::new(Context::create(
                location,
                platform,
                Architecture::default(),
                // manifest,
                // project_root,
                Options::default()
            ).await?);
            // println!("clean context: {:#?}", ctx);

            if deps {
                ctx.deps.clean().await?;
            }

            ctx.clean().await?;

            // println!("clean all: {:?} manifest: {:#?}", all, manifest);
            // cmd!("rm","-rf", repository.name()).run()?;

                // let run = manifest.run.expect("no run directive found");
                // run.execute().await?;
        },
        Action::Init {
            name,
            js,
            manifest,
            force,
        } => {
            // let arch = Architecture::default();
            let folder : PathBuf = env::current_dir().unwrap().into();
            let name = if let Some(name) = name {
                name
            } else {
                folder.file_name().unwrap().to_str().unwrap().to_string()
            };
            // let name = name.as_ref().unwrap_or(folder.file_name().expect("").to_str().expect());
            let options = init::Options {
                js, manifest, force
            };
            let mut project = init::Project::try_new(name, folder)?;

            project.generate(options).await?;

        }
    }

    Ok(())
}

// #[async_std::main]
#[tokio::main]
async fn main() -> Result<()> {
    match async_main().await {
        Err(e) => println!("\n{}", e),
        Ok(_) => { }
    };
    Ok(())
}

