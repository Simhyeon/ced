use thiserror::Error;

pub type CedResult<T> = Result<T,CedError>;

#[derive(Error, Debug)]
pub enum CedError {
    #[error("Index out of range")]
    OutOfRangeError,
    #[error("Cell data is invalid")]
    InvalidCellData
}
