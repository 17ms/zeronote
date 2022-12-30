use actix_http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use log::debug;
use oauth2::{basic::BasicErrorResponseType, RequestTokenError, StandardErrorResponse};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

// Wrapper for generic backend errors to convert them to readable & returnable responses

#[derive(Debug)]
pub enum AppError {
    DieselResult(diesel::result::Error),
    DieselPool(diesel::r2d2::PoolError),
    WebBlocking(actix_web::error::BlockingError),
    Validator(validator::ValidationErrors),
    Uuid(uuid::Error),
    JsonPayLoad(actix_web::error::JsonPayloadError),
    OAuth2Token(
        RequestTokenError<
            oauth2::reqwest::Error<reqwest::Error>,
            StandardErrorResponse<BasicErrorResponseType>,
        >,
    ),
    Oauth2Parse(oauth2::url::ParseError),
    JwtGeneric(jsonwebtokens::error::Error),
    JwtCognito(jsonwebtokens_cognito::Error),
    JwtParse(jwt::Error),
    JwtMissingClaim(String),
    HeaderToStr(actix_http::header::ToStrError),
    MissingConfig(String),
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
            Self::MissingConfig(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Validator(_) => StatusCode::BAD_REQUEST,
            Self::Uuid(_) => StatusCode::BAD_REQUEST,
            Self::JsonPayLoad(_) => StatusCode::BAD_REQUEST,
            Self::OAuth2Token(_) => StatusCode::UNAUTHORIZED,
            Self::Oauth2Parse(_) => StatusCode::UNAUTHORIZED,
            Self::JwtGeneric(_) => StatusCode::UNAUTHORIZED,
            Self::JwtCognito(_) => StatusCode::UNAUTHORIZED,
            Self::JwtParse(_) => StatusCode::UNAUTHORIZED,
            Self::JwtMissingClaim(_) => StatusCode::UNAUTHORIZED,
            Self::HeaderToStr(_) => StatusCode::UNAUTHORIZED,
            Self::AuthNotFound(_) => StatusCode::UNAUTHORIZED,
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_http::body::BoxBody> {
        debug!(target: "errors_file", "{}", self);
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
            AppError::MissingConfig(s) => ("500".into(), s.into()),
            AppError::Validator(_) => ("400".into(), "Invalid JSON payload".into()),
            AppError::Uuid(_) => ("400".into(), "Invalid UUID".into()),
            AppError::JsonPayLoad(_) => ("400".into(), "Invalid JSON payload".into()),
            AppError::OAuth2Token(_) => ("401".into(), "Invalid JWT token".into()),
            AppError::Oauth2Parse(_) => ("401".into(), "Invalid JWT token".into()),
            AppError::JwtGeneric(_) => ("401".into(), "Invalid JWT token".into()),
            AppError::JwtCognito(_) => ("401".into(), "Invalid JWT token".into()),
            AppError::JwtParse(_) => ("401".into(), "Invalid JWT token".into()),
            AppError::JwtMissingClaim(_) => ("401".into(), "Invalid JWT token".into()),
            AppError::HeaderToStr(_) => ("401".into(), "Invalid JWT token".into()),
            AppError::AuthNotFound(s) => ("401".into(), s.into()),
        };

        AppErrorResponse { code, message }
    }
}
