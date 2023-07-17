use crate::prelude::*;
use async_std::path::Path;
use async_std::path::PathBuf;
use trauma::{
    download::Download,
    download::Status,
    // Error,
    downloader::DownloaderBuilder,
};

// fn to_target(dir: &PathBuf, folder: &str) -> PathBuf {
//     Path::new(dir).join(folder)//.into_os_string().into_string().unwrap()
// }

#[derive(Debug, Clone)]
pub struct Meta {
    pub file: String,
    pub folder: String,
    pub url: String,
    // pub executable: String,
    target: PathBuf,
    has_folder: bool,
}

impl Meta {
    pub fn new(file: &str, folder: &str, url: &str, target: &PathBuf, has_folder: bool) -> Self {
        Meta {
            file: file.to_string(),
            folder: folder.to_string(),
            url: url.to_string(),
            target: target.as_path().join(folder),
            has_folder,
        }
    }

    pub fn target(&self) -> PathBuf {
        if self.has_folder {
            self.target.as_path().join(&self.folder)
        } else {
            self.target.clone()
        }
    }
}

pub fn get_nwjs_suffix(platform: &Platform) -> String {
    let nw_platform: NwPlatform = platform.clone().into();
    nw_platform.to_string()
}

pub fn get_nwjs_archive_extension(platform: &Platform) -> String {
    match platform {
        Platform::Windows => "zip",
        Platform::Linux => "tar.gz",
        Platform::MacOS => "zip",
    }
    .into()
}

pub fn get_nwjs_ffmpeg_meta(platform: &Platform, arch: &Architecture, manifest: &Manifest, target: &PathBuf) -> Meta {
    let arch = arch.to_nwjs_arch();
    let version = &manifest.nwjs.version();
    let suffix = get_nwjs_suffix(platform);
    let folder = format!("ffmpeg-{version}-{suffix}-{arch}");
    let file = format!("{version}-{suffix}-{arch}.zip");
    let url = format!(
        "https://github.com/iteufel/nwjs-ffmpeg-prebuilt/releases/download/{version}/{file}"
    );
    Meta::new(&file, &folder, &url, target, false)
}

pub fn get_nwjs_sdk_meta(
    platform: &Platform,
    arch: &Architecture,
    manifest: &Manifest,
    target: &PathBuf,
    version_override: Option<String>,
) -> Meta {
    let arch = arch.to_nwjs_arch();
    let version = format!(
        "v{}",
        version_override.unwrap_or(manifest.nwjs.version())
    );
    let suffix = get_nwjs_suffix(platform);
    let folder = format!("nwjs-sdk-{version}-{suffix}-{arch}");
    let archive_extension = get_nwjs_archive_extension(platform);
    let file = format!("{folder}.{archive_extension}");
    let url = format!("https://dl.nwjs.io/{version}/{file}");
    Meta::new(&file, &folder, &url, target, true)
}

pub fn get_nwjs_meta(
    platform: &Platform,
    arch: &Architecture,
    manifest: &Manifest,
    target: &PathBuf,
    version_override: Option<String>,
) -> Meta {
    let arch = arch.to_nwjs_arch();
    let version = format!(
        "v{}",
        version_override.unwrap_or(manifest.nwjs.version())
    );
    let suffix = get_nwjs_suffix(platform);
    let folder = format!("nwjs-{version}-{suffix}-{arch}");
    let archive_extension = get_nwjs_archive_extension(platform);
    let file = format!("{folder}.{archive_extension}");
    let url = format!("https://dl.nwjs.io/{version}/{file}");
    Meta::new(&file, &folder, &url, target, true)
}

#[derive(Debug)]
pub struct Deps {
    pub ffmpeg: Option<Meta>,
    pub nwjs: Meta,
    pub dir: PathBuf,
}

impl Deps {
    pub fn new(
        platform: &Platform,
        arch: &Architecture,
        manifest: &Manifest,
        sdk: bool,
        nwjs_version_override: Option<String>,
    ) -> Deps {
        let home_dir: PathBuf = home::home_dir().unwrap().into();
        let dir: PathBuf = Path::new(&home_dir).join(".cargo-nw");

        let nwjs = if sdk {
            get_nwjs_sdk_meta(platform, arch, manifest, &dir, nwjs_version_override)
        } else {
            get_nwjs_meta(platform, arch, manifest, &dir, nwjs_version_override)
        };

        let ffmpeg = if manifest.nwjs.ffmpeg.unwrap_or(false) {
            Some(get_nwjs_ffmpeg_meta(platform, arch, manifest, &dir))
        } else {
            None
        };

        Deps { dir, nwjs, ffmpeg }
    }

    fn get_targets(&self) -> Vec<Meta> {
        let mut list = Vec::new();
        list.push(self.nwjs.clone());
        if let Some(ffmpeg) = &self.ffmpeg {
            list.push(ffmpeg.clone())
        }
        list
    }

    pub async fn clean(&self) -> Result<()> {
        if self.dir.exists().await {
            log_info!("Cleaning", "`{}`", self.dir.display());
            async_std::fs::remove_dir_all(&self.dir).await?;
        }

        Ok(())
    }

    pub async fn ensure(&self) -> Result<()> {
        // log!("Dependencies","checking");
        let targets = self.get_targets();
        // println!("targets: {:?}", targets);

        let downloads = targets
            .iter()
            .filter(|meta| !std::path::Path::new(&self.dir).join(&meta.folder).exists())
            .collect::<Vec<&Meta>>();

        if !downloads.is_empty() {
            log_info!("Dependencies", "... downloading NW dependencies ...");
            // println!("");

            self.download(&downloads).await?;
            println!();

            for meta in downloads {
                log_info!("Dependencies", "extracting {}", &meta.file);
                let file = Path::new(&self.dir).join(&meta.file);
                // let target_dir = meta.get_extract_path(&self.dir);
                extract(&file, &meta.target.clone()).await?;
            }
        } else {
            // log!("Dependencies","ok");
        }

        Ok(())
    }

    async fn download(&self, list: &[&Meta]) -> Result<()> {
        let downloads: Vec<Download> = list
            .iter()
            .map(|meta| Download::try_from(meta.url.as_str()).unwrap())
            .collect();
        // let reqwest_rs = "https://github.com/seanmonstar/reqwest/archive/refs/tags/v0.11.9.zip";
        // let downloads = vec![Download::try_from(reqwest_rs).unwrap()];
        let downloader = DownloaderBuilder::new()
            .directory(self.dir.clone().into())
            // .directory(PathBuf::from("output").into())
            .build();
        let slist = downloader.download(&downloads).await;

        for summary in slist.iter() {
            match summary.status() {
                Status::Fail(e) => return Err(Error::String(e.into())),
                Status::NotStarted => {
                    return Err(
                        format!("Unable to start download for: {}", summary.download().url).into(),
                    )
                }
                Status::Skipped(msg) => {
                    log_info!("Dependencies", "{}", msg);
                    // return Err(Error::String(e.into()))
                }
                Status::Success => {}
            }
        }

        Ok(())
    }
}
