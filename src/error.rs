use std::{error::Error, fmt::Display};

use reqwest::{header::InvalidHeaderValue, StatusCode};

#[derive(Debug)]
pub enum JellyfinError {
    AuthorizationError,
    NotFound,
    ParseError,
    ServerError,
    BadRequest,
    RequestError(reqwest::Error),
    UnhandledError(String),
}

impl Display for JellyfinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            JellyfinError::AuthorizationError => "Authorization error",
            JellyfinError::NotFound => "Not found",
            JellyfinError::ParseError => "Parse error",
            JellyfinError::ServerError => "Server error",
            JellyfinError::BadRequest => "Bad request",
            JellyfinError::RequestError(e) => "Request error",
            JellyfinError::UnhandledError(e) => e.as_str(),
        })
    }
}

impl Error for JellyfinError {}

impl From<StatusCode> for JellyfinError {
    fn from(code: StatusCode) -> Self {
        match code {
            StatusCode::UNAUTHORIZED => Self::AuthorizationError,
            StatusCode::NOT_FOUND => Self::NotFound,
            StatusCode::INTERNAL_SERVER_ERROR => Self::ServerError,
            StatusCode::BAD_REQUEST => Self::BadRequest,
            code => Self::UnhandledError(code.to_string()),
        }
    }
}

impl From<InvalidHeaderValue> for JellyfinError {
    fn from(_: InvalidHeaderValue) -> Self {
        Self::ParseError
    }
}

impl From<url::ParseError> for JellyfinError {
    fn from(_: url::ParseError) -> Self {
        Self::ParseError
    }
}

impl From<reqwest::Error> for JellyfinError {
    fn from(e: reqwest::Error) -> Self {
        Self::RequestError(e)
    }
}

impl From<serde_json::Error> for JellyfinError {
    fn from(_: serde_json::Error) -> Self {
        Self::ParseError
    }
}
