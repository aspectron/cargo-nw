use crate::prelude::*;
use async_std::path::Path;

pub enum ExecArgs {
    String(String),
    Argv(Vec<String>),
}

impl ExecArgs {
    pub fn try_new(cmd: &Option<String>, argv: &Option<Vec<String>>) -> Result<ExecArgs> {
        if cmd.is_some() && argv.is_some() {
            Err(format!(
                "cmd and argv can not be present at the same time: '{:?}' and '{:?}' ",
                cmd.as_ref().unwrap(),
                argv.as_ref().unwrap()
            )
            .into())
        } else if let Some(cmd) = cmd {
            Ok(ExecArgs::String(cmd.clone()))
        } else if let Some(argv) = argv {
            Ok(ExecArgs::Argv(argv.clone()))
        } else {
            Err("ExecArgs::try_new() cmd or argv must be present".into())
        }
    }

    pub fn get(&self, tpl: &Tpl) -> Vec<String> {
        match self {
            ExecArgs::String(cmd) => tpl
                .transform(cmd)
                .split(' ')
                .map(|s| s.to_string())
                .collect::<Vec<String>>(),
            ExecArgs::Argv(argv) => argv.iter().map(|v| tpl.transform(v)).collect(),
        }
    }
}

impl From<&[&str]> for ExecArgs {
    fn from(args: &[&str]) -> Self {
        ExecArgs::Argv(args.iter().map(|s| s.to_string()).collect())
    }
}

impl From<Vec<&str>> for ExecArgs {
    fn from(args: Vec<&str>) -> Self {
        ExecArgs::Argv(args.iter().map(|s| s.to_string()).collect())
    }
}

pub async fn execute_with_context(
    ctx: &Context,
    ec: &ExecutionContext,
    cwd: Option<&Path>,
    tpl: &Tpl,
) -> Result<()> {
    let cwd = normalize(tpl.transform(&cwd.unwrap_or(&ctx.app_root_folder).to_string_lossy()))?;
    let cwd = ec
        .cwd
        .as_ref()
        .map(|folder| {
            let folder = Path::new(folder);
            if folder.is_absolute() {
                folder.to_path_buf()
            } else {
                cwd.join(folder)
            }
        })
        .unwrap_or_else(|| cwd.to_path_buf());

    execute(
        ctx,
        &ec.get_args()?,
        &cwd,
        &ec.env,
        &ec.family,
        &ec.platform,
        &ec.arch,
        tpl,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn execute(
    ctx: &Context,
    // ctx: &Context,
    args: &ExecArgs,
    cwd: &Path,
    // cwd: &Option<String>,
    env: &Option<Vec<String>>,
    family: &Option<PlatformFamily>,
    platform: &Option<Platform>,
    arch: &Option<Architecture>,
    tpl: &Tpl,
) -> Result<()> {
    let cwd = normalize(cwd)?;

    if family.is_some() && family.as_ref() != Some(&PlatformFamily::default()) {
        return Ok(());
    }

    if platform.is_some() && platform.as_ref() != Some(&ctx.platform) {
        return Ok(());
    }

    if arch.is_some() && arch.as_ref() != Some(&ctx.arch) {
        return Ok(());
    }

    let argv = args.get(tpl);
    if !cwd.is_dir().await {
        return Err(format!(
            "unable to locate folder: `{}` while running `{:?}`",
            cwd.display(),
            argv
        )
        .into());
    }

    let program = argv
        .first()
        .expect("missing program (frist argument) in the execution config");
    let args = argv[1..].to_vec();

    let mut proc = duct::cmd(program, args).dir(cwd);
    if let Some(env) = env {
        let defs = get_env_defs(env)?;
        for (k, v) in defs.iter() {
            proc = proc.env(k, v);
        }
    }

    if let Err(e) = proc.run() {
        println!("Error executing: {argv:?}");
        Err(e.into())
    } else {
        Ok(())
    }
}
