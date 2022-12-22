use async_std::path::*;
use crate::prelude::*;

#[derive(Debug, Clone, Deserialize, Eq, PartialEq)]
pub struct Write {
    pub file : String,
    pub content : String,
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Stage {
    Build,
    Package,
    Deploy,
    Publish,
    // Dependency,
}


#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Action {
    pub platform : Option<Vec<Platform>>,
    pub arch : Option<Vec<Architecture>>,
    pub family : Option<OsFamily>,
    pub stage : Option<Stage>,
    pub items : Vec<ActionItem>,
}


impl Action {
    pub async fn execute(&self, stage : &Stage, ctx: &Context, tpl: &Tpl, src_folder: &Path, dest_folder: &Path) -> Result<()> {

        if stage != self.stage.as_ref().unwrap_or(&Stage::Build) {
            return Ok(());
        }

        if let Some(platforms) = &self.platform {
            if !platforms.contains(&ctx.platform) {
                return Ok(());
            }
        }

        if let Some(arch) = &self.arch {
            if !arch.contains(&ctx.arch) {
                return Ok(());
            }
        }

        if let Some(family) = &self.family {
            if family != &OsFamily::default() {
                return Ok(());
            }
        }

        for item in self.items.iter() {
            item.execute(stage, ctx, tpl, src_folder, dest_folder).await?;
        }

        Ok(())
    }
}


#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionItem {
    pub platform : Option<Vec<Platform>>,
    pub arch : Option<Vec<Architecture>>,
    pub family : Option<OsFamily>,
    pub stage : Option<Stage>,

    pub run : Option<ExecutionContext>,
    pub copy : Option<Copy>,
    pub write : Option<Write>,
    pub script : Option<Script>,
}



impl ActionItem {
    pub async fn execute(&self,stage: &Stage, ctx: &Context, tpl: &Tpl, src_folder: &Path, dest_folder: &Path) -> Result<()> {
        if stage != self.stage.as_ref().unwrap_or(&Stage::Build) {
            return Ok(());
        }

        if let Some(platforms) = &self.platform {
            if !platforms.contains(&ctx.platform) {
                return Ok(());
            }
        }

        if let Some(arch) = &self.arch {
            if !arch.contains(&ctx.arch) {
                return Ok(());
            }
        }

        if let Some(family) = &self.family {
            if family != &OsFamily::default() {
                return Ok(());
            }
        }

        if let Some(execution_context) = &self.run {
            execute_with_context(&ctx, execution_context, Some(src_folder),tpl).await?;
        }

        if let Some(copy_settings) = &self.copy {
            copy(tpl,copy_settings,&src_folder,&dest_folder).await?;
        }

        if let Some(write) = &self.write {
            let file = tpl.transform(&write.file);
            let file = Path::new(&file);

            let parent = file.parent();
            if let Some(parent) = parent {
                async_std::fs::create_dir_all(&parent).await?;
            }
            // println!("writing file: `{}` content: {}", file.display(), write.content);
            async_std::fs::write(&file,&tpl.transform(&write.content)).await?;
        }

        if let Some(script) = &self.script {
            script.execute(tpl,src_folder).await?;
        }

        Ok(())
    }
}

pub async fn execute_actions(
    stage : Stage,
    ctx : &Context,
    tpl : &Tpl,
    // src_folder: &Path,
    // dest_folder: &Path,
    // installer: &Box<dyn Installer>,
    target_folder : &Path,

) -> Result<()> {

    if let Some(actions) = &ctx.manifest.action {
        // let actions = actions
        //     .iter()
        //     .filter(|action|
        //         action
        //         .stage
        //         .as_ref()
        //         .map(|stage|stage == &current_stage)
        //         .unwrap_or(false)
        //     )
        //     .collect::<Vec<_>>();

        // let tpl = ctx.tpl_clone();

        // let target_folder = installer.target_folder();
        for action in actions {
            // println!("execution action: {:?}", action);
            action.execute(&stage, ctx,tpl,&ctx.project_root_folder,&target_folder).await?;
        }
    }

    Ok(())
}
