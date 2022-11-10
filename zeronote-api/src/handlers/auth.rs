use crate::{errors::AppError, extractors::token::*};
use actix_web::{post, web};
use oauth2::{basic::BasicTokenType, EmptyExtraTokenFields, StandardTokenResponse};

// Handler for Cognito's /oauth2/token request

// TODO: remove panics
#[post("/token")]
pub async fn get_token(
    config: web::Data<CognitoConfig>,
    body: web::Form<AuthTokenBody>,
) -> web::Json<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>> {
    let token_json = web::block(move || verify_token_req_code(config, body))
        .await
        .map_err(AppError::WebBlocking)
        .unwrap()
        .await
        .unwrap();

    token_json
}
