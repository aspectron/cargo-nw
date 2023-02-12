use super::*;

pub const NW_TOML: &str = include_str!("../../resources/init/generic-rs/nw.toml");
pub const GITIGNORE: &str = include_str!("../../resources/init/generic-rs/.gitignore");
pub const INDEX_JS: &str = include_str!("../../resources/init/generic-rs/index.js");
pub const INDEX_HTML: &str = include_str!("../../resources/init/generic-rs/app/index.html");
pub const CARGO_TOML: &str = include_str!("../../resources/init/generic-rs/Cargo.toml");
pub const LIB_RS: &str = include_str!("../../resources/init/generic-rs/src/lib.rs");
pub const BUILD_SH: &str = include_str!("../../resources/init/generic-rs/build.sh");
pub const BUILD_PS1: &str = include_str!("../../resources/init/generic-rs/build.ps1");

pub async fn generate(project: &Project, manifest: bool) -> Result<()> {
    let tpl = project.tpl()?;
    let files = if manifest {
        [("nw.toml", tpl.transform(generic_rs::NW_TOML))].to_vec()
    } else {
        let package = PackageJson {
            name: project.title.clone(),
            // main: "app/index.js".to_string(),
            main: "index.js".to_string(),
            version: Some(project.version.clone()),
            description: Some("".to_string()),
        };
        let package_json = serde_json::to_string_pretty(&package).unwrap();

        [
            (".gitignore", generic_rs::GITIGNORE.to_string()),
            ("package.json", tpl.transform(&package_json)),
            ("index.js", tpl.transform(generic_rs::INDEX_JS)),
            ("app/index.html", tpl.transform(generic_rs::INDEX_HTML)),
            // ("root/page2.html", tpl.transform(PAGE2_HTML)),
            ("src/lib.rs", tpl.transform(generic_rs::LIB_RS)),
            ("nw.toml", tpl.transform(generic_rs::NW_TOML)),
            ("Cargo.toml", tpl.transform(generic_rs::CARGO_TOML)),
            ("build", tpl.transform(generic_rs::BUILD_SH)),
            ("build.ps1", tpl.transform(generic_rs::BUILD_PS1)),
        ]
        .to_vec()
    };

    let images = project.images();
    project.create_folders(&files, &images).await?;
    project.write_files(&files, &images).await?;

    cfg_if! {
        if #[cfg(not(target_os = "windows"))] {
            fs::set_permissions(Path::new("build"), std::os::unix::fs::PermissionsExt::from_mode(0o755)).await?;
        }
    }

    println!("Please run 'build' script to build the project");
    println!("Following this, you can run 'nw .' or 'cargo nw run' to start the application");
    println!();

    Ok(())
}
