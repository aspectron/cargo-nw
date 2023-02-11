use crate::prelude::*;
use std::path::PathBuf;

pub struct Runner {
    ctx: Arc<Context>,
}

impl Runner {
    pub fn new(ctx: Arc<Context>) -> Self {
        Self { ctx }
    }

    pub async fn run(&self) -> Result<()> {
        self.ctx.deps.ensure().await?;

        let nwjs_folder = &self.ctx.deps.nwjs.target();

        cfg_if! {
            if #[cfg(target_os = "windows")] {
                let nw = PathBuf::from(nwjs_folder).join("nw.exe");
            } else if #[cfg(target_os = "macos")] {
                let nwjs_folder = nwjs_folder.join("nwjs.app/Contents/MacOS");
                let nw = PathBuf::from(&nwjs_folder).join("nwjs");
            } else {
                let nw = PathBuf::from(nwjs_folder).join("nw");
            }
        }

        println!("{}", nwjs_folder.display());
        // cmd!("ls -la").dir(folder.clone()).run()?;
        // cmd!("pwd").dir(folder.clone()).run()?;
        let folder = self.ctx.app_root_folder.clone();
        cmd!(nw, ".").dir(folder).run()?;

        Ok(())
    }
}
