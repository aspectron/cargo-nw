use crate::prelude::*;
use async_std::fs;
use async_std::path::PathBuf;
use async_std::task::sleep;
use console::style;
use duct::cmd;

pub struct DMG {
    app_name: String,
    app_title: String,
    // app_version : String,
    app_folder: PathBuf,
    mount_icon: PathBuf,
    background_image: PathBuf,
    build_folder: PathBuf,
    // options : Option<MacOsDiskImage>,
    options: Option<MacOsDiskImage>,
    // output_folder : PathBuf,
    build_file: PathBuf,
    output_file: PathBuf,
}

impl DMG {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        app_name: &str,
        app_title: &str,
        app_version: &str,
        platform: &str,
        arch: &str,
        app_folder: &PathBuf,
        mount_icon: &PathBuf,
        background_image: &PathBuf,
        options: &Option<MacOsDiskImage>,
        build_folder: &PathBuf,
        output_folder: &PathBuf,
    ) -> DMG {
        let filename = format!("{app_name}-{app_version}-{platform}-{arch}");
        let build_file = build_folder.join(format!("{filename}.build.dmg"));
        let output_file = output_folder.join(format!("{filename}.dmg"));

        DMG {
            app_name: app_name.to_string(),
            app_title: app_title.to_string(),
            // app_version : app_version.to_string(),
            mount_icon: mount_icon.to_path_buf(),
            background_image: background_image.to_path_buf(),
            app_folder: app_folder.to_path_buf(),
            build_folder: build_folder.to_path_buf(),
            options: options.clone(),
            // output_folder : output_folder.to_path_buf(),
            build_file,
            output_file,
        }
    }

    async fn configure_finder(&self) -> Result<()> {
        let options = self.options.clone().unwrap_or_default();
        let window_caption_height = options.window_caption_height();
        let window_position = options.window_position();
        let window_size = options.window_size();
        let icon_size = options.icon_size();
        let application_icon_position = options.application_icon_position();
        let system_applications_folder_position = options.system_applications_folder_position();

        // let caption_bar_height = 59;
        let window_width = window_size[0];
        let window_height = window_size[1] + window_caption_height;
        let window_l = window_position[0];
        let window_t = window_position[1] + window_caption_height;
        let window_r = window_l + window_width;
        let window_b = window_t + window_height;
        let icon_l = application_icon_position[0];
        let icon_t = application_icon_position[1];
        let apps_icon_l = system_applications_folder_position[0];
        let apps_icon_t = system_applications_folder_position[1];
        // let apps_icon_t = icon_t;
        // let apps_icon_l = window_width - 100;

        let app_name = &self.app_name;
        let app_title = &self.app_title;

        // set the bounds of container window to {{{window_t}, {window_l}, {window_r}, {window_b}}}\n\

        let script =
        format!("\
            tell application \"Finder\"\n\
                tell disk \"{app_title}\"\n\
                    open\n\
                    delay 1\n\
                    set current view of container window to icon view\n\
                    set toolbar visible of container window to false\n\
                    set statusbar visible of container window to false\n\
                    set the bounds of container window to {{{window_l}, {window_t}, {window_r}, {window_b}}}\n\
                    set theViewOptions to the icon view options of container window\n\
                    set arrangement of theViewOptions to not arranged\n\
                    set icon size of theViewOptions to {icon_size}\n\
                    set background picture of theViewOptions to file \".background:{app_name}.png\"\n\
                    set position of item \"{app_title}.app\" of container window to {{{icon_l}, {icon_t}}}\n\
                    set position of item \"Applications\" of container window to {{{apps_icon_l}, {apps_icon_t}}}\n\
                    update without registering applications\n\
                    delay 5\n\
                    close\n\
                end tell\n\
            end tell\n\
        ");

        // make new alias file at container window to POSIX file "/Applications" with properties {name:"Applications"}
        // println!("{}",script);

        let osa_script_file = self.build_folder.join("osa");
        fs::write(&osa_script_file, script).await?;

        cmd!("osascript", osa_script_file.to_str().unwrap())
            .stdout_null()
            .run()?;

        std::fs::remove_file(osa_script_file)?;

        Ok(())
    }

    async fn copy_aux_files(&self, mountpoint: &PathBuf) -> Result<()> {
        let background_folder = mountpoint.join(".background");
        std::fs::create_dir_all(&background_folder)?;

        // let from = self.ctx.setup_resources_folder.join("background.png");
        let to = background_folder.join(format!("{}.png", self.app_name));
        fs::copy(&self.background_image, to).await?;

        #[cfg(target_family = "unix")]
        std::os::unix::fs::symlink("/Applications", mountpoint.join("Applications"))?;

        Ok(())
    }

    async fn configure_icon(&self, mountpoint: &PathBuf) -> Result<()> {
        let icns = &self.mount_icon;
        let volume_icns = mountpoint.join(".VolumeIcon.icns");
        std::fs::copy(icns, &volume_icns)?;
        cmd!("setfile", "-c", "icnC", volume_icns)
            .stdout_null()
            .run()?;
        cmd!("setfile", "-a", "C", mountpoint).stdout_null().run()?;
        Ok(())
    }

    pub async fn create(&self) -> Result<PathBuf> {
        let volume_name = &self.app_title;
        let mountpoint = PathBuf::from(format!("/Volumes/{volume_name}"));

        if std::path::Path::new(&mountpoint).exists() {
            log_info!("DMG", "{}", style("detaching existing DMG image").yellow());
            cmd!("hdiutil", "detach", &mountpoint).stdout_null().run()?;
        }

        if std::path::Path::new(&self.build_file).exists() {
            std::fs::remove_file(&self.build_file)?;
        }

        if std::path::Path::new(&self.output_file).exists() {
            std::fs::remove_file(&self.output_file)?;
        }

        log_info!("DMG", "creating (UDRW HFS+)");
        cmd!(
            "hdiutil",
            "create",
            "-volname",
            volume_name,
            "-srcfolder",
            &self.app_folder,
            "-ov",
            "-fs",
            "HFS+",
            "-format",
            "UDRW",
            &self.build_file
        )
        .stdout_null()
        .run()?;

        // println!("vvv: {:?}", vvv);

        log_info!("DMG", "attaching");
        cmd!(
            "hdiutil",
            "attach",
            "-readwrite",
            "-noverify",
            "-noautoopen",
            &self.build_file,
        )
        .stdout_null()
        .run()?;

        log_info!("DMG", "configuring DMG window");
        self.copy_aux_files(&mountpoint).await?;
        self.configure_finder().await?;
        log_info!("DMG", "configuring DMG icon");
        self.configure_icon(&mountpoint).await?;

        log_info!("DMG", "sync");
        cmd!("sync").stdout_null().run()?;
        sleep(std::time::Duration::from_millis(1000)).await;
        cmd!("sync").stdout_null().run()?;

        log_info!("DMG", "detaching");
        cmd!("hdiutil", "detach", &mountpoint).stdout_null().run()?;

        log_info!("DMG", "compressing (UDZO)");
        cmd!(
            "hdiutil",
            "convert",
            "-format",
            "UDZO",
            "-imagekey",
            "zlib-level=9",
            "-o",
            &self.output_file,
            &self.build_file
        )
        .stdout_null()
        .run()?;

        // let dmg_size = std::fs::metadata(&self.output_file)?.len() as f64;
        // log!("DMG","resulting DMG size: {:.2}Mb", dmg_size/1024.0/1024.0);

        Ok(self.output_file.clone())
    }
}
