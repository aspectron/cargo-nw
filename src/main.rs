use std::{sync::Arc, str::FromStr};

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

// #[cfg(target_os = "macos")]
pub mod macos;
// #[cfg(target_os = "linux")]
pub mod linux;
// #[cfg(target_os = "windows")]
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
    let action = match args { Cmd::Args(args) => args.action };
    println!("action: {:?}", action);

    let platform = Platform::default();
    let arch = Architecture::default();
    let manifest = Manifest::load().await?;
    match action {
        Action::Build {
            sdk,
            // target,
            // archive,
            target,
            default
        } => {

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

            let ctx = Arc::new(Context::new(platform,arch,manifest,options));

            println!("");

            // println!("build context: {:#?}", ctx);
            let build = Builder::new(ctx);
            build.execute(targets).await?;
            // for build in manifest.build.expect("no build directives found").iter() {
            //     build.execute().await?;
            // }
        },
        Action::Clean { 
            all, 
            deps 
        } => {
            let deps = deps || all;

            let ctx = Context::new(platform,arch,manifest,Options::default());
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
        Action::Init { js } => {
            println!("TODO - init template project...");

            let project = init::Project::try_new()?;

            project.generate()?;

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

