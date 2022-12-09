use sha2::{Digest, Sha256};
use crate::prelude::*;
// use async_std::path::PathBuf;
use async_std::path::Path;
use async_std::fs;

pub async fn generate_sha256sum(file: &Path) -> Result<()> {

    let extension = format!("{}.sha256sum",file.extension().unwrap().to_string_lossy());
    let mut signature = file.clone().to_path_buf();
    signature.set_extension(extension);

    let data = fs::read(file).await
        .map_err(|_|format!("Unable to read {}", file.to_string_lossy()))?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    let hash = hasher.finalize();
    let hex = format!("{:x}", hash);
    std::fs::write(&signature, hex).expect("Failed to write hash to file");
    Ok(())
}