pub mod dmg;

use crate::prelude::*;
use async_std::fs;
use async_std::path::Path;
use async_std::path::PathBuf;
use cfg_if::cfg_if;
use chrono::Datelike;
use dmg::DMG;
use duct::cmd;
use fs_extra::dir;
use image::imageops::FilterType;
use image::GenericImageView;
use regex::Regex;

pub struct MacOS {
    pub nwjs_root_folder: PathBuf,
    pub app_contents_folder: PathBuf,
    pub app_resources_folder: PathBuf,
    pub target_folder: PathBuf,
    // pub app_nw_folder : PathBuf,
    pub tpl: Tpl,

    pub ctx: Arc<Context>,
}

#[async_trait]
impl Installer for MacOS {
    async fn check(&self, _targets: &TargetSet) -> Result<()> {
        Ok(())
    }

    async fn init(&self, _targets: &TargetSet) -> Result<()> {
        std::fs::create_dir_all(&self.target_folder)?;

        Ok(())
    }

    async fn create(&self, targets: &TargetSet) -> Result<Vec<PathBuf>> {
        self.copy_nwjs_bundle().await?;
        self.copy_app_data().await?;
        self.rename_app_bundle(&self.app_contents_folder).await?;
        self.generate_resource_strings(&self.app_contents_folder)
            .await?;
        self.generate_icons().await?;

        execute_actions(Stage::Package, &self.ctx, &self.tpl, &self.target_folder).await?;

        // if let Some(actions) = &self.ctx.manifest.package.actions {
        //     for action in actions {
        //         log_info!("Build","executing pack action");
        //         if let Execute::Pack(ec) = action {
        //             log_info!("MacOS","executing `{}`",ec.display(Some(&tpl)));
        //             self.ctx.execute_with_context(ec,Some(&self.app_nw_folder),None).await?;
        //         }
        //     }
        // }

        let mut files = Vec::new();
        if !self.ctx.dry_run && targets.contains(&Target::Archive) {
            log_info!("MacOS", "creating archive");

            let level = self
                .ctx
                .manifest
                .package
                .archive
                .clone()
                .unwrap_or_default();
            let filename = Path::new(&format!("{}.zip", self.ctx.app_snake_name)).to_path_buf();
            let target_file = self.ctx.output_folder.join(filename);
            compress_folder(&self.nwjs_root_folder, &target_file, level)?;

            files.push(target_file);
        }

        if !self.ctx.dry_run && targets.contains(&Target::DMG) {
            log_info!("MacOS", "creating DMG build");

            let background_image = find_file(
                &self.ctx.setup_resources_folder,
                &self.ctx.images.macos_disk_image(),
            )
            .await?;

            let dmg = DMG::new(
                &self.ctx.manifest.application.name,
                &self.ctx.manifest.application.title,
                &self.ctx.manifest.application.version,
                &self.ctx.platform.to_string(),
                &self.ctx.arch.to_string(),
                &self.nwjs_root_folder,
                &self.app_resources_folder.join("app.icns"),
                &background_image,
                &self.ctx.manifest.macos_disk_image,
                &self.ctx.build_folder, //.join(&self.ctx.app_snake_name),
                &self.ctx.output_folder,
            );

            let dmg_file = dmg.create().await?; // self.create_dmg().await?;
            files.push(dmg_file);
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

impl MacOS {
    pub fn new(ctx: Arc<Context>) -> MacOS {
        let nwjs_root_folder = ctx
            .build_folder
            .join(format!("{}.app", &ctx.manifest.application.title));
        let target_folder = nwjs_root_folder
            .join("Contents")
            .join("Resources")
            .join("app.nw");

        let tpl = create_installer_tpl(&ctx, &target_folder);

        MacOS {
            app_contents_folder: nwjs_root_folder.join("Contents"),
            app_resources_folder: nwjs_root_folder.join("Contents").join("Resources"),
            target_folder, //: nwjs_root_folder.join("Contents").join("Resources").join("app.nw"),
            nwjs_root_folder,
            ctx,
            tpl,
        }
    }

    fn get_current_framework_folder(&self) -> Result<PathBuf> {
        let frameworks = self
            .app_contents_folder
            .join("Frameworks")
            .join("nwjs Framework.framework")
            .join("Versions");
        let version = std::fs::read_to_string(frameworks.join("Current"))?;
        Ok(frameworks.join(version))
    }

    async fn copy_nwjs_bundle(&self) -> Result<()> {
        let mut options = dir::CopyOptions::new();
        options.content_only = true;
        options.skip_exist = true;

        log_info!("Integrating", "NW binaries");
        dir::copy(
            // &nwjs_deps,
            Path::new(&self.ctx.deps.nwjs.target()).join("nwjs.app"),
            &self.nwjs_root_folder,
            &options,
        )?;

        if self.ctx.manifest.nwjs.ffmpeg.unwrap_or(false) {
            log_info!("Integrating", "FFMPEG binaries");
            fs::copy(
                Path::new(&self.ctx.deps.ffmpeg.as_ref().unwrap().target()).join("libffmpeg.dylib"),
                self.get_current_framework_folder()?.join("libffmpeg.dylib"),
            )
            .await?;
        }

        Ok(())
    }

    async fn copy_app_data(&self) -> Result<()> {
        log_info!("Integrating", "application data");

        // std::fs::create_dir_all(&self.app_nw_folder)?;

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

    async fn generate_icons(&self) -> Result<()> {
        log_info!("MacOS", "generating icons");

        // in the future, refactor to use https://crates.io/crates/icns
        // currently, this crate doesn't support all formats

        let app_icon = find_file(
            &self.ctx.setup_resources_folder,
            &self.ctx.images.macos_application(),
        )
        .await?;
        let document_icon = find_file(
            &self.ctx.setup_resources_folder,
            &self.ctx.images.macos_document(),
        )
        .await?;

        self.generate_icns_internal(&app_icon, &self.app_resources_folder.join("app.icns"))
            .await?;
        self.generate_icns_internal(
            &document_icon,
            &self.app_resources_folder.join("document.icns"),
        )
        .await?;

        Ok(())
    }

    async fn _generate_icns_sips(&self, png: &PathBuf, icns: &PathBuf) -> Result<()> {
        let iconset_folder = self.ctx.cargo_target_folder.join("icns.iconset");
        if !std::path::Path::new(&iconset_folder).exists() {
            std::fs::create_dir_all(&iconset_folder)?;
        }

        let sizes = vec![512, 256, 128, 64, 32, 16];
        for size in sizes {
            let raw = size * 2;
            let name = format!("icon_{size}x{size}@2.png");
            cmd!(
                "sips",
                "-z",
                format!("{raw}"),
                format!("{raw}"),
                png,
                "--out",
                &iconset_folder.join(name)
            ) //.run()?;
            .stdin_null()
            .read()?;

            let name = format!("icon_{size}x{size}.png");
            cmd!(
                "sips",
                "-z",
                format!("{size}"),
                format!("{size}"),
                png,
                "--out",
                &iconset_folder.join(name)
            ) //.run()?;
            .stdin_null()
            .read()?;
        }

        cmd!("iconutil", "-c", "icns", "--output", icns, "icns.iconset")
            .dir(&self.ctx.cargo_target_folder)
            .run()?;

        std::fs::remove_dir_all(iconset_folder)?;

        Ok(())
    }

    async fn generate_icns_internal(&self, png: &PathBuf, icns: &PathBuf) -> Result<()> {
        let mut src =
            image::open(png).unwrap_or_else(|err| panic!("Unable to open {png:?}: {err}"));

        let dimensions = src.dimensions();
        if dimensions.0 != 1024 || dimensions.1 != 1024 {
            log_warn!("Resources", "`{}`", png.display());
            log_warn!(
                "Resources",
                "application icon dimensions are {}x{}; should be 1024x1024",
                dimensions.0,
                dimensions.1
            );
        }

        let iconset_folder = self.ctx.cargo_target_folder.join("icns.iconset");
        if !std::path::Path::new(&iconset_folder).exists() {
            std::fs::create_dir_all(&iconset_folder)?;
        }

        cfg_if! {
            if #[cfg(debug_assertions)] {
                let resize_filter_type = FilterType::Triangle;
            } else {
                let resize_filter_type = FilterType::Lanczos3;
            }
        }

        let sizes = vec![512, 256, 128, 64, 32, 16];
        for size in sizes {
            let dest = src.resize(size * 2, size * 2, resize_filter_type);
            let name = format!("icon_{size}x{size}@2.png");
            dest.save(iconset_folder.join(name)).unwrap();
            let dest = src.resize(size, size, resize_filter_type);
            let name = format!("icon_{size}x{size}.png");
            dest.save(iconset_folder.join(name)).unwrap();
            src = dest;
        }

        cmd!("iconutil", "-c", "icns", "--output", icns, "icns.iconset")
            .dir(&self.ctx.cargo_target_folder)
            .run()?;

        std::fs::remove_dir_all(iconset_folder)?;

        Ok(())
    }

    async fn plist_bundle_rename(
        &self,
        plist_file: &PathBuf,
        name: &str,
        version: Option<&str>,
    ) -> Result<()> {
        let mut text = fs::read_to_string(plist_file).await?;
        //let name = name.replace("-", " ");

        let regex =
            Regex::new(r"<key>CFBundleDisplayName</key>([^<]*)<string>([^<]*)</string>").unwrap();
        let replace = format!("<key>CFBundleDisplayName</key>$1<string>{name}</string>");
        text = regex.replace(&text, replace).to_string();

        let regex = Regex::new(r"<key>CFBundleName</key>([^<]*)<string>([^<]*)</string>").unwrap();
        let replace = format!("<key>CFBundleName</key>$1<string>{name}</string>");
        text = regex.replace(&text, replace).to_string();

        if let Some(version) = version {
            let regex =
                Regex::new(r"<key>CFBundleShortVersionString</key>([^<]*)<string>([^<]*)</string>")
                    .unwrap();
            let replace =
                format!("<key>CFBundleShortVersionString</key>$1<string>{version}</string>");
            text = regex.replace(&text, replace).to_string();
        }

        fs::write(plist_file, text).await?;

        Ok(())
    }

    async fn generate_resource_strings(&self, app_contents_folder: &PathBuf) -> Result<()> {
        let app_title = &self.ctx.manifest.application.title;
        let version = &self.ctx.manifest.application.version;
        let year = format!("{}", chrono::Utc::now().year());

        let copyright = if let Some(copyright) = &self.ctx.manifest.application.copyright {
            copyright.to_string()
        } else if let Some(authors) = &self.ctx.manifest.application.authors {
            format!("Copyright {year} {authors}")
        } else {
            format!("Copyright {year} {app_title} Developers")
        };

        let _resource_text = format!("\
    CFBundleDisplayName = \"{app_title}\";\n\
    CFBundleGetInfoString = \"{app_title} {version}, {copyright}, The Chromium Authors, NW.js contributors, Node.js. All rights reserved.\";\n\
    CFBundleName = \"{app_title}\";\n\
    NSHumanReadableCopyright = \"{copyright}, The Chromium Authors, NW.js contributors, Node.js. All rights reserved.\";\n\
    ");
        // CFBundleGetInfoString = \"nwjs 107.0.5304.88, Copyright 2022 The Chromium Authors, NW.js contributors, Node.js. All rights reserved.\";\n\

        let resource_text = format!("\
    CFBundleName = \"{app_title}\";\n\
    CFBundleGetInfoString = \"{app_title} {version} {copyright}, The Chromium Authors, NW.js contributors, Node.js. All rights reserved.\";\n\
    NSBluetoothAlwaysUsageDescription = \"Once Chromium has access, websites will be able to ask you for access.\";\n\
    NSBluetoothPeripheralUsageDescription = \"Once Chromium has access, websites will be able to ask you for access.\";\n\
    NSCameraUsageDescription = \"Once Chromium has access, websites will be able to ask you for access.\";\n\
    NSHumanReadableCopyright = \"{app_title} {version} {copyright}, The Chromium Authors, NW.js contributors, Node.js. All rights reserved.\";\n\
    NSLocationUsageDescription = \"Once Chromium has access, websites will be able to ask you for access.\";\n\
    NSMicrophoneUsageDescription = \"Once Chromium has access, websites will be able to ask you for access.\";\n\
    ");

        // FIXME setup the contact usage string...
        // NSContactsUsageDescription = \"Details from your contacts can help you fill out forms more quickly in ${app_title}.\";\n\

        let resources_folder = app_contents_folder.join("Resources");
        let paths = std::fs::read_dir(&resources_folder)
            .unwrap_or_else(|_| panic!("unable to iterate {:?}", &resources_folder));
        for file in paths.flatten() {
            if file.file_name().into_string().unwrap().ends_with(".lproj") {
                fs::write(file.path().join("InfoPlist.strings"), &resource_text).await?;
            }
        }

        Ok(())
    }

    async fn rename_app_bundle(&self, app_contents_folder: &PathBuf) -> Result<()> {
        log_info!("MacOS", "configuring application bundle");

        let plist_file = app_contents_folder.join("info.plist");
        self.plist_bundle_rename(
            &plist_file,
            &self.ctx.manifest.application.title,
            Some(&self.ctx.manifest.application.version),
        )
        .await?;

        Ok(())
    }
}
