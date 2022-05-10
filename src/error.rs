use dcsv::DcsvError;

pub type CedResult<T> = Result<T, CedError>;

#[derive(Debug)]
pub enum CedError {
    CommandError(String),
    CsvDataError(DcsvError),
    InvalidColumn(String),
    InvalidPageOperation(String),
    InvalidRowData(String),
    IoError(IoErrorWithMeta),
    OutOfRangeError,
}

impl std::fmt::Display for CedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CommandError(txt) => write!(f,"ERR : Invalid command call =\n{0}",txt),
            Self::CsvDataError(err) => write!(f,"{err}"),
            Self::InvalidColumn(txt) => write!(f,"ERR : Invalid column =\n{0}",txt),
            Self::InvalidPageOperation(txt) => write!(f,"ERR : Invalid page operation =\n{0}",txt),
            Self::InvalidRowData(txt) => write!(f,"ERR : Invalid row data =\n{0}",txt),
            Self::IoError(io_error) => write!(f,"ERR : IO Error =\n{0}",io_error),
            Self::OutOfRangeError => write!(f,"ERR : Index out of range"),
        }
    }
}

impl From<DcsvError> for CedError {
    fn from(err: DcsvError) -> Self {
        Self::CsvDataError(err)
    }
}

impl CedError {
    pub fn io_error(err: std::io::Error, meta: &str) -> Self {
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
        write!(f, "{} :: {}", self.error, self.meta)
    }
}

impl std::fmt::Display for IoErrorWithMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} :: {}", self.error, self.meta)
    }
}
