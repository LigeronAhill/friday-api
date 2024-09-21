use std::{error::Error, fmt::Display};

use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum AppError {
    DbError(String),
    ReqwestError(String),
    MailError(String),
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
        Self::DbError(value.to_string())
    }
}
impl From<reqwest::Error> for AppError {
    fn from(value: reqwest::Error) -> Self {
        Self::ReqwestError(value.to_string())
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
impl From<imap::error::Error> for AppError {
    fn from(value: imap::error::Error) -> Self {
        Self::MailError(value.to_string())
    }
}
