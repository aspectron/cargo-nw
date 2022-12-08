use async_std::path::Path;
use async_std::path::PathBuf;
use duct::cmd;
use std::string::ToString;
use crate::prelude::*;
use super::innosetup::*;

const INNO_SETUP_COMPIL32: &str = "C:/Program Files (x86)/Inno Setup 6/compil32.exe";



pub struct ISS {
    ctx: Arc<Context>,
    app_name : String,
    app_title : String,
    app_group : String,
    app_version : String,
    app_uuid : String,
    app_folder : PathBuf,
    app_authors: String,
    app_url: String,
    app_exe_file: String,
    run_on_startup: Option<String>,
    run_after_setup: Option<bool>,
    setup_icon_file: PathBuf,
    // // setup_icon : PathBuf,
    // // background_image : PathBuf,
    build_folder : PathBuf,
    // // output_folder : PathBuf,

    // // build_file : PathBuf,
    iss_filename : String,
    output_file : PathBuf,
}

impl ISS {
    pub fn new(
        ctx: Arc<Context>,
        setup_icon_file : PathBuf,
    ) -> ISS {

        let windows = ctx.manifest.windows.as_ref().expect("nwjs.toml missing [windows] section");

        let app_name = ctx.manifest.application.name.clone();
        let app_title = ctx.manifest.application.title.clone();
        let app_version = ctx.manifest.application.version.clone();
        let build_folder = ctx.build_folder.clone();
        let app_folder = build_folder.join(&app_title);
        let app_authors = if let Some(authors) = ctx.manifest.application.authors.as_ref() {
            authors.to_string()
        } else {
            format!("{app_title} Developers")
        };
        let app_url = if let Some(url) = ctx.manifest.application.url.as_ref() {
            url.to_string()
        } else {
            "N/A".to_string()
        };

        let app_group = windows.group.to_string();
        let app_uuid = windows.uuid.to_string();

        let app_exe_file = if let Some(executable) = windows.executable.as_ref() {
            executable.to_string()
        } else {
            Path::new(&format!("{app_name}.exe")).to_str().unwrap().to_string()
        };

        let iss_filename = format!("{}-{}-{}-{}",
            app_name,
            app_version,
            ctx.platform,
            ctx.arch,
        );
        let output_file = ctx.output_folder.join(format!("{iss_filename}.exe"));

        let run_on_startup = windows.run_on_startup.clone();
        let run_after_setup = windows.run_after_setup.clone();

        ISS {
            ctx,
            app_name,
            app_title,
            app_group,
            app_uuid,
            app_version,
            app_authors,
            app_url,
            app_folder,
            build_folder,
            iss_filename,
            output_file,
            app_exe_file,
            setup_icon_file,
            run_on_startup,
            run_after_setup,
        }
    }

    pub async fn create(&self) -> Result<PathBuf> {
        self.check_innosetup_compiler()?;
        self.generate_iss().await?;
        Ok(self.output_file.clone())
    }

    pub fn check_innosetup_compiler(&self) -> Result<()> {
        if !std::path::Path::new(INNO_SETUP_COMPIL32).exists() {
            println!("");
            println!("fatal: unable to locate: {}", INNO_SETUP_COMPIL32);
			println!("please download innosetup 6 at:");
			println!("https://jrsoftware.org/isdl.php");
            println!("");
            return Err("Missing InnoSetup Compiler".into())
        }
        Ok(())
    
    }

    pub async fn generate_iss(&self) -> Result<()> {

        let mut iss = InnoSetup::new();

        iss
            .define("AppName",&self.app_title)
            .define("AppGroup", &self.app_group) 
            .define("AppVersion",&self.app_version)
            .define("AppPublisher", &self.app_authors)
            .define("AppURL",&self.app_url)
            .define("AppExeName",&self.app_exe_file);
            
        iss.setup()
            .directives(&[

                ("AppId",&self.app_uuid),
                ("AppName","{#AppName}"),
                ("AppVersion","{#AppVersion}"),
                ("AppVerName","{#AppName} {#AppVersion}"), // ?
                ("AppPublisher","{#AppPublisher}"),
                ("AppPublisherURL","{#AppURL}"),
                ("AppSupportURL","{#AppURL}"),
                ("AppUpdatesURL","{#AppURL}"),
                ("DefaultDirName","{pf64}\\{#AppGroup}\\{#AppName}"),
                ("DisableDirPage","yes"),
                ("UsePreviousAppDir","no"),
                ("OutputBaseFilename",&self.iss_filename),
                ("OutputDir",quote!(self.ctx.output_folder.to_str().unwrap())),
                ("SetupIconFile",quote!(self.setup_icon_file.clone().into_os_string().into_string()?)),
                // ("SetupIconFile",quote!(self.ctx.setup_resources_folder.join(format!("{}.ico",self.app_name)).to_str().unwrap())),
                ("Compression","lzma/normal"),
                ("SolidCompression","yes"),
                // ;PrivilegesRequired=admin
                ("AlwaysShowComponentsList","False"),
                ("ShowComponentSizes","False"),
                ("RestartIfNeededByRun","False"),
                ("MinVersion","0,6.0"),
                // ;
                ("UserInfoPage","False"),
                ("DefaultGroupName","{#AppGroup}"),
                ("UninstallDisplayIcon",&format!("{{app}}\\{}",self.app_exe_file)),
                ("CloseApplications","force"),
                // ; "ArchitecturesAllowed=x64" specifies that Setup cannot run on
                // ; anything but x64.
                ("ArchitecturesAllowed",&self.ctx.arch.to_string()),
                // ; "ArchitecturesInstallIn64BitMode=x64" requests that the install be
                // ; done in "64-bit mode" on x64, meaning it should use the native
                // ; 64-bit Program Files directory and the 64-bit view of the registry.
                ("ArchitecturesInstallIn64BitMode",&self.ctx.arch.to_string()),
                ("WizardImageFile",quote!(self.ctx.setup_resources_folder.join("innosetup-164x314.bmp").to_str().unwrap().to_string())),
                ("WizardSmallImageFile",quote!(self.ctx.setup_resources_folder.join("innosetup-55x58.bmp").to_str().unwrap().to_string())),
                // ("WizardSmallImageFile=<%- path.join(RESOURCES,ident+'-55x58.bmp') %>
            ]);

        iss
            .icons()
            .icon("{group}\\{#AppName}","{app}\\{#AppExeName}",None)
            .icon("{group}\\{cm:UninstallProgram,{#AppName}}","{uninstallexe}",None)
            .icon("{commondesktop}\\{#AppName}","{app}\\{#AppExeName}",Some("desktopicon"))
            .icon("{userappdata}\\Microsoft\\Internet Explorer\\Quick Launch\\{#AppName}","{app}\\{#AppExeName}",Some("quicklaunchicon"));

        iss.tasks()
            .task(
                "desktopicon",
                "{cm:CreateDesktopIcon}",
                "{cm:AdditionalIcons}",
                None,
                None
            )
            .task(
                "quicklaunchicon",
                "{cm:CreateQuickLaunchIcon}",
                "{cm:AdditionalIcons}",
                Some("unchecked"),
                Some(&[("OnlyBelowVersion","0,6.1")])
            );


        iss.files()
            .replicate(
                self.app_folder.to_str().unwrap(),
                "{app}",
                Some("recursesubdirs ignoreversion")
            );

        iss.install_delete()
            .args(&[&[("Type","filesandordirs"),("Name","\"{app}\"")]]);

        if let Some(languages) = self.ctx.manifest.languages.as_ref() {
            let languages = languages.languages.as_ref().unwrap();
            let languages = languages.iter().map(|l|l.as_str()).collect::<Vec<&str>>(); 
            iss.languages(&languages);
        } else {
            iss.languages(&["english"]);
        }

        if let Some(run_on_startup) = self.run_on_startup.as_ref() {
            let root = match run_on_startup.to_lowercase().as_str() {
                "user" | "hkcu" => { "HKCU" },
                "system" | "everyone" | "hklm" => { "HKLM"},
                _ => {
                    panic!("nwjs.toml - unsupported 'run_on_startup' value '{}' must be 'user' or 'everyone'", run_on_startup);
                }
            };

            iss.registry()
            .register(
                root,
                "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                "string",
                &self.app_title,
                &format!("\"\"{{app}}\\{}\"\"",self.app_exe_file),
                Some("uninsdeletevalue")
            );
        }

        let firewall = iss.firewall();
        firewall.add_rule(
            &format!("{} App", self.app_title),
            "{#AppExeName}",
            "in+out"
        );

        if self.run_after_setup.unwrap_or(false) {
            let run = iss.run();
            run.exec(
                "{#AppExeName}",
                None,
                Some("{cm:LaunchProgram,{#StringChange(AppName, '&', '&&')}}"),
                Some("nowait postinstall")
            );
        }

        // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

        let iss_text = iss.to_string();
        let iss_file = self.build_folder.join(format!("{}.iss",self.app_name));
        std::fs::write(&iss_file, &iss_text)?;

        log!("InnoSetup","building...");
        cmd!(INNO_SETUP_COMPIL32, "/cc", iss_file).stdin_null().run()?;
        let setup_size = std::fs::metadata(&self.output_file)?.len() as f64;
        log!("InnoSetup","resulting setup size: {:.2}Mb", setup_size/1024.0/1024.0);
        // let code = await this.utils.spawn(,['/cc', path.join(this.TEMP,this.ident+'-impl.iss')], { cwd : this.ROOT, stdio : 'inherit' });
			
        Ok(())
    }

}