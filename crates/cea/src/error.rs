use cea_sys::{cea_err, cea_error_code_CEA_SUCCESS};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{context}: CEA error code {code}")]
    Cea { code: cea_err, context: String },
    #[error("{context}: invalid length")]
    InvalidLength { context: String },
    #[error("{context}: NUL byte in string")]
    Nul { context: String },
}

pub(crate) fn check(code: cea_err, context: &str) -> Result<()> {
    if code == cea_error_code_CEA_SUCCESS {
        Ok(())
    } else {
        Err(Error::Cea {
            code,
            context: context.to_string(),
        })
    }
}
