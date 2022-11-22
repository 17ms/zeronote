use crate::{errors::app_error::AppError, services::auth::*};
use actix_web::{post, web};
use oauth2::{basic::BasicTokenType, EmptyExtraTokenFields, StandardTokenResponse};

// Verifies PKCE code & fetches access and refresh tokens from AWS

#[post("/token")]
pub async fn fetch_jwt(
    config: web::Data<CognitoConfig>,
    body: web::Form<AuthTokenBody>,
) -> Result<web::Json<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>>, AppError> {
    let token_json = web::block(move || pkce_code_verification(config, body))
        .await
        .map_err(AppError::WebBlocking)?
        .await?;

    Ok(token_json)
}
