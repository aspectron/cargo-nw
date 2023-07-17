mod desktop;
pub mod snap;

use crate::prelude::*;
use async_std::fs;
use async_std::path::{Path, PathBuf};
use desktop::*;
use fs_extra::dir;

pub struct Linux {
    ctx: Arc<Context>,
    tpl: Tpl,
    pub target_folder: PathBuf,
}

impl Linux {
    pub fn new(ctx: Arc<Context>) -> Linux {
        let nwjs_root_folder = ctx.build_folder.join(&ctx.app_snake_name);
        let target_folder = if ctx.manifest.package.use_app_nw.unwrap_or(false) {
            nwjs_root_folder
        } else {
            nwjs_root_folder.join("app.nw")
        };

        let tpl = create_installer_tpl(&ctx, &target_folder);

        Linux {
            ctx,
            tpl,
            // nwjs_root_folder
            target_folder,
        }
    }
}

#[async_trait]
impl Installer for Linux {
    async fn init(&self, _targets: &TargetSet) -> Result<()> {
        Ok(())
    }
    async fn check(&self, targets: &TargetSet) -> Result<()> {
        if targets.contains(&Target::Snap) {
            if let Err(err) = cmd("snapcraft", ["--version"]).run() {
                println!("{err}");
                return Err(
                    "Unable to run `snapcraft`, please install using `sudo apt install snapcraft`"
                        .into(),
                );
            }
        }

        Ok(())
    }

    async fn create(&self, targets: &TargetSet) -> Result<Vec<PathBuf>> {
        self.copy_nwjs_folder().await?;
        self.rename_app_binary().await?;
        self.copy_app_data().await?;
        self.copy_icons().await?;
        self.create_desktop_file().await?;

        execute_actions(Stage::Package, &self.ctx, &self.tpl, &self.target_folder).await?;

        let mut files = Vec::new();
        if !self.ctx.dry_run {
            log_info!("Linux", "creating archive");

            // archive is needed for both archive target and for snap
            let level = self
                .ctx
                .manifest
                .package
                .archive
                .clone()
                .unwrap_or_default();
            let archive_filename =
                Path::new(&format!("{}.zip", self.ctx.app_snake_name)).to_path_buf();
            let archive_path = self.ctx.output_folder.join(&archive_filename);
            compress_folder(&self.target_folder, &archive_path, level)?;

            if !self.ctx.dry_run && targets.contains(&Target::Archive) {
                files.push(archive_path.clone());
            }

            #[cfg(any(target_os = "linux", feature = "unix"))]
            if !self.ctx.dry_run && targets.contains(&Target::Snap) {
                // let target_file = target_archive.file_name().unwrap().to_str().unwrap();

                let snap = crate::linux::snap::Snap::try_new(&self.ctx, &archive_path)?;
                log_info!(
                    "Linux",
                    "creating Snap package for '{}' channel",
                    snap.data.grade.to_string()
                );
                snap.create().await?;
                let snap_file = snap.build().await?;
                files.push(snap_file);
            }
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

impl Linux {
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

        if self.ctx.manifest.nwjs.ffmpeg.unwrap_or(false) {
            log_info!("Integrating", "FFMPEG binaries");
            fs::create_dir_all(self.target_folder.join("lib")).await?;
            fs::copy(
                Path::new(&self.ctx.deps.ffmpeg.as_ref().unwrap().target()).join("libffmpeg.so"),
                self.target_folder.join("lib").join("libffmpeg.so"),
            )
            .await?;
        }

        Ok(())
    }

    async fn rename_app_binary(&self) -> Result<()> {
        fs::rename(
            self.target_folder.join("nw"),
            self.target_folder.join(&self.ctx.manifest.application.name),
        )
        .await?;
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

        ctx.update_package_json(&self.target_folder).await?;

        Ok(())
    }

    async fn copy_icons(&self) -> Result<()> {
        let app_icon = find_file(
            &self.ctx.setup_resources_folder,
            &self.ctx.images.linux_application(),
        )
        .await?;

        let filename = format!("{}.png", self.ctx.manifest.application.name);
        fs::copy(&app_icon, self.target_folder.join(filename)).await?;

        Ok(())
    }

    async fn create_desktop_file(&self) -> Result<()> {
        let application = &self.ctx.manifest.application;

        // TODO where should this be located?
        let desktop_file = self
            .target_folder
            .join(format!("{}.desktop", application.name));
        let mut df = DesktopFile::new(desktop_file);

        let iconfile = format!("{}.png", application.name);

        df
        .entry("Type","Application")
        .entry("Version",&application.version)
        .entry("Name",&application.title)
        .entry("Comment",&self.ctx.manifest.description.short)
        // .entry("Path","")
        .entry("Exec",&application.name)
        .entry("Icon",&iconfile)
        .entry("Terminal","false")
        // .entry("Categories","")
        ;

        df.store().await?;

        // TODO - not tested!
        let df_install_script_text = format!(
            "\
desktop-file-install --dir=$HOME/.local/share/applications {}.desktop\n\
update-desktop-database $HOME/.local/share/applications\n\
",
            application.name
        );
        let dfinstall_script_file = self.target_folder.join(format!("{}.sh", application.name));
        fs::write(&dfinstall_script_file, df_install_script_text).await?;
        #[cfg(target_family = "unix")]
        fs::set_permissions(
            dfinstall_script_file,
            std::os::unix::fs::PermissionsExt::from_mode(0o755),
        )
        .await?;

        Ok(())
    }
}
