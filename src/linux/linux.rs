use async_std::path::{PathBuf, Path};
use crate::prelude::*;

pub struct Linux {
    ctx : Arc<Context>,
}

impl Linux {
    pub fn new(ctx: Arc<Context>) -> Linux {
        Linux {
            ctx
        }
    }
}

#[async_trait]
impl Installer for Linux {
    async fn create(&self, targets: TargetSet) -> Result<Vec<PathBuf>> {

        let mut files = Vec::new();

        if targets.contains(&Target::Archive)  || self.ctx.manifest.package.archive.unwrap_or(false) {
            log!("Linux","creating archive");
            
            let filename = Path::new(&format!("{}.tgz",self.ctx.app_snake_name)).to_path_buf();
            files.push(filename);
        }

        if targets.contains(&Target::Snap) {
            log!("Linux","creating SNAP package");
            
            // let filename = Path::new(format!("{}.zip",self.ctx.app_snake_name)).to_path_buf();
            // files.push(filename);
            let snap = crate::linux::snap::Snap::new(&self.ctx);
            snap.create().await?;
        }

        Ok(vec![])
    }
}
