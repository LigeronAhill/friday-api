use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum AppError {
    DbError(mongodb::error::Error),
    ReqwestError(reqwest::Error),
}

pub type Result<T> = core::result::Result<T, AppError>;
impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Error for AppError {}
impl From<mongodb::error::Error> for AppError {
    fn from(value: mongodb::error::Error) -> Self {
        Self::DbError(value)
    }
}
impl From<reqwest::Error> for AppError {
    fn from(value: reqwest::Error) -> Self {
        Self::ReqwestError(value)
    }
}
impl From<AppError> for shuttle_runtime::Error {
    fn from(value: AppError) -> Self {
        match value {
            AppError::DbError(e) => shuttle_runtime::Error::Database(e.to_string()),
            _ => shuttle_runtime::Error::Custom(value.into()),
        }
    }
}
