use crate::errors::app_error::AppError;
use actix_web::web;
use oauth2::{
    basic::{BasicClient, BasicTokenType},
    reqwest::async_http_client,
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, EmptyExtraTokenFields, PkceCodeVerifier,
    RedirectUrl, StandardTokenResponse, TokenUrl,
};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone)]
pub struct CognitoConfig {
    // .env should be removed from production
    pub auth_url: String,
    pub token_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub keyset_region: String,
    pub keyset_pool_id: String,
}

impl Default for CognitoConfig {
    fn default() -> Self {
        let cognito_domain = env::var("COGNITO_DOMAIN").expect("COGNITO_DOMAIN must be set");
        let auth_url = cognito_domain.clone() + "/oauth2/authorize";
        let token_url = cognito_domain + "/oauth2/token";
        let client_id = env::var("CLIENT_ID").expect("CLIENT_ID must be set");
        let client_secret = env::var("CLIENT_SECRET").expect("CLIENT_SECRET must be set");
        let keyset_pool_id = env::var("KEYSET_POOL_ID").expect("KEYSET_POOL_ID must be set");
        let keyset_region: String = keyset_pool_id.split("_").next().unwrap().into();

        Self {
            auth_url,
            token_url,
            client_id,
            client_secret,
            keyset_region,
            keyset_pool_id,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthTokenBody {
    grant_type: String,
    client_id: String,
    code: String,
    code_verifier: String,
    redirect_uri: String,
}

pub async fn pkce_code_verification(
    config: web::Data<CognitoConfig>,
    body: web::Form<AuthTokenBody>,
) -> Result<web::Json<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>>, AppError> {
    let req = body.into_inner();
    let config = config.into_inner();
    let client = BasicClient::new(
        ClientId::new(req.client_id.clone()),
        Some(ClientSecret::new(config.client_secret.clone())),
        AuthUrl::new(config.auth_url.clone()).map_err(AppError::Oauth2Parse)?,
        Some(TokenUrl::new(config.token_url.clone()).map_err(AppError::Oauth2Parse)?),
    )
    .set_redirect_uri(RedirectUrl::new(req.redirect_uri).map_err(AppError::Oauth2Parse)?);

    let pkce_verifier = PkceCodeVerifier::new(req.code_verifier);
    let token_result = client
        .exchange_code(AuthorizationCode::new(req.code))
        .set_pkce_verifier(pkce_verifier)
        .request_async(async_http_client)
        .await;

    match token_result {
        Ok(value) => Ok(web::Json(value)),
        Err(e) => Err(AppError::RequestToken(e)),
    }
}
