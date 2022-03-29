use thiserror::Error;

pub type CedResult<T> = Result<T,CedError>;

#[derive(Error, Debug)]
pub enum CedError {
    #[error("IO Error : {0}")]
    IoError(IoErrorWithMeta),
    #[error("Index out of range")]
    OutOfRangeError,
    #[error("Invalid row data =\n{0}")]
    InvalidRowData(String),
    #[error("Invalid cell data =\n{0}")]
    InvalidCellData(String),
    #[cfg(feature="cli")]
    #[error("Command line error {0}")]
    CliError(String),
}

impl CedError {
    pub fn io_error(err: std::io::Error, meta : &str) -> Self {
        Self::IoError(IoErrorWithMeta::new(err, meta))
    }
}

pub struct IoErrorWithMeta {
    error: std::io::Error,
    meta: String,
}

impl IoErrorWithMeta {
    pub fn new(error: std::io::Error, meta: &str) -> Self {
        Self {
            error,
            meta: meta.to_owned(),
        }
    }
}

impl std::fmt::Debug for IoErrorWithMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{} :: {}",self.error,self.meta)
    }
}

impl std::fmt::Display for IoErrorWithMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{} :: {}",self.error,self.meta)
    }
}
