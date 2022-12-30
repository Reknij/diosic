use std::fmt::{self, Display};

use actix_web::{http::StatusCode, ResponseError};

#[derive(Debug)]
pub enum APIErrorType {
    NoPermission,
    NoFound,
    Unspecified,
    NoFoundUser,
    Unexpected,
}

#[derive(Debug)]
pub struct APIError {
    error_type: APIErrorType,
    note: Option<String>,
}

impl APIError {
    pub fn with(error_type: APIErrorType) -> Self {
        Self {
            error_type,
            note: None,
        }
    }
    pub fn note<T: Into<String>>(mut self, note: T)-> Self {
        self.note = Some(note.into());
        self
    }
}

impl Display for APIErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            APIErrorType::NoPermission => write!(f, "NoPermission"),
            APIErrorType::NoFound => write!(f, "NoFound"),
            APIErrorType::Unspecified => write!(f, "Unspecified"),
            APIErrorType::NoFoundUser => write!(f, "NoFoundUser"),
            APIErrorType::Unexpected => write!(f, "Unexpected"),
        }
    }
}

impl Display for APIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.note {
            Some(note) => match self.error_type {
                APIErrorType::NoPermission => write!(f, "NoPermission: {}", note),
                APIErrorType::NoFound => write!(f, "NoFound: {}", note),
                APIErrorType::Unspecified => write!(f, "Unspecified: {}", note),
                APIErrorType::NoFoundUser => write!(f, "NoFoundUser: {}", note),
                APIErrorType::Unexpected => write!(f, "Unexpected: {}", note),
            },
            None=> {
                write!(f, "{}", self.error_type)
            }
        }
    }
}

impl ResponseError for APIError {
    fn status_code(&self) -> StatusCode {
        match self.error_type {
            APIErrorType::NoFound => StatusCode::NOT_FOUND,
            APIErrorType::NoPermission => StatusCode::NOT_ACCEPTABLE,
            APIErrorType::Unspecified => StatusCode::INTERNAL_SERVER_ERROR,
            APIErrorType::NoFoundUser => StatusCode::NOT_FOUND,
            APIErrorType::Unexpected => StatusCode::EXPECTATION_FAILED,
        }
    }
}
