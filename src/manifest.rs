use async_std::fs::*;
use async_std::path::{PathBuf, Path};
use crate::prelude::*;
use regex::Regex;
use path_dedot::*;

#[derive(Debug, Clone, Deserialize)]
pub struct Manifest {
    /// Application settings
    pub application : Application,
    /// Description settings
    pub description : Description,
    /// Package build directives
    pub package : Package,
    /// Script for building application dependencies
    #[serde(rename = "dependency")]
    pub dependencies : Option<Vec<Dependency>>,
    /// Node Webkit directives
    #[serde(rename = "node-webkit")]
    pub node_webkit : NodeWebkit,
    /// Windows-specific settings
    pub windows : Option<Windows>,
    /// Firewall settings
    pub firewall : Option<Firewall>,
    /// Language settings
    pub languages : Option<Languages>,
}

impl Manifest {

    pub async fn locate(location: Option<String>) -> Result<PathBuf> {
        let cwd = current_dir().await;

        let location = if let Some(location) = location {
            let location = Path::new(&location).to_path_buf();
            if location.is_absolute() {
                location
            } else {
                cwd.join(&location)
            }
        } else {
            cwd
        };

        let location = match location.extension() {
            Some(extension) if extension.to_str().unwrap() == "toml" && location.is_file().await => {
                Some(location)
            },
            _ => {
                let location = location.join("nw.toml");
                if location.is_file().await {
                    Some(location)
                } else {
                    None
                }
            }
        };

        if let Some(location) = location {
            let location = std::path::PathBuf::from(&location).parse_dot()?.to_path_buf();
            Ok(location.into())
        } else {
            Err(format!("Unable to locate 'nw.toml' manifest").into())
        }
    }
    
    pub async fn load(toml : &PathBuf) -> Result<Manifest> {
        let nwjs_toml = read_to_string(toml).await?;
        let manifest: Manifest = match toml::from_str(&nwjs_toml) {
            Ok(manifest) => manifest,
            Err(err) => {
                return Err(format!("Error loading nw.toml: {}", err).into());
            }
        };    

        manifest.sanity_checks()?;

        Ok(manifest)
    }
    
    pub fn sanity_checks(&self) -> Result<()> {

        let regex = Regex::new(r"^[^\s]*[a-z0-9-_]*$").unwrap();
        if !regex.is_match(&self.application.name) {
            return Err(format!("invalid application name '{}'", self.application.name).into());
        }

        Ok(())
    }
}

/// Application section of the nw.toml manifest
#[derive(Debug, Clone, Deserialize)]
pub struct Application {
    /// Application name (must be alphanumeric, lowercase, underscore and dash)
    /// This name is used to identity the project in file names
    pub name: String,
    /// Project version in 1.2.3 or 1.2.3.4 format
    pub version: String,
    /// Human-readable title of the project
    pub title: String,
    /// Project Authors
    pub authors: Option<String>,
    /// Organization - Used in Windows and MacOS application manifests
    pub organization: String,
    /// Copyright message
    pub copyright: Option<String>,
    /// Trademarks message (included in Windows resources).
    pub trademarks: Option<String>,
    /// URL of the application.
    pub url: Option<String>,
}

/// Description directives
#[derive(Debug, Clone, Deserialize)]
pub struct Description {
    /// Short application description.
    pub short: String,
    /// Long application description.
    pub long: String,
}



#[derive(Debug, Clone, Deserialize)]
// #[allow(non_camel_case_types)]
pub struct ExecutionContext {
    pub name: Option<String>,
    pub argv : Option<Vec<String>>,
    pub cmd : Option<String>,
    pub folder : Option<String>,
    pub platform: Option<Platform>,
    pub arch: Option<Architecture>,
    pub env : Option<Vec<String>>,
}

impl ExecutionContext {
    pub fn validate(&self) -> Result<()> {
        if self.argv.is_none() && self.cmd.is_none() {
            Err(format!("no command or arguments specified").into())
        } else if self.argv.is_some() && self.cmd.is_some() {
            Err(format!("invalid execution arguments - both 'argv' and 'cmd' are not allowed argv: {:?} cmd: {:?}",
                self.argv.as_ref().unwrap(),
                self.cmd.as_ref().unwrap()
            ).into())
        } else {
            Ok(())
        }
    }
    pub fn get_args(&self) -> Result<ExecArgs> {
        ExecArgs::try_new(&self.cmd,&self.argv)
    }

    pub fn display(&self, tpl: Option<&Tpl>) -> String {
        self.name.clone().unwrap_or_else(|| {
            let descr = self.get_args().unwrap().get(tpl).join(" ");
            if descr.len() > 30 {
                format!("{} ...", &descr[0..30])
            } else {
                descr
            }
        })
    }
}

/// Execute actions that are invoked at different stage of the package integration
/// For argument specification please see [`ExecutionContext`]
#[derive(Debug, Clone, Deserialize)]
// #[allow(non_camel_case_types)]
pub enum Execute {
    /// Executed in the project folder after cleanup operations, before the build proceses. 
    /// This stage can be used to prepare external dependencies.
    #[serde(rename = "build")]
    Build(ExecutionContext),
    /// Executed after the application data has been copied into the target
    /// folder.
    #[serde(rename = "pack")]
    Pack(ExecutionContext),
    /// Executed after the installation package integration is complete. This stage
    /// can be used to execute an external script that uploads resulting builds.
    /// 
    /// For example: 
    /// ```
    /// scp $OUTPUT/$NAME-$VERSION* user@server:path/
    /// ```
    /// 
    #[serde(rename = "deploy")]
    Deploy(ExecutionContext),
    /// Executed only when `cargo nw` is executed with `publish` action:
    /// 
    /// ```
    /// cargo nw publish
    /// ```
    /// 
    #[serde(rename = "publish")]
    Publish(ExecutionContext),
}

/// Build directives.
#[derive(Debug, Clone, Deserialize)]
pub enum Build {
    /// Run `wasmpack` before the integration.
    WASM {
        /// Runs `cargo clean` before the build.
        clean : Option<bool>,
        /// Deletes the `target` folder before the build.
        purge : Option<bool>,
        /// Enable `wasmpack` development build.
        dev : Option<bool>,
        /// Specify a custom output directory (default `root/wasm`) 
        /// when running the build command.
        outdir : Option<String>,
        /// Shell arguments for the build command.
        args : Option<String>,
        /// Environment variables for the build command in the
        /// form of "VAR=VALUE" per entry.
        env : Option<Vec<String>>,
    },
    /// Run `npm` before the integration
    NPM {
        /// Deletes `node_modules` folder before running `npm`.
        clean : Option<bool>,
        /// Deletes `package-lock.json` folder before running `npm`.
        #[serde(rename = "clean-package-lock")]
        clean_package_lock : Option<bool>,
        /// Enables `npm` development build. By default the build
        /// process will include `--omit dev` argument, causing
        /// NPM to produce a release build.
        dev : Option<bool>,
        /// Additional command line arguments passed to `npm`.
        args : Option<String>,
        /// Environment variables for the npm build command.
        env : Option<Vec<String>>,
    },
    /// Run a custom script/command before the integration
    #[serde(rename = "custom")]
    Custom(ExecutionContext),
}

/// Package directives
#[derive(Debug, Clone, Deserialize)]
pub struct Package {
    /// Use `.gitignore` entries as part of the `exclude` globs
    /// (default `true`). This prevents files in `.gitignore` to
    /// be copied as a part of the integration process.
    pub gitignore: Option<bool>,
    /// Build actions executed on the source project folder
    /// before any files are copied as a part of the integration
    /// process.
    pub build: Option<Vec<Build>>,
    /// Forces cargo-nw to always generate an Archive build.
    pub archive: Option<Archive>,
    /// If enabled, cargo-nw will generate sha256sum files.
    /// beside the output packages.
    pub signatures: Option<bool>,
    /// Resource folder relative to the manifes file.
    /// This folder should contain the application icon
    /// as well as images and icons needed by setup generators.
    pub resources: Option<String>,
    /// Project root relative to the manifest file. All 
    /// integration operations will occur from this folder.
    pub root: Option<String>,
    /// List of inclusion globs used during the project
    /// integration (default `**/*` - all files).  If you
    /// specify entries in this list, you have to cover all
    /// files that need to be copied.
    pub include: Option<Vec<CopyFilter>>,
    /// List of exclusion globs used during project integration
    /// NOTE: if `gitignore` is true, list of `.gitignore` entries
    /// is copied into this exclusion list at the start of the 
    /// build process.
    pub exclude: Option<Vec<CopyFilter>>,
    /// Copy hidden files (default: false).
    pub hidden: Option<bool>,
    /// Execute actions during different stages of the build process
    /// Supported values are `build`, `pack`, `deploy`, `publish` 
    /// Please see [`Execute`] for additional information.
    pub execute: Option<Vec<Execute>>,
    /// Customm output folder (default: `target/setup`).
    pub output: Option<String>,
}

/// Copy filter used in `package.include` and `package.exclude` sections
#[derive(Debug, Clone, Deserialize)]
pub enum CopyFilter {
    #[serde(rename = "glob")]
    Glob(Vec<String>),
    #[serde(rename = "regex")]
    Regex(Vec<String>),
}


/// Copy options used as a part of [`Dependency`] directive
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "copy")]
pub struct Copy {
    /// Glob filter - allows to specify a list of globs for
    /// file copy. For example:
    /// 
    /// * `["app/*","assets/**/{*.html,*.js}"]`
    /// 
    pub glob : Option<Vec<String>>,
    /// Regex filter - allows to specify a list of regular
    /// expressions for file matching.
    /// 
    /// For example:
    /// *  `["myprogram(.exe|.lib)?$"]` - will match `myprogram`, `myprogram.exe`, `myprogram.lib`
    /// 
    pub regex : Option<Vec<String>>,
    // pub exclude : Option<Vec<CopyFilter>>,
    /// Destination folder relative to the project root
    pub to : String,
    /// Copy hidden files (files that start with `.`) - default: `false`
    pub hidden : Option<bool>,
    /// Copy all source files into the target folder without preserving
    /// subfolders (results in all files being placed in the target folder)
    pub flatten : Option<bool>,
}

/// Git directive used as a part of the [`Dependency`] section
#[derive(Debug, Clone, Deserialize)]
pub struct Git {
    /// Git repository url
    pub url : String,
    /// Repository branch
    pub branch : Option<String>,
}

/// Dependency section
#[derive(Debug, Clone, Deserialize)]
pub struct Dependency {
    /// Name of the dependency (will be displayed during the build process)
    pub name : Option<String>,
    /// Git url of the dependency repository
    pub git : Option<Git>,
    pub run : Vec<ExecutionContext>,
    pub copy : Vec<Copy>,
}

/// Node Webkit Directives
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "node-webkit")]
pub struct NodeWebkit {
    ///
    /// Node Webkit version. This version must be downloadable
    /// from https://nwjs.io/downloads
    /// 
    /// WARNING: If using FFMPEG builds, the available FFMPEG version
    /// must match the Node Webkit version. FFMPEG downloads are available
    /// at: https://github.com/nwjs-ffmpeg-prebuilt/nwjs-ffmpeg-prebuilt/releases/
    /// 
    pub version: String,
    /// Enable automatic  inregration of FFMPEG libraries.
    pub ffmpeg: Option<bool>,
    /// Use Node Webkit SDK edition. Please note that an SDK-including build cane also be
    /// produced via a command line argument as follows:
    /// ```
    /// cargo nw build all --sdk
    /// ```
    /// Be aware that SDK builds allow users access to your application environment
    pub sdk: Option<bool>,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Deserialize)]
pub struct Windows {
    /// UUID string used by InnoSetup for application
    /// registration.
    pub uuid: String,
    /// Windows Start Menu group name in which the 
    /// application will be places.
    pub group: String,
    /// Name of the executable file (`my-file.exe`); 
    /// By default cargo-nw will use  the project name
    /// declared in the application section.
    pub executable: Option<String>,
    /// Create Windows registry entries to auto-start
    /// the application on Windows startup.
    pub run_on_startup: Option<String>,
    /// Causes InnoSetup to prompt the "Would you like to
    /// run the application now" dialog after setup is complete.
    pub run_after_setup: Option<bool>,
    /// Custom path for the setup icon (default `resources/setup/application.png`)
    pub setup_icon: Option<String>,
    /// Custom Windows resource strings that will be added to the
    /// application executable. Additional information can be 
    /// found here: https://learn.microsoft.com/en-us/windows/win32/menurc/string-str
    pub resources : Option<Vec<WindowsResourceString>>
}

/// Windows resource strings: https://learn.microsoft.com/en-us/windows/win32/menurc/string-str
#[derive(Debug, Clone, Deserialize)]
pub enum WindowsResourceString {
    ProductName(String),
    ProductVersion(String),
    FileVersion(String),
    FileDescription(String),
    CompanyName(String),
    LegalCopyright(String),
    LegalTrademarks(String),
    InternalName(String),
    Custom { name : String, value : String },

}

///
/// Firewall directives
/// 
/// Instructs InnoSetup to run `advfirewall firewall add rule` command
/// after the application installation on the target computer.
#[derive(Debug, Clone, Deserialize)]
pub struct Firewall {
    /// list of ports to open for the main application executable.
    pub ports: Option<Vec<String>>,
    /// list of additional firewall rules.
    pub rules: Option<Vec<String>>,
}

/// Language directives
#[derive(Debug, Clone, Deserialize)]
pub struct Languages {
    /// List of languages used by the application. This will configure
    /// InnoSetup to make the matching language set availabe during
    /// the installation.
    pub languages: Option<Vec<String>>,
}

// ~~~

/// `package.json` manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageJson {
    pub name : String,
    pub main : String,
    pub description : Option<String>,
    pub version : Option<String>,
}

/// Zip Archive compression modes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Archive {
    STORE,
    BZIP2,
    DEFLATE,
    ZSTD
}

impl Default for Archive {
    fn default() -> Self {
        Archive::ZSTD
    }
}

impl Into<zip::CompressionMethod> for Archive {
    fn into(self) -> zip::CompressionMethod {
        match self {
            Archive::STORE => zip::CompressionMethod::Stored,
            Archive::BZIP2 => zip::CompressionMethod::Bzip2,
            Archive::DEFLATE => zip::CompressionMethod::Deflated,
            Archive::ZSTD => zip::CompressionMethod::Zstd,
        }
    }
}

impl ToString for Archive {
    fn to_string(&self) -> String {
        match self {
            Archive::STORE => "STORE",
            Archive::BZIP2 => "BZIP2",
            Archive::DEFLATE => "DEFLATE",
            Archive::ZSTD => "ZSTD",
        }.into()
    }
}
