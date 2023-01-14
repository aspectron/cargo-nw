use crate::prelude::*;
use sha2::{Digest, Sha256};
// use async_std::path::PathBuf;
use async_std::fs;
use async_std::path::Path;

pub async fn generate_signatures(file: &Path, signatures: &Vec<Signature>) -> Result<()> {
    let data = fs::read(file)
        .await
        .map_err(|_| format!("Unable to read {}", file.to_string_lossy()))?;

    for signature in signatures.iter() {
        let (signature_file, hex) = match signature {
            Signature::SHA256 => {
                let extension =
                    format!("{}.sha256sum", file.extension().unwrap().to_string_lossy());
                let mut signature_file = file.clone().to_path_buf();
                signature_file.set_extension(extension);
                let mut hasher = Sha256::new();
                hasher.update(&data);
                let hash = hasher.finalize();
                (signature_file, format!("{:x}", hash))
            }
        };
        std::fs::write(&signature_file, hex).expect("Failed to write hash to file");
    }
    Ok(())
}
