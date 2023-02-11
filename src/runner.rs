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

        let folder = &self.ctx.deps.nwjs.folder;

        cfg_if! {
            if #[cfg(target_arch = "win32")] {
                let nw = PathBuf::from(folder).join("nw.exe");
            } else if #[cfg(target_arch = "macos")] {
                let nw = PathBuf::from(folder).join("nw");
            } else {
                let nw = PathBuf::from(folder).join("nw");
            }
        }

        cmd!(nw, ".").dir(folder).run()?;

        Ok(())
    }
}
