use trauma::{
    download::Download, 
    downloader::DownloaderBuilder, 
    download::Status,
    // Error,
};
use async_std::path::Path;
use async_std::path::PathBuf;
use crate::prelude::*;

fn to_target(dir: &PathBuf, folder: &str) -> PathBuf {
    Path::new(dir).join(folder)//.into_os_string().into_string().unwrap()
}

#[derive(Debug, Clone)]
pub struct Meta {
    pub file : String,
    pub folder : String,
    pub url : String,
    // pub extract_to_subfolder : bool,
    pub target: PathBuf,
}

impl Meta {
    pub fn new(file: &str, folder: &str, url: &str, target: &PathBuf) -> Self {
        Meta {
            file: file.to_string(),
            folder: folder.to_string(),
            url: url.to_string(),
            // extract_to_subfolder: false,
            target: to_target(target,folder)
        }
    }
    // pub fn new_with_subfolder(file: &str, folder: &str, url: &str, target: &PathBuf) -> Self {
    //     Meta {
    //         file: file.to_string(),
    //         folder: folder.to_string(),
    //         url: url.to_string(),
    //         target_dir: to_target(target,folder)
    //         // extract_to_subfolder: true,
    //     }
    // }
    // pub fn get_extract_path(&self, dir: &PathBuf) -> PathBuf {
    //     let target = Path::new(dir);
    //     if self.extract_to_subfolder {
    //         target.join(&self.folder)
    //     } else {
    //         target.into()
    //     }
    // }
}

pub fn get_nwjs_suffix(platform: &Platform) -> String {
    match platform {
        Platform::Windows => "win",
        Platform::Linux => "linux",
        Platform::MacOS => "osx",
    }.into()
}

pub fn get_nwjs_archive_extension(platform: &Platform) -> String {
    match platform {
        Platform::Windows => "zip",
        Platform::Linux => "tar.gz",
        Platform::MacOS => "zip",
    }.into()
}

pub fn get_nwjs_ffmpeg_meta(
    platform: &Platform,
    manifest: &Manifest,
    target: &PathBuf,
) -> Meta {

    let version = &manifest.nwjs.version;
    let suffix = get_nwjs_suffix(platform);
    let folder = format!("ffmpeg-{version}-{suffix}-x64");
    let file = format!("{version}-{suffix}-x64.zip");
    let url = format!("https://github.com/iteufel/nwjs-ffmpeg-prebuilt/releases/download/{version}/{file}");
    Meta::new(&file,&folder,&url,&target)
}

pub fn get_nwjs_sdk_meta(
    platform: &Platform,
    manifest : &Manifest,
    target: &PathBuf,
) -> Meta {
    let version = format!("v{}",manifest.nwjs.version);
    let suffix = get_nwjs_suffix(platform);
    let folder = format!("nwjs-sdk-{version}-{suffix}-x64");
    let archive_extension = get_nwjs_archive_extension(platform);
    let file = format!("{folder}.{archive_extension}");
    let url = format!("https://dl.nwjs.io/{version}/{file}");
    Meta::new(&file,&folder,&url,target)
}

pub fn get_nwjs_meta(
    platform: &Platform,
    manifest : &Manifest,
    target: &PathBuf,
) -> Meta {
    let version = format!("v{}",manifest.nwjs.version);
    let suffix = get_nwjs_suffix(platform);
    let folder = format!("nwjs-{version}-{suffix}-x64");
    let archive_extension = get_nwjs_archive_extension(platform);
    let file = format!("{folder}.{archive_extension}");
    let url = format!("https://dl.nwjs.io/{version}/{file}");
    Meta::new(&file,&folder,&url,target)
}

#[derive(Debug)]
pub struct Dependencies {
    pub ffmpeg : Option<Meta>,
    pub nwjs : Meta,
    pub dir : PathBuf,
}

impl Dependencies {
    pub fn new(
        platform: &Platform,
        manifest: &Manifest,
        sdk: bool,
    ) -> Dependencies {
        let home_dir: PathBuf = home::home_dir().unwrap().into();
        let dir: PathBuf = Path::new(&home_dir).join(".nwjs");

        let nwjs = if sdk {
            get_nwjs_sdk_meta(platform, manifest, &dir)
        } else {
            get_nwjs_meta(platform, manifest, &dir)
        };

        let ffmpeg = if manifest.nwjs.ffmpeg.unwrap_or(false) {
            Some(get_nwjs_ffmpeg_meta(platform, manifest, &dir))
        } else {
            None
        };

        Dependencies {
            dir,
            nwjs,
            ffmpeg,
        }
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
            .filter(|meta|
                !std::path::Path::new(&self.dir)
                .join(&meta.folder)
                .exists())
            .collect::<Vec<&Meta>>();

        if !downloads.is_empty() {
            log!("Dependencies","downloading...");
            println!("");
            
            self.download(&downloads).await?;
            println!("");
            
            for meta in downloads {
                log!("Dependencies","extracting {}", &meta.file);
                let file = Path::new(&self.dir).join(&meta.file);
                // let target_dir = meta.get_extract_path(&self.dir);
                extract(&file, &meta.target).await?;
            }
        } else {
            log!("Dependencies","ok");
        }
        
        Ok(())
    }

    async fn download(&self, list: &Vec<&Meta>) -> Result<()> {

        let downloads: Vec<Download> = list.iter().map(|meta|Download::try_from(meta.url.as_str()).unwrap()).collect();
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
                Status::NotStarted => return Err(format!("Unable to start download for: {}",summary.download().url).into()),
                Status::Skipped(msg) => {
                    log!("Dependencies","{}",msg);
                    // return Err(Error::String(e.into()))
                },
                Status::Success => { }
            }
        }

        Ok(())
    }


}