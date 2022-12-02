// use crate::manifest::*;
// use crate::result::Result;
use clap::{Parser,Subcommand};
#[allow(unused_imports)]
use duct::cmd;

pub mod error;
pub mod result;
pub mod manifest;
// pub mod dmg;
pub mod build;
pub mod utils;
pub mod platform;
pub mod context;
pub mod deps;
pub mod prelude;
pub mod installer;
pub mod macos;
pub mod linux;
pub mod windows;
pub mod log;

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

        #[clap(short, long)]
        archive : bool,
        
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
}


pub async fn async_main() -> Result<()> {
    
    // let cwd = std::env::current_dir()?;
    let args = Cmd::parse();
    let platform = Platform::new();
    let manifest = Manifest::load().await?;
    let action = match args { Cmd::Args(args) => args.action };
    match action {
        Action::Build {
            sdk,
            // target,
            archive,
        } => {

            let installer_type = if archive {
                InstallerType::Archive
            } else {
                match platform {
                    Platform::Windows => InstallerType::InnoSetup,
                    // FIXME - allow user to specify package manager
                    Platform::Linux => InstallerType::Archive,
                    Platform::MacOS => InstallerType::DMG,
                }
            };

            let options = Options {
                sdk : sdk.unwrap_or(false),
            };

            let ctx = Context::new(platform,manifest,options);

            println!("");

            // println!("build context: {:#?}", ctx);
            let build = Build::new(ctx);
            build.execute(installer_type).await?;
            // for build in manifest.build.expect("no build directives found").iter() {
            //     build.execute().await?;
            // }
        },
        Action::Clean { 
            all, 
            deps 
        } => {
            let deps = deps || all;

            let ctx = Context::new(platform,manifest,Options::default());
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
    }

    Ok(())
}

// #[async_std::main]
#[tokio::main]
async fn main() -> Result<()> {
    match async_main().await {
        Err(e) => println!("{}", e),
        Ok(_) => { }
    };
    Ok(())
}

