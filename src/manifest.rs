use std::collections::HashSet;
use async_std::fs::*;
use async_std::path::{PathBuf, Path};
use crate::prelude::*;
use regex::Regex;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
// #[serde(deny_unknown_fields)]
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
    // #[serde(rename = "node-webkit")]
    pub node_webkit : NodeWebkit,
    /// Windows-specific settings
    pub windows : Option<Windows>,
    /// InnoSetup-specific settings
    pub innosetup : Option<InnoSetup>,
    /// Firewall settings
    pub firewall : Option<Firewall>,
    /// Language settings
    pub languages : Option<Languages>,
    /// DMG settings
    pub macos_disk_image: Option<MacOsDiskImage>,
    /// Snap settings
    pub snap : Option<Snap>,
    /// Custom overrides of default icon paths
    pub images : Option<Images>,

    pub action : Option<Vec<Action>>,
    // pub innosetup : HashMap<String, InnoSetupManifest>,
}

// #[derive(Debug, Clone, Deserialize)]
// pub struct InnoSetupManifest(Vec<HashMap<String, String>>);
//  {
//     // dependencies: HashMap<String, String>,
//     // name : String,
//     version : Option<String>,
// }

// #[derive(Debug, Clone, Deserialize)]
// pub struct Test {
//     // dependencies: HashMap<String, String>,
//     // name : String,
//     version : Option<String>,
// }

impl Manifest {

    pub async fn locate(location: Option<String>) -> Result<PathBuf> {
        let cwd = current_dir().await;

        let location = if let Some(location) = location {
            if location.starts_with("~/") {
                home::home_dir().expect("unable to get home directory").join(&location[2..]).into()
            } else {
                let location = Path::new(&location).to_path_buf();
                if location.is_absolute() {
                    location
                } else {
                    cwd.join(&location)
                }
            }
        } else {
            cwd
        };

        let locations = [
            &location,
            &location.with_extension("toml"),
            &location.join("nw.toml")
        ];

        for location in locations.iter() {
            match location.canonicalize().await {
                Ok(location) => {
                    if location.is_file().await {
                        return Ok(location)
                    }
                }, 
                _ => { }
            }
        }

        Err(format!("Unable to locate 'nw.toml' manifest").into())
    }
    
    pub async fn load(toml : &PathBuf) -> Result<Manifest> {
        let nw_toml = read_to_string(toml).await?;
        let mut manifest: Manifest = match toml::from_str(&nw_toml) {
            Ok(manifest) => manifest,
            Err(err) => {
                return Err(format!("Error loading nw.toml: {}", err).into());
            }
        };    

        let folder = toml.parent().unwrap();
        resolve_value_paths(folder, &mut [
            &mut manifest.application.name,
            &mut manifest.application.title,
            &mut manifest.application.version,
            &mut manifest.application.organization,
            &mut manifest.description.short,
            &mut manifest.description.long,
        ]).await?;

        manifest.sanity_checks()?;

        Ok(manifest)
    }
    
    pub fn sanity_checks(&self) -> Result<()> {

        let regex = Regex::new(r"^[^\s]*[a-z0-9-_]*$").unwrap();
        if !regex.is_match(&self.application.name) {
            return Err(format!("invalid application name '{}'", self.application.name).into());
        }

        if self.description.short.len() > 78 {
            return Err(Error::ShortDescriptionIsTooLong);
        }

        Ok(())
    }
}

/// Application section of the nw.toml manifest
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
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
    /// Application license (Open-Source / Commercial)
    pub license: Option<String>,
    /// End User License Agreement (TODO)
    pub eula: Option<String>,
    /// URL of the application.
    pub url: Option<String>,
}

/// Description directives
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Description {
    /// Short application description.
    pub short: String,
    /// Long application description.
    pub long: String,
}



#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionContext {
    pub name: Option<String>,
    pub argv : Option<Vec<String>>,
    pub cmd : Option<String>,
    pub cwd : Option<String>,
    pub platform: Option<Platform>,
    pub arch: Option<Architecture>,
    pub family: Option<PlatformFamily>,
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

    pub fn display(&self, tpl: &Tpl) -> String {
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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
    /// Disables build types except archive (`build all` will result in archive only)
    /// This can be useful for utility projects that do not require interactive installation.
    pub disable : Option<Vec<Target>>,
    /// If present, generates a signature file beside the redistributable.
    /// A signature file like sha256sum contains hex hash of the original file.
    /// Please note that this is a lightweight fingerprinting of the original file contents.
    /// If security is a concern, you should utilize GPG signatures.
    pub signatures: Option<Vec<Signature>>,
    /// Resource folder relative to the manifes file.
    /// This folder should contain the application icon
    /// as well as images and icons needed by setup generators.
    pub resources: Option<String>,
    /// Project root relative to the manifest file. All 
    /// integration operations will occur from this folder.
    pub source: Option<String>,
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
    // pub execute: Option<Vec<Execute>>,
    // pub actions : Vec<Action>,
    /// Customm output folder (default: `target/setup`).
    pub output: Option<String>,
}

/// Copy filter used in `package.include` and `package.exclude` sections
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum CopyFilter {
    #[serde(rename = "glob")]
    Glob(Vec<String>),
    #[serde(rename = "regex")]
    Regex(Vec<String>),
}


/// Copy options used as a part of [`Dependency`] directive
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "copy", deny_unknown_fields)]
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
    /// Rename files to the target filename/pattern
    // pub rename : Option<String>,
    pub file : Option<String>,
}

/// Git directive used as a part of the [`Dependency`] section
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Git {
    /// Git repository url
    pub url : String,
    /// Repository branch
    pub branch : Option<String>,
}

/// Dependency section
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Dependency {
    /// Name of the dependency (will be displayed during the build process)
    pub name : Option<String>,
    pub platform : Option<Vec<Platform>>,
    pub arch : Option<Vec<Architecture>>,
    /// Git url of the dependency repository
    pub git : Option<Git>,
    pub run : Vec<ExecutionContext>,
    pub copy : Vec<Copy>,
}

/// Node Webkit Directives
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "node-webkit", deny_unknown_fields)]
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

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InnoSetup {
    /// Wizard file resizing (default: true)
    pub resize_wizard_files : Option<bool>
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
/// Snap directives
/// 
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snap {
    ///
    /// Snap channel: 'stable', 'devel'; default 'stable'
    /// 
    pub channel : Option<Channel>,
    ///
    /// Snap confinement: 'strict', 'classic', 'devmode'; default: 'classic'
    /// 
    pub confinement : Option<Confinement>,
    ///
    /// Specify additional SNAP interfaces that may be required by your application.
    /// List of the interfaces can be found here: https://snapcraft.io/docs/supported-interfaces
    /// Default SNAP interfaces included are: `browser-support`, `network`, `network-bind`.
    /// 
    pub interfaces : Option<HashSet<String>>,
    ///    
    /// Additional packages (libraries) that should be included for ELF resolution. Packages can be found using
    /// `apt list | grep <package-substring>`.  This may be needed only if you are including your own additional
    /// binaries in the SNAP distributable.
    ///
    pub packages : Option<HashSet<String>>,

    /// SNAP package base (default: `core22`)
    pub base : Option<String>,
}

impl Default for Snap {
    fn default() -> Snap {
        Snap {
            channel : None,
            confinement : None,
            interfaces: None,
            packages: None,
            base: None,
        }
    }
}

///
/// Firewall directives
/// 
/// Instructs InnoSetup to run `advfirewall firewall add rule` command
/// after the application installation on the target computer.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Firewall {
    /// Firewall application settings
    pub application : Option<FirewallApplication>,
    /// Additional firewall rules
    /// If you need to define separate ports for in
    /// and out directions, you need to define separate rules
    pub rules : Option<Vec<FirewallRule>>
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FirewallApplication {
    pub direction : Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FirewallRule {
    pub name : String,
    pub program : String,
    pub direction : Option<String>,
}

/// Language directives
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
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

impl PackageJson {
    pub fn try_load<P>(filepath : P)-> Result<PackageJson> 
    where P : AsRef<std::path::Path>
    {
        let text = std::fs::read_to_string(filepath)?;
        let package_json: PackageJson = serde_json::from_str(&text)?;
        Ok(package_json)
    }
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct CargoToml {
//     pub package : CargoPackage
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct CargoPackage {
//     pub name : String,
//     pub version : String,
//     pub description : String,
// }

// impl CargoToml {
//     pub fn try_load<P>(filepath : P)-> Result<CargoToml> 
//     where P : AsRef<std::path::Path>
//     {
//         let cargo_toml_text = std::fs::read_to_string(filepath)?;
//         let cargo_toml_manifest: CargoToml = match toml::from_str(&cargo_toml_text) {
//             Ok(cargo_toml_manifest) => cargo_toml_manifest,
//             Err(err) => {
//                 return Err(format!("Error loading nw.toml: {}", err).into());
//             }
//         };
//         Ok(cargo_toml_manifest)
//     }
// }


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Algorithm {
    STORE,
    BZIP2,
    DEFLATE,
    ZSTD
}

impl Default for Algorithm {
    fn default() -> Algorithm {
        Algorithm::DEFLATE
    }
}

impl Into<zip::CompressionMethod> for Algorithm {
    fn into(self) -> zip::CompressionMethod {
        match self {
            Algorithm::STORE => zip::CompressionMethod::Stored,
            Algorithm::BZIP2 => zip::CompressionMethod::Bzip2,
            Algorithm::DEFLATE => zip::CompressionMethod::Deflated,
            Algorithm::ZSTD => zip::CompressionMethod::Zstd,
        }
    }
}

impl ToString for Algorithm {
    fn to_string(&self) -> String {
        match self {
            Algorithm::STORE => "STORE",
            Algorithm::BZIP2 => "BZIP2",
            Algorithm::DEFLATE => "DEFLATE",
            Algorithm::ZSTD => "ZSTD",
        }.into()
    }
}

/// Zip Archive compression modes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Archive {
    pub include : Option<bool>,
    pub algorithm : Option<Algorithm>,
    pub subfolder : Option<bool>,
}

impl Default for Archive {
    fn default() -> Self {
        Archive {
            include : Some(true),
            algorithm: Some(Algorithm::default()),
            subfolder: Some(true),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Signature {
    SHA256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MacOsDiskImage {
    pub window_caption_height : Option<i32>,
    pub window_position : Option<[i32;2]>,
    pub window_size : Option<[i32;2]>,
    pub icon_size : Option<i32>,
    pub application_icon_position : Option<[i32;2]>,
    pub system_applications_folder_position : Option<[i32;2]>
}

impl Default for MacOsDiskImage {
    fn default() -> Self {
        MacOsDiskImage {
            window_caption_height : None, 
            window_position : None,
            window_size : None,
            icon_size : None,
            application_icon_position : None,
            system_applications_folder_position : None
        }
    }
}

impl MacOsDiskImage {
    pub fn window_caption_height(&self) -> i32 { self.window_caption_height.unwrap_or(60) }
    pub fn window_position(&self) -> [i32;2] { self.window_position.unwrap_or([200,200]) }
    pub fn window_size(&self) -> [i32;2] { self.window_size.unwrap_or([485,330]) }
    pub fn icon_size(&self) -> i32 { self.icon_size.unwrap_or(72) }
    pub fn application_icon_position(&self) -> [i32;2] { self.application_icon_position.unwrap_or([100,158]) }
    pub fn system_applications_folder_position(&self) -> [i32;2] { self.system_applications_folder_position.unwrap_or([385,158]) }
}



// ~~~

async fn resolve_value_paths(folder: &Path, paths : &mut [&mut String]) -> Result<()> {
    for path in paths.iter_mut() {
        if is_value_path(path) {
            let location = path.clone();
            path.clear();
            let value = match load_value_path(folder, &location).await {
                Ok(value) => value,
                Err(err) => {
                    return Err(format!("unable to locate `{}` in `{}`: {}",location,folder.display(),err).into())
                }
            };
            path.push_str(&value);
        }
    }

    Ok(())
}

fn is_value_path(v : &str) -> bool {
    v.contains("::")
}

async fn load_value_path(folder: &Path, location: &str) -> Result<String> {
    let parts = location.split("::").collect::<Vec<_>>();
    let filename = folder.join(parts[0]).canonicalize().await?;
    let value_path = parts[1].split(".").collect::<Vec<_>>();

    let extension = filename
        .extension()
        .expect(&format!("unable to determine file type for file `{}` due to missing extension",filename.display()))
        .to_str()
        .unwrap();

    match extension {
        "toml" => {
            let text = std::fs::read_to_string(&filename)?;
            let mut v: &toml::Value = &toml::from_str(&text)?;
            for field in value_path.iter() {
                v = v.get(field)
                    .ok_or::<Error>(format!(
                        "unable to resolve the value `{}` in `{}`",
                        value_path.join("."),
                        filename.display()
                    ).into())?;
            }
            Ok(v.as_str().unwrap().to_string())
        }, 
        "json" => {
            let text = std::fs::read_to_string(&filename)?;
            let mut v: &serde_json::Value = &serde_json::from_str(&text)?;
            for field in value_path.iter() {
                v = v.get(field)
                    .ok_or::<Error>(format!(
                        "unable to resolve the value `{}` in `{}`",
                        value_path.join("."),
                        filename.display()
                    ).into())?;
            }
            Ok(v.as_str().unwrap().to_string())
        },
        _ => Err(format!("path parser: file extension `{extension}` is not supported").into())
    }

}