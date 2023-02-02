use crate::prelude::*;
use async_std::path::*;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScriptKind {
    Bash,
    Sh,
    Zsh,
    Bat,
    Cmd,
    Ps1,
    Other(String),
}

impl ToString for ScriptKind {
    fn to_string(&self) -> String {
        match self {
            ScriptKind::Bash => "bash",
            ScriptKind::Sh => "sh",
            ScriptKind::Zsh => "zsh",
            ScriptKind::Bat => "bat",
            ScriptKind::Cmd => "cmd",
            ScriptKind::Ps1 => "ps1",
            ScriptKind::Other(s) => s.as_str(),
        }
        .to_string()
    }
}

impl FromStr for ScriptKind {
    type Err = ();
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "bash" => Ok(ScriptKind::Bash),
            "sh" => Ok(ScriptKind::Sh),
            "zsh" => Ok(ScriptKind::Zsh),
            "bat" => Ok(ScriptKind::Bat),
            "cmd" => Ok(ScriptKind::Cmd),
            "ps1" => Ok(ScriptKind::Ps1),
            _ => Ok(ScriptKind::Other(s.to_string())),
        }
    }
}

impl ScriptKind {
    pub fn interpreter(&self) -> Option<Vec<String>> {
        match self {
            ScriptKind::Bash => Some(vec!["bash"]),
            ScriptKind::Sh => Some(vec!["sh"]),
            ScriptKind::Zsh => Some(vec!["zsh"]),
            ScriptKind::Bat => Some(vec!["cmd.exe", "/k"]),
            ScriptKind::Cmd => Some(vec!["cmd.exe", "/k"]),
            ScriptKind::Ps1 => Some(vec!["powershell.exe"]),
            ScriptKind::Other(_) => None,
        }
        .map(|v| v.iter().map(|s| s.to_string()).collect::<Vec<_>>())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Script {
    #[serde(rename = "type")]
    pub kind: ScriptKind,
    pub name: Option<String>,
    pub interpreter: Option<Vec<String>>,
    pub script: String,
}

impl Script {
    pub async fn execute(&self, tpl: &Tpl, cwd: &Path) -> Result<()> {
        let name = self
            .name
            .clone()
            .map(|s| format!("'{s}'"))
            .unwrap_or_else(|| "".to_string());
        log_info!("Script", "running script {name}");
        let file = format!(
            "{}.{}",
            self.name
                .clone()
                .unwrap_or_else(|| Uuid::new_v4().to_string()),
            self.kind.to_string()
        );
        let mut argv = self
            .interpreter
            .clone()
            .or_else(|| self.kind.interpreter())
            .unwrap_or_else(|| {
                panic!(
                    "unable to determine interpreter for script `{file}`; please specify explicitly"
                )
            });
        let file = cwd.join(&file);
        async_std::fs::write(&file, &self.script).await?;
        argv.push(file.to_str().unwrap().to_string());
        let proc = argv.remove(0);

        cmd(proc, argv).dir(cwd).full_env(&tpl.map).run()?;

        Ok(())
    }
}
