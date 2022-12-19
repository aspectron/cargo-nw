use async_std::path::*;
use crate::prelude::*;


#[derive(Debug, Clone, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Stage {
    Build,
    Package,
    Deploy,
    Publish,
    // Dependency,
}


// #[derive(Debug, Clone, Deserialize)]
// pub enum PlatformVariant {
//     Any(Platform),
//     List(Vec<Platform>)
// }

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Action {
    pub platform : Option<Vec<Platform>>,
    pub arch : Option<Vec<Architecture>>,
    pub stage : Option<Stage>,

    pub run : Option<Vec<ExecutionContext>>,
    pub copy : Option<Vec<Copy>>,
    pub script : Option<Script>,
}

impl Action {
    pub async fn execute(&self, ctx: &Context, tpl: &Tpl, src_folder: &Path, dest_folder: &Path) -> Result<()> {
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

        if let Some(execution_context_list) = &self.run {
            for execution_context in execution_context_list.iter() {
                execute_with_context(&ctx, execution_context, Some(src_folder),tpl).await?;
            }
        }

        if let Some(copy_settings_list) = &self.copy {
            for copy_settings in copy_settings_list.iter() {
                copy(tpl,copy_settings,&src_folder,&dest_folder).await?;
            }
        }

        if let Some(script) = &self.script {
            script.execute(tpl,src_folder).await?;
        }

        Ok(())
    }
}

pub async fn execute_actions(
    ctx : &Context,
    tpl : &Tpl,
    current_stage : Stage,
    // src_folder: &Path,
    // dest_folder: &Path,
    // installer: &Box<dyn Installer>,
    target_folder : &Path,

) -> Result<()> {

    if let Some(actions) = &ctx.manifest.action {
        let actions = actions
            .iter()
            .filter(|action|
                action
                .stage
                .as_ref()
                .map(|stage|stage == &current_stage)
                .unwrap_or(false)
            )
            .collect::<Vec<_>>();

        // let tpl = ctx.tpl_clone();

        // let target_folder = installer.target_folder();
        for action in actions {
            action.execute(ctx,tpl,&ctx.project_root_folder,&target_folder).await?;
        }
    }

    Ok(())
}
