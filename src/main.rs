use crate::manifest::*;
use crate::result::Result;
use clap::{Parser,Subcommand};
#[allow(unused_imports)]
use duct::cmd;
// use console::style;

mod error;
mod result;
mod manifest;
mod darwin;
mod dmg;
mod build;
mod utils;
mod platform;

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
    Build,
    /// Clean cache files
    Clean { 
        #[clap(short, long)]
        all : Option<bool>
    },
}


pub async fn async_main() -> Result<()> {
    
    // let cwd = std::env::current_dir()?;
    let args = Cmd::parse();
    let manifest = Manifest::load().await?;
    let action = match args { Cmd::Args(args) => args.action };
    match action {
        Action::Build => {

            println!("build manifest: {:#?}", manifest);
            // for build in manifest.build.expect("no build directives found").iter() {
            //     build.execute().await?;
            // }
        },
        Action::Clean { all } => {
            let all = all.unwrap_or(false);
            println!("clean all: {:?} manifest: {:#?}", all, manifest);
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

