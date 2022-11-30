use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error: {0}")]
    String(String),
    
    #[error("Error: {0}")]
    IoError(#[from] std::io::Error),

    
}
