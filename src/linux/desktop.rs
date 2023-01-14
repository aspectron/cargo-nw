use crate::prelude::*;
use async_std::{fs, path::PathBuf};

pub struct DesktopFile {
    list: Vec<(String, String)>,
    filename: PathBuf,
}

impl DesktopFile {
    pub fn new(filename: PathBuf) -> DesktopFile {
        DesktopFile {
            list: Vec::new(),
            filename: filename,
        }
    }
}

impl DesktopFile {
    pub fn entry(&mut self, k: &str, v: &str) -> &mut Self {
        self.list.push((k.to_string(), v.to_string()));
        self
    }

    pub async fn store(&self) -> Result<()> {
        let text = self
            .list
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<String>>()
            .join("\n");
        let text = format!("[Desktop Entry]\n\n{text}");
        fs::write(&self.filename, text).await?;
        Ok(())
    }
}
