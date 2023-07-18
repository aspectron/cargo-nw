mod innosetup;
mod iss;

use crate::prelude::*;
use async_std::fs;
use async_std::path::Path;
use async_std::path::PathBuf;
use chrono::Datelike;
use console::style;
use fs_extra::dir;
use image::imageops::FilterType;
use image::GenericImageView;
use std::collections::HashMap;
use winres_edit::*;

pub struct Windows {
    ctx: Arc<Context>,
    tpl: Tpl,
    // nwjs_root_folder: PathBuf,
    target_folder: PathBuf,
    setup_icon_file: PathBuf,
    pub app_exe_file: String,
}

impl Windows {
    pub fn new(ctx: Arc<Context>) -> Windows {
        let nwjs_root_folder = ctx.build_folder.join(&ctx.app_snake_name);
        // let nwjs_root_folder = ctx.build_folder.clone(); // ctx.build_folder.join(&ctx.manifest.application.title);
        let target_folder = if ctx.manifest.package.use_app_nw.unwrap_or(false) {
            nwjs_root_folder
        } else {
            nwjs_root_folder.join("app.nw")
        };
        let app_name = ctx.manifest.application.name.clone();

        let setup_icon_file = if let Some(crate::manifest::Windows {
            setup_icon: Some(setup_icon),
            ..
        }) = &ctx.manifest.windows
        {
            ctx.app_root_folder.join(setup_icon)
        } else {
            ctx.cache_folder.join(format!("{app_name}-setup.ico"))
        };

        let app_exe_file = match ctx.manifest.windows {
            Some(crate::manifest::Windows {
                executable: Some(ref executable),
                ..
            }) => executable.clone(),
            _ => {
                format!("{}.exe", ctx.manifest.application.name)
            }
        };

        let tpl = create_installer_tpl(&ctx, &target_folder);

        Windows {
            ctx,
            tpl,
            app_exe_file,
            // nwjs_root_folder,
            target_folder,
            setup_icon_file,
        }
    }
}

#[async_trait]
impl Installer for Windows {
    async fn init(&self, _targets: &TargetSet) -> Result<()> {
        Ok(())
    }

    async fn check(&self, targets: &TargetSet) -> Result<()> {
        if targets.contains(&Target::InnoSetup)
            && !std::path::Path::new(iss::INNO_SETUP_COMPIL32).exists()
        {
            println!();
            println!("fatal: unable to locate: `{}`", iss::INNO_SETUP_COMPIL32);
            println!("please download innosetup 6 at:");
            println!("https://jrsoftware.org/isdl.php");
            println!();
            return Err("missing InnoSetup compiler".into());
        }

        Ok(())
    }

    async fn create(&self, targets: &TargetSet) -> Result<Vec<PathBuf>> {
        self.copy_nwjs_folder().await?;
        self.copy_app_data().await?;
        self.update_resources().await?;

        // let tpl = create_installer_tpl(
        //     &self.ctx,
        //     &self.nwjs_root_folder
        // );

        execute_actions(Stage::Package, &self.ctx, &self.tpl, &self.target_folder).await?;

        let mut files = Vec::new();

        if !self.ctx.dry_run && targets.contains(&Target::Archive) {
            log_info!("Windows", "creating archive");

            // let filename = Path::new(&format!("{}.zip",self.ctx.app_snake_name)).to_path_buf();
            let level = self
                .ctx
                .manifest
                .package
                .archive
                .clone()
                .unwrap_or_default();
            let filename = Path::new(&format!("{}.zip", self.ctx.app_snake_name)).to_path_buf();
            let target_file = self.ctx.output_folder.join(filename);
            compress_folder(&self.target_folder, &target_file, level)?;

            files.push(target_file);
            // files.push(filename);
        }

        #[cfg(any(target_os = "windows", feature = "multiplatform"))]
        if !self.ctx.dry_run && targets.contains(&Target::InnoSetup) {
            self.create_innosetup_icon(&self.setup_icon_file).await?;
            let wizard_image_files = self.create_innosetup_images().await?;

            let setup_script = iss::ISS::new(
                self.ctx.clone(),
                self.target_folder.clone(),
                self.setup_icon_file.clone(),
                wizard_image_files,
            );

            let filename = setup_script.create().await?;
            files.push(filename);
        }

        Ok(files)
    }

    fn tpl(&self) -> Tpl {
        self.tpl.clone()
    }

    fn target_folder(&self) -> PathBuf {
        self.target_folder.clone()
    }
}

impl Windows {
    async fn copy_nwjs_folder(&self) -> Result<()> {
        let mut options = dir::CopyOptions::new();
        options.content_only = true;
        options.skip_exist = true;

        log_info!("Integrating", "NW binaries");

        dir::copy(
            Path::new(&self.ctx.deps.nwjs.target()),
            &self.target_folder,
            &options,
        )?;

        fs::rename(
            self.target_folder.join("nw.exe"),
            self.target_folder.join(&self.app_exe_file),
        )
        .await?;

        if self.ctx.manifest.nwjs.ffmpeg.unwrap_or(false) {
            log_info!("Integrating", "FFMPEG binaries");
            fs::copy(
                Path::new(&self.ctx.deps.ffmpeg.as_ref().unwrap().target()).join("ffmpeg.dll"),
                self.target_folder.join("ffmpeg.dll"),
            )
            .await?;
        }

        Ok(())
    }

    async fn copy_app_data(&self) -> Result<()> {
        log_info!("Integrating", "application data");

        // let tpl = self.ctx.tpl_clone();
        copy_folder_with_filters(
            &self.ctx.app_root_folder,
            &self.target_folder,
            (&self.tpl, &self.ctx.include, &self.ctx.exclude).try_into()?,
            CopyOptions::new(self.ctx.manifest.package.hidden.unwrap_or(false)),
        )
        .await?;

        self.ctx.update_package_json(&self.target_folder).await?;

        Ok(())
    }

    fn get_resource_strings(&self) -> Vec<(String, String)> {
        let windows = self.ctx.manifest.windows.as_ref();
        let application = &self.ctx.manifest.application;
        let description = &self.ctx.manifest.description;

        let mut list: HashMap<&str, String> = [
            ("ProductVersion", &application.version),
            ("ProductName", &application.title),
            ("FileVersion", &application.version),
            ("FileDescription", &description.short),
            ("InternalName", &application.title),
            ("CompanyName", &application.organization),
            (
                "LegalCopyright",
                &format!(
                    "Copyright © {} {}",
                    chrono::Utc::now().year(),
                    application.organization
                ),
            ),
        ]
        .into_iter()
        .map(|(k, v)| (k, v.to_string()))
        .collect();

        if let Some(copyright) = &application.copyright {
            list.insert("LegalCopyright", copyright.into());
        }

        list.insert("OriginalFilename", self.app_exe_file.clone());

        if let Some(crate::manifest::Windows {
            resources: Some(resources),
            ..
        }) = windows
        {
            for resource in resources {
                match resource {
                    WindowsResourceString::ProductName(value) => {
                        list.insert("ProductName", value.into());
                    }
                    WindowsResourceString::ProductVersion(value) => {
                        list.insert("ProductVersion", value.into());
                    }
                    WindowsResourceString::FileVersion(value) => {
                        list.insert("FileVersion", value.into());
                    }
                    WindowsResourceString::FileDescription(value) => {
                        list.insert("FileDescription", value.into());
                    }
                    WindowsResourceString::CompanyName(value) => {
                        list.insert("CompanyName", value.into());
                    }
                    WindowsResourceString::LegalCopyright(value) => {
                        list.insert("LegalCopyright", value.into());
                    }
                    WindowsResourceString::LegalTrademarks(value) => {
                        list.insert("LegalTrademarks", value.into());
                    }
                    WindowsResourceString::InternalName(value) => {
                        list.insert("InternalName", value.into());
                    }
                    WindowsResourceString::Custom { name, value } => {
                        list.insert(name, value.into());
                    }
                }
            }
        }

        list.into_iter()
            .map(|(k, v)| (self.tpl.transform(k), self.tpl.transform(&v)))
            .collect()
    }

    async fn create_innosetup_images(&self) -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
        log_info!("InnoSetup", "generating wizard image files");

        let mut small_files = Vec::new();
        let small_file_bmp = self.ctx.cache_folder.join("innosetup-wizard-small.bmp");
        let small_file_png = self
            .ctx
            .setup_resources_folder
            .join("innosetup-wizard-small.png");
        let mut small_src = image::open(&small_file_png)
            .unwrap_or_else(|err| panic!("Unable to open '{}': {err}", small_file_png.display()));
        if !small_file_bmp.exists().await {
            small_src.save(&small_file_bmp).unwrap_or_else(|err| {
                panic!("Unable to save '{}': {err}", small_file_bmp.display())
            });
        }
        small_files.push(small_file_bmp.clone());

        let mut large_files = Vec::new();
        let large_file_bmp = self.ctx.cache_folder.join("innosetup-wizard-large.bmp");
        let large_file_png = self
            .ctx
            .setup_resources_folder
            .join("innosetup-wizard-large.png");
        let mut large_src = image::open(&large_file_png)
            .unwrap_or_else(|err| panic!("Unable to open '{}': {err}", large_file_png.display()));
        if !large_file_bmp.exists().await {
            large_src.save(&large_file_bmp).unwrap_or_else(|err| {
                panic!("Unable to save '{}': {err}", large_file_bmp.display())
            });
        }
        large_files.push(large_file_bmp.clone());

        if let Some(innosetup) = &self.ctx.manifest.innosetup {
            if innosetup.resize_wizard_files.unwrap_or(false) {
                return Ok((small_files, large_files));
            }
        }

        // https://jrsoftware.org/ishelp/index.php?topic=setup_wizardsmallimagefile
        let small_resolutions = [
            (138, 140),
            (119, 123),
            (110, 106),
            (92, 97),
            (83, 80),
            (64, 68),
            (55, 55),
        ];

        // https://jrsoftware.org/ishelp/index.php?topic=setup_wizardsmallimagefile
        let large_resolutions = [
            (410, 797),
            (355, 700),
            (328, 604),
            (273, 556),
            (246, 459),
            (192, 386),
            (164, 314),
        ];

        cfg_if! {
            if #[cfg(debug_assertions)] {
                let resize_filter_type = FilterType::Triangle;
            } else {
                let resize_filter_type = FilterType::Lanczos3;
            }
        }

        for (width, height) in small_resolutions.iter() {
            let dimensions = small_src.dimensions();
            if *width < dimensions.0 && *height < dimensions.1 {
                let filename = self
                    .ctx
                    .cache_folder
                    .join(format!("innosetup-wizard-small-{}x{}.bmp", *width, *height));
                if !filename.exists().await {
                    small_src = small_src.resize(*width, *height, resize_filter_type);
                    small_src.save(&filename).unwrap_or_else(|err| {
                        panic!("Unable to save '{}': {err}", filename.display())
                    });
                }
                small_files.push(filename);
            }
        }

        for (width, height) in large_resolutions.iter() {
            let dimensions = large_src.dimensions();
            if *width < dimensions.0 && *height < dimensions.1 {
                let filename = self
                    .ctx
                    .cache_folder
                    .join(format!("innosetup-wizard-large-{}x{}.bmp", *width, *height));
                if !filename.exists().await {
                    large_src = large_src.resize(*width, *height, resize_filter_type);
                    large_src.save(&filename).unwrap_or_else(|err| {
                        panic!("Unable to save '{}': {err}", filename.display())
                    });
                }
                large_files.push(filename);
            }
        }

        Ok((small_files, large_files))
    }

    async fn create_innosetup_icon(&self, ico_file: &PathBuf) -> Result<()> {
        log_info!("InnoSetup", "generating icons");

        if Path::new(ico_file).exists().await {
            return Ok(());
        }

        let app_icon_png = find_file(
            &self.ctx.setup_resources_folder,
            &self.ctx.images.innosetup_icon(),
        )
        .await?;

        let mut src = image::open(&app_icon_png)
            .unwrap_or_else(|err| panic!("Unable to open '{app_icon_png:?}': {err}"));
        let dimensions = src.dimensions();
        if dimensions.0 != 1024 || dimensions.1 != 1024 {
            println!();
            println!(
                "WARNING: {}",
                app_icon_png.file_name().unwrap().to_str().unwrap()
            );
            println!(
                "         ^^^ icon dimensions are {}x{}; must be 1024x1024",
                dimensions.0, dimensions.1
            );
            println!();
        }

        cfg_if! {
            if #[cfg(debug_assertions)] {
                let resize_filter_type = FilterType::Triangle;
            } else {
                let resize_filter_type = FilterType::Lanczos3;
            }
        }

        let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);

        let sizes = vec![256, 128, 64, 32, 16];
        for size in sizes {
            let dest = src.resize(size, size, resize_filter_type);

            let image_data = dest.as_rgba8().expect("Unable to get RGBA8 image data");
            let image_ico = ico::IconImage::from_rgba_data(
                image_data.width(),
                image_data.height(),
                image_data.as_raw().clone(),
            );
            icon_dir.add_entry(ico::IconDirEntry::encode(&image_ico).unwrap());
            src = dest;
        }

        let ico_file_fd = std::fs::File::create(ico_file)?;
        icon_dir.write(ico_file_fd).unwrap();

        Ok(())
    }

    async fn update_resources(&self) -> Result<()> {
        log_info!("Windows", "updating resources");
        let strings = self.get_resource_strings();

        let mut version = self
            .ctx
            .manifest
            .application
            .version
            .trim()
            .split('.')
            .map(|s| s.parse::<u16>().unwrap())
            .collect::<Vec<u16>>();

        if version.len() > 4 {
            return Err(format!(
                "invalid version format '{}' ... must be '1.2.3' or '1.2.3.4'",
                self.ctx.manifest.application.version
            )
            .into());
        }
        if version.len() < 4 {
            version.resize(4, 0);
        }
        let version: [u16; 4] = version
            .clone()
            .try_into()
            .map_err(|_| format!("Unable to parse version '{version:?}'"))?;

        // ~~~

        let app_icon_png = find_file(
            &self.ctx.setup_resources_folder,
            &self.ctx.images.windows_application(),
        )
        .await?;

        let mut app_icon_image = image::open(&app_icon_png)
            .unwrap_or_else(|err| panic!("Unable to open '{app_icon_png:?}': {err}"));

        if app_icon_image.width() < 256 || app_icon_image.height() < 256 {
            log_warn!(
                "Resources",
                "{}",
                style(
                    "application icon image size should be at least 256x256 (1024x1024 for MacOS)"
                )
                .red()
            );
        }
        if app_icon_image.width() > 256 || app_icon_image.height() > 256 {
            app_icon_image = app_icon_image.resize(256, 256, FilterType::Lanczos3);
        }

        let app_icon_image_data = app_icon_image
            .as_rgba8()
            .expect("Unable to get RGBA8 image data");
        let app_icon_image_ico = ico::IconImage::from_rgba_data(
            app_icon_image_data.width(),
            app_icon_image_data.height(),
            app_icon_image_data.as_raw().clone(),
        );
        let app_icon_encoded = ico::IconDirEntry::encode(&app_icon_image_ico).unwrap();
        // let app_res_file = self.ctx.build_folder.join(&self.app_exe_file);
        let app_res_file = self.target_folder.join(&self.app_exe_file);
        // let app_res_file = std::path::PathBuf::from(app_res_file.as_path());
        let mut resources = Resources::new(&std::path::PathBuf::from(app_res_file.as_path()));
        resources.load().unwrap_or_else(|err| {
            panic!(
                "Unable to load resources from '{}': {err}",
                app_res_file.display()
            )
        });
        resources.open().unwrap_or_else(|err| {
            panic!(
                "Unable to open resource file '{}' for updates: {err}",
                app_res_file.display()
            )
        });

        resources
            .find(resource_type::ICON, Id::Integer(1))
            .expect("unable to find main icon")
            .replace(app_icon_encoded.data())?
            .update()?;

        resources
            .get_version_info()?
            .expect("Unable to get version info")
            .set_file_version(&version)
            .set_product_version(&version)
            .insert_strings(
                &strings
                    .iter()
                    .map(|v| (v.0.as_str(), v.1.as_str()))
                    .collect::<Vec<_>>(),
            )
            // .remove_string("LastChange")
            .update()?;

        resources.close();

        Ok(())
    }
}
