use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt::Display;

#[derive(Debug)]
pub enum AppErrorType {
    DieselResultError,
    DieselR2d2Error,
    ActixWebBlockingError,
    //NotFoundError, (404)
    //ValidationError, (400)
    //UnauthorizedError, (401)
    //TooManyRequestsError, (429)
}

#[derive(Debug)]
pub struct AppError {
    pub code: String,
    pub message: String,
    pub error: AppErrorType,
}

impl From<diesel::result::Error> for AppError {
    fn from(_: diesel::result::Error) -> Self {
        AppError {
            code: "500".to_string(),
            message: "Internal Server Error".to_string(),
            error: AppErrorType::DieselResultError,
        }
    }
}

impl From<diesel::r2d2::PoolError> for AppError {
    fn from(_: diesel::r2d2::PoolError) -> Self {
        AppError {
            code: "500".to_string(),
            message: "Internal Server Error".to_string(),
            error: AppErrorType::DieselR2d2Error,
        }
    }
}

impl From<actix_web::error::BlockingError> for AppError {
    fn from(_: actix_web::error::BlockingError) -> Self {
        AppError {
            code: "500".to_string(),
            message: "Internal Server Error".to_string(),
            error: AppErrorType::ActixWebBlockingError,
        }
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self.error {
            AppErrorType::DieselResultError => StatusCode::INTERNAL_SERVER_ERROR,
            AppErrorType::DieselR2d2Error => StatusCode::INTERNAL_SERVER_ERROR,
            AppErrorType::ActixWebBlockingError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code()).json(AppErrorResponse {
            code: self.code.as_str(),
            message: self.message.as_str(),
        })
    }
}

#[derive(Debug, Serialize)]
pub struct AppErrorResponse<'a> {
    pub code: &'a str,
    pub message: &'a str,
}

// TODO: Unit tests
