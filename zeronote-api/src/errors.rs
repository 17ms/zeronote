use actix_http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use actix_web_httpauth::headers::www_authenticate::bearer::Bearer;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

// Wrapper for general errors to make them readable and returnable as responses

#[derive(Debug)]
pub enum AppError {
    DieselResult(diesel::result::Error),
    DieselPool(diesel::r2d2::PoolError),
    WebBlocking(actix_web::error::BlockingError),
    Validator(validator::ValidationErrors),
    Uuid(uuid::Error),
    JsonPayLoad(actix_web::error::JsonPayloadError),
    WebAuthentication(actix_web_httpauth::extractors::AuthenticationError<Bearer>),
    JWTDecode(jsonwebtoken::errors::Error),
    JWTUnsupportedAlgorithm(jsonwebtoken::jwk::AlgorithmParameters),
    AuthNotFound(String),
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> actix_http::StatusCode {
        match *self {
            Self::DieselResult(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::DieselPool(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::WebBlocking(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Validator(_) => StatusCode::BAD_REQUEST,
            Self::Uuid(_) => StatusCode::BAD_REQUEST,
            Self::JsonPayLoad(_) => StatusCode::BAD_REQUEST,
            Self::WebAuthentication(_) => StatusCode::UNAUTHORIZED,
            Self::JWTDecode(_) => StatusCode::UNAUTHORIZED,
            Self::JWTUnsupportedAlgorithm(_) => StatusCode::UNAUTHORIZED,
            Self::AuthNotFound(_) => StatusCode::UNAUTHORIZED,
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_http::body::BoxBody> {
        HttpResponse::build(self.status_code()).json(AppErrorResponse::new(self))
    }
}

impl From<diesel::result::Error> for AppError {
    fn from(_: diesel::result::Error) -> Self {
        AppError::AuthNotFound("nice".to_string())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppErrorResponse {
    pub code: String,
    pub message: String,
}

impl AppErrorResponse {
    fn new(app_error: &AppError) -> Self {
        let (code, message) = match app_error {
            AppError::DieselResult(_) => ("500".into(), "Internal Server Error".into()),
            AppError::DieselPool(_) => ("500".into(), "Internal Server Error".into()),
            AppError::WebBlocking(_) => ("500".into(), "Internal Server Error".into()),
            AppError::Validator(_) => ("400".into(), "Invalid JSON payload".into()),
            AppError::Uuid(_) => ("400".into(), "Invalid UUID".into()),
            AppError::JsonPayLoad(_) => ("400".into(), "Invalid JSON payload".into()),
            AppError::WebAuthentication(_) => {
                ("401".into(), "Resource requires authentication".into())
            }
            AppError::JWTDecode(_) => (
                "401".into(),
                "Authorization header must follow format 'Bearer <access-token>'".into(),
            ),
            AppError::JWTUnsupportedAlgorithm(algorithm) => (
                "401".into(),
                format!("Unsupported encryption algorithm, found '{:?}'", algorithm),
            ),
            AppError::AuthNotFound(s) => ("401".into(), s.into()),
        };

        AppErrorResponse { code, message }
    }
}
