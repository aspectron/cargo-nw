use std::path::PathBuf;
use trauma::{download::Download, downloader::DownloaderBuilder, Error};

use crate::platform::*;
use crate::manifest::*;
use crate::utils::*;

// pub struct NwjsFiles {
//     ffmpeg : String,

// }

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

pub struct FetchMeta {
    pub file : String,
    pub url : String,
}

impl FetchMeta {
    pub fn new(file: &str, url: &str) -> Self {
        FetchMeta {
            file: file.to_string(),
            url: url.to_string(),
        }
    }
}

pub fn get_nwjs_ffmpeg_url(platform: &Platform, manifest : &Manifest) -> FetchMeta {

    let version = &manifest.nwjs.version;
    let suffix = get_nwjs_suffix(platform);
    let file = format!("{version}-{suffix}-x64.zip");
    let url = format!("https://github.com/iteufel/nwjs-ffmpeg-prebuilt/releases/download/{version}/{file}");
    FetchMeta::new(&file,&url)
}

pub fn get_nwjs_sdk_url(platform: &Platform, manifest : &Manifest) -> FetchMeta {
    let version = format!("v{}",manifest.nwjs.version);
    let suffix = get_nwjs_suffix(platform);
    let archive_extension = get_nwjs_archive_extension(platform);
    let file = format!("nwjs-sdk-{version}-{suffix}-x64.{archive_extension}");
    let url = format!("https://dl.nwjs.io/{version}/{file}");
    FetchMeta::new(&file,&url)
}

pub fn get_nwjs_url(platform: &Platform, manifest : &Manifest) -> FetchMeta {
    let version = format!("v{}",manifest.nwjs.version);
    let suffix = get_nwjs_suffix(platform);
    let archive_extension = get_nwjs_archive_extension(platform);
    let file = format!("nwjs-{version}-{suffix}-x64.{archive_extension}");
    let url = format!("https://dl.nwjs.io/{version}/{file}");
    FetchMeta::new(&file,&url)
}

// this.NWJS_SUFFIX = { windows : 'win', darwin : 'osx', linux : 'linux' }[PLATFORM];
// 		this.NWJS_ARCHIVE_EXTENSION = { windows : 'zip', darwin : 'zip', 'linux' : 'tar.gz' }[PLATFORM];

pub async fn download() -> Result<(), Error> {

    let reqwest_rs = "https://github.com/seanmonstar/reqwest/archive/refs/tags/v0.11.9.zip";
    let downloads = vec![Download::try_from(reqwest_rs).unwrap()];
    let downloader = DownloaderBuilder::new()
        .directory(PathBuf::from("output"))
        .build();
    downloader.download(&downloads).await;
    Ok(())
}