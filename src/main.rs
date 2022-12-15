use std::{sync::Arc, env};
use async_std::path::PathBuf;
use clap::{Parser,Subcommand};
use console::style;

pub mod error;
pub mod result;
pub mod manifest;
pub mod images;
pub mod builder;
pub mod utils;
pub mod archive;
pub mod platform;
pub mod context;
pub mod deps;
pub mod prelude;
pub mod installer;
pub mod log;
pub mod init;
pub mod signatures;
pub mod tpl;
pub mod copy;
pub mod exec;

cfg_if! {
    if #[cfg(feature = "multiplatform")] {
        pub mod macos;
        pub mod linux;
        pub mod windows;
    } else {
        #[cfg(any(target_os = "macos", feature = "unix"))]
        pub mod macos;
        #[cfg(any(target_os = "linux", feature = "unix"))]
        pub mod linux;
        #[cfg(target_os = "windows")]
        pub mod windows;
    }
}

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
    #[clap(name = "nw")]
    #[clap(about, author, version)]
    #[clap(
        setting = clap::AppSettings::DeriveDisplayOrder,
    )]
    Args(Args),
}


#[derive(Debug, clap::Args)]
struct Args {
    /// Location of the nw.toml manifest file
    #[clap(name = "manifest")]
    location: Option<String>,
    /// Action to execute (build,clean,init)
    #[clap(subcommand)]
    action : Action,
    /// Enable verbose mode
    #[clap(short, long)]
    verbose : bool,
    
    #[cfg(feature = "unix")]
    #[clap(short, long)]
    #[cfg(feature = "unix")]
    platform : Platform,
}

#[derive(Subcommand, Debug)]
enum Action {
    /// Build Node Webkit Application package
    Build {
        /// Package using Node Webkit SDK edition
        #[clap(short, long)]
        sdk : Option<bool>,

        #[cfg(any(target_os = "linux", feature = "unix"))]
        #[clap(short, long, help = "Snap distribution channel (linux only)")]
        #[cfg(any(target_os = "linux", feature = "unix"))]
        channel : Option<Channel>,
        
        #[cfg(any(target_os = "linux", feature = "unix"))]
        #[clap(short, long, help = "Snap package confinement (linux only)")]
        #[cfg(any(target_os = "linux", feature = "unix"))]
        confinement : Option<Confinement>,
        
        /// Target platform architecture (x64,ia32,aarch64)
        #[clap(short, long)]
        arch : Option<Architecture>,
        
        /// Output folder
        #[clap(short, long)]
        output : Option<String>,
        
        /// Package target (for multi-target output)
        #[clap(short, long)]
        target : Option<Vec<Target>>,
        
        /// Package target
        #[clap(subcommand)]
        default: Option<Target>,
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
        /// Create 'nw.toml' manifest file only
        #[clap(long)]
        manifest : bool,
        /// Force overwrite existing project files
        #[clap(long)]
        force : bool,
    },
    Publish {
        /// Output folder
        #[clap(short, long)]
        output : Option<String>,
    },
    #[cfg(feature = "test")]
    Test {
        // #[clap(name = "manifest")]
        // manifest: Option<String>,
    }
}


pub async fn async_main() -> Result<()> {
    
    let args = Cmd::parse();
    let Cmd::Args(Args {
        action,
        location,
        verbose,

        #[cfg(feature = "unix")]
        platform

    }) = args;

    cfg_if! {
        if #[cfg(not(feature = "unix"))] {
            let platform = Platform::default();
        }
    }
    
    match action {
        Action::Build {
            // verbose,
            sdk,
            arch,
            target,
            default,
            output,
            #[cfg(any(target_os = "linux", feature = "unix"))]
            channel,
            #[cfg(any(target_os = "linux", feature = "unix"))]
            confinement,
        } => {

            if verbose {
                log::enable_verbose();
            }

            let mut targets = TargetSet::new();
            if let Some(target) = target {
                targets.extend(target);
            }

            if let Some(default) = default {
                targets.insert(default);
            }

            if targets.contains(&Target::All) {
                targets = Target::get_all_targets();
            }

            cfg_if! {
                if #[cfg(not(any(target_os = "linux", feature = "unix")))] {
                    let channel = Channel::default();
                    let confinement = Confinement::default();
                }
            }
            
            let options = Options {
                sdk : sdk.unwrap_or(false),
                channel,
                confinement,
            };

            let arch = arch.unwrap_or_default();
            let ctx = Arc::new(Context::create(
                location,
                output,
                platform,
                arch,
                options
            ).await?);

            if ctx.manifest.package.archive.is_some() {
                targets.insert(Target::Archive);
            }

            let build = Builder::new(ctx);
            build.execute(targets).await?;
        },
        Action::Clean { 
            all, 
            deps 
        } => {
            let deps = deps || all;

            let ctx = Arc::new(Context::create(
                location,
                None,
                platform,
                Architecture::default(),
                Options::default()
            ).await?);

            if deps {
                ctx.deps.clean().await?;
            }

            ctx.clean().await?;

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

        },
        Action::Publish {
            output
        } => {

            let arch = Architecture::default();
            let ctx = Arc::new(Context::create(
                location,
                output,
                platform,
                arch,
                Options::default()
            ).await?);

            if let Some(actions) = &ctx.manifest.package.execute {
                log_info!("Publish","executing actions");
                for action in actions {
                    if let Execute::Publish(ec) = action {
                        log_info!("Publish","executing `{}`",ec.display(Some(&ctx.tpl())));
                        ctx.execute_with_context(ec, None,None).await?;
                    }
                }
            }
        },
        #[cfg(feature = "test")]
        Action::Test {
        } => {

            let arch = Architecture::default();
            let ctx = Arc::new(Context::create(
                location,
                None,
                platform,
                arch,
                Options::default()
            ).await?);

            println!("{:#?}",ctx);
        }
    }

    Ok(())
}

// #[async_std::main]
#[tokio::main]
async fn main() -> Result<()> {
    let result = async_main().await;
    match &result {
        // Err(Error::String(s)) => println!("\n{}", style(s).red()),
        Err(Error::Warning(warn)) => println!("\nWarning: {}\n",style(format!("{}", warn)).yellow()),
        Err(err) => println!("\n{}\n",style(format!("{}", err)).red()),
        Ok(_) => { }
    };

    if result.is_err() {
        std::process::exit(1);
    }
    
    Ok(())
}

