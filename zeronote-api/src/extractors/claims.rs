use crate::errors::AppError;
use actix_web::{http::Uri, FromRequest};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use awc::Client;
use jsonwebtoken::{
    decode, decode_header,
    jwk::{AlgorithmParameters, JwkSet},
    Algorithm, DecodingKey, Validation,
};
use serde::Deserialize;
use std::{env, future::Future, pin::Pin};

// Extract and validate JWT (+ claims)

#[derive(Debug, Deserialize, Clone)]
pub struct Auth0Config {
    audience: String,
    authority: String,
}

impl Default for Auth0Config {
    fn default() -> Self {
        let audience = env::var("AUTH0_AUDIENCE").expect("AUTH0_AUDIENCE must be set");
        let authority = env::var("AUTH0_AUTHORITY").expect("AUTH0_AUTHORITY must be set");

        Self {
            audience,
            authority,
        }
    }
}

// https://github.com/auth0-developer-hub/api_actix-web_rust_hello-world/blob/main/src/extractors/claims.rs#L94
#[derive(Debug, Deserialize)]
pub struct Claims {
    _sub: String, // TODO: use sub
}

impl FromRequest for Claims {
    type Error = AppError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    // TODO: remove unnecessary panics from the claims extractor below (before v0.1.0 & release)
    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_http::Payload,
    ) -> Self::Future {
        let auth_config = req.clone().app_data::<Auth0Config>().unwrap().clone();
        let extractor = BearerAuth::extract(req);

        Box::pin(async move {
            let credentials = extractor.await.map_err(AppError::WebAuthentication)?;
            let token = credentials.token();
            let header = decode_header(token).map_err(AppError::JWTDecode)?;
            let kid = header.kid.ok_or_else(|| {
                AppError::AuthNotFound("Key ID (kid) not found in token header".into())
            })?;
            let domain = auth_config.authority.as_str();
            let jwks: JwkSet = Client::new()
                .get(
                    Uri::builder()
                        .scheme("https")
                        .authority(domain)
                        .path_and_query("/.well-known/jwks.json")
                        .build()
                        .unwrap(),
                )
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();
            let jwk = jwks.find(&kid).ok_or_else(|| {
                AppError::AuthNotFound("Key ID (kid) doesn't match any JWK".into())
            })?;
            match jwk.clone().algorithm {
                AlgorithmParameters::RSA(ref rsa) => {
                    let mut validation = Validation::new(Algorithm::RS256);
                    validation.set_audience(&[auth_config.audience.as_str()]);
                    validation.set_issuer(&[Uri::builder()
                        .scheme("https")
                        .authority(domain)
                        .path_and_query("/")
                        .build()
                        .unwrap()]);
                    let key = DecodingKey::from_rsa_components(&rsa.n, &rsa.e)
                        .map_err(AppError::JWTDecode)?;
                    let token =
                        decode::<Claims>(token, &key, &validation).map_err(AppError::JWTDecode)?;
                    Ok(token.claims)
                }
                algorithm => Err(AppError::JWTUnsupportedAlgorithm(algorithm)),
            }
        })
    }
}
