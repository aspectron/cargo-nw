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

    /// Location of nw.toml file
    #[clap(name = "location")]//, help = "location of nw.toml file")]
    location: Option<String>,

    #[clap(subcommand)]
    action : Action,
    // #[clap(short, long)]
    // verbose : Option<bool>,
}

#[derive(Subcommand, Debug)]
enum Action {
    /// Build NWJS package
    Build {
        #[clap(short, long)]
        sdk : Option<bool>,

        // #[clap(short, long)]
        // archive : bool,
        

        #[clap(short, long)]
        target : Option<Vec<Target>>,
        
        #[clap(subcommand)]
        default: Option<Target>
        // #[clap(short, long)]
        // target : Option<String>,

    },
    /// Clean cache files
    Clean { 
        #[clap(long)]
        deps : bool,
        #[clap(long)]
        all : bool,
    },

    Init {

        #[clap(name = "name", help = "the name of the project")]
        name: Option<String>,

        #[clap(long)]
        js : bool,
        
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
    let arch = Architecture::default();
    
    
    match action {
        Action::Build {
            sdk,
            // target,
            // archive,
            target,
            default
        } => {

            // let nw_toml = Manifest::locate(location).await?;
            // let manifest = Manifest::load(&nw_toml).await?;
            // let project_root = nw_toml.parent().unwrap(); //get_parent_folder_name(&nw_toml);
                    // let installer_type = if archive {
            //     Target::Archive
            // } else {
            //     match platform {
            //         Platform::Windows => Target::InnoSetup,
            //         Platform::MacOS => Target::InnoSetup,
            //         // FIXME - allow user to specify package manager
            //         Platform::Linux => Target::Archive,
            //         // Platform::MacOS => InstallerType::DMG,
            //     }
            // };

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

            let ctx = Arc::new(Context::create(
                location,
                platform,
                arch,
                // manifest,
                // project_root,
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
                arch,
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
            js
        } => {
            let folder : PathBuf = env::current_dir().unwrap().into();
            let name = if let Some(name) = name {
                name
            } else {
                folder.file_name().unwrap().to_str().unwrap().to_string()
            };
            // let name = name.as_ref().unwrap_or(folder.file_name().expect("").to_str().expect());
            let mut project = init::Project::try_new(name, folder)?;

            project.generate().await?;

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

