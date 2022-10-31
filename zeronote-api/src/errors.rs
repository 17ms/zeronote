use actix_web::{
    error::{self, InternalError, JsonPayloadError},
    http::StatusCode,
    HttpResponse, ResponseError,
};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug)]
pub enum AppErrorType {
    DieselResultError,
    DieselR2d2Error,
    ActixWebBlockingError,
    ValidationError,
    UnauthorizedError,
    //NotFoundError, (404)
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
        AppError::new(AppErrorType::DieselResultError)
    }
}

impl From<diesel::r2d2::PoolError> for AppError {
    fn from(_: diesel::r2d2::PoolError) -> Self {
        AppError::new(AppErrorType::DieselR2d2Error)
    }
}

impl From<actix_web::error::BlockingError> for AppError {
    fn from(_: actix_web::error::BlockingError) -> Self {
        AppError::new(AppErrorType::ActixWebBlockingError)
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(err: validator::ValidationErrors) -> Self {
        AppError::new(AppErrorType::ValidationError)
    }
}

impl From<uuid::Error> for AppError {
    fn from(_: uuid::Error) -> Self {
        AppError::new(AppErrorType::ValidationError)
    }
}

impl From<&actix_web::error::JsonPayloadError> for AppError {
    fn from(_: &actix_web::error::JsonPayloadError) -> Self {
        AppError::new(AppErrorType::ValidationError)
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
            AppErrorType::ValidationError => StatusCode::BAD_REQUEST,
            AppErrorType::UnauthorizedError => StatusCode::UNAUTHORIZED,
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code()).json(AppErrorResponse {
            code: self.code.clone(),
            message: self.message.clone(),
        })
    }
}

impl AppError {
    pub fn new(error_type: AppErrorType) -> Self {
        let (code, message, error) = match error_type {
            AppErrorType::ActixWebBlockingError => {
                ("500".into(), "Internal Server Error".into(), error_type)
            }
            AppErrorType::DieselR2d2Error => {
                ("500".into(), "Internal Server Error".into(), error_type)
            }
            AppErrorType::DieselResultError => {
                ("500".into(), "Internal Server Error".into(), error_type)
            }
            AppErrorType::ValidationError => {
                ("400".into(), "Invalid JSON payload".into(), error_type)
            }
            AppErrorType::UnauthorizedError => {
                ("401".into(), "Unauthorized request".into(), error_type)
            }
        };

        AppError {
            code,
            message,
            error,
        }
    }

    pub fn json_default_err_handler(err: JsonPayloadError) -> InternalError<JsonPayloadError> {
        let app_err = AppError::from(&err);
        error::InternalError::from_response(
            err,
            HttpResponse::build(app_err.status_code()).json(AppErrorResponse {
                code: app_err.code.clone(),
                message: app_err.message.clone(),
            }),
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppErrorResponse {
    pub code: String,
    pub message: String,
}
