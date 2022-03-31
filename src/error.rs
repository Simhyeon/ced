use thiserror::Error;

pub type CedResult<T> = Result<T,CedError>;

#[derive(Error, Debug)]
pub enum CedError {
    #[error("ERR : IO Error =\n{0}")]
    IoError(IoErrorWithMeta),
    #[error("ERR : Index out of range")]
    OutOfRangeError,
    #[error("ERR : Insufficient row data")]
    InsufficientRowData,
    #[error("ERR : Invalid row data =\n{0}")]
    InvalidRowData(String),
    #[error("ERR : Invalid column =\n{0}")]
    InvalidColumn(String),
    #[error("ERR : Invalid cell data =\n{0}")]
    InvalidCellData(String),
    #[cfg(feature="cli")]
    #[error("ERR : Command line error =\n{0}")]
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
