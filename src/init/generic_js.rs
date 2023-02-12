use super::*;

pub const NW_TOML: &str = include_str!("../../resources/init/generic-js/nw.toml");
pub const GITIGNORE: &str = include_str!("../../resources/init/generic-js/.gitignore");
pub const INDEX_JS: &str = include_str!("../../resources/init/generic-js/index.js");
pub const INDEX_HTML: &str = include_str!("../../resources/init/generic-js/app/index.html");
pub const APP_JS: &str = include_str!("../../resources/init/generic-js/app/app.js");

pub async fn generate(project: &Project, manifest: bool) -> Result<()> {
    let tpl = project.tpl()?;
    let files = if manifest {
        [("nw.toml", tpl.transform(generic_js::NW_TOML))].to_vec()
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
            (".gitignore", generic_js::GITIGNORE.to_string()),
            ("package.json", tpl.transform(&package_json)),
            ("index.js", tpl.transform(generic_js::INDEX_JS)),
            ("app/index.html", tpl.transform(generic_js::INDEX_HTML)),
            ("app/app.js", tpl.transform(generic_js::APP_JS)),
            ("nw.toml", tpl.transform(generic_js::NW_TOML)),
        ]
        .to_vec()
    };

    let images = project.images();
    project.create_folders(&files, &images).await?;
    project.write_files(&files, &images).await?;

    println!("You can run 'nw .' or 'cargo nw run' to start the application");
    println!();

    Ok(())
}
