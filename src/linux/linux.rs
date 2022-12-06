use async_std::path::PathBuf;
use crate::prelude::*;

pub struct Linux {
    _ctx : Arc<Context>,
}

impl Linux {
    pub fn new(_ctx: Arc<Context>) -> Linux {
        Linux {
            _ctx
        }
    }
}

#[async_trait]
impl Installer for Linux {
    async fn create(&self, targets: TargetSet) -> Result<Vec<PathBuf>> {

        println!("[linux] creating {:?} installer", targets);


        Ok(vec![])
    }
}
