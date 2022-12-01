use crate::{errors::app_error::AppError, services::auth::CognitoConfig};
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    web, Error,
};
use jsonwebtokens_cognito::KeySet;
use std::{
    future::{ready, Future, Ready},
    pin::Pin,
    rc::Rc,
};

// Authorizes the incoming request (for /api endpoints) based on the JWT in Authentication header

pub struct Authorization;

impl<S: 'static, B> Transform<S, ServiceRequest> for Authorization
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Error = Error;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;
    type InitError = ();
    type Response = ServiceResponse<B>;
    type Transform = AuthMiddleware<S>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct AuthMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;
    type Response = ServiceResponse<B>;

    fn poll_ready(
        &self,
        ctx: &mut core::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();

        Box::pin(async move {
            let config = req
                .app_data::<web::Data<CognitoConfig>>()
                .ok_or(AppError::MissingConfig(
                    "Missing internal AWS Cognito configuration".into(),
                ))?
                .clone()
                .into_inner();
            let keyset = KeySet::new(config.keyset_region.clone(), config.keyset_pool_id.clone())
                .map_err(AppError::JWTCognito)?;
            let verifier = keyset
                .new_access_token_verifier(&[config.client_id.as_str()])
                .build()
                .map_err(AppError::JWTGeneric)?;
            let auth_header = req
                .headers()
                .get("Authorization")
                .ok_or(AppError::AuthNotFound(
                    "Authorization header missing".into(),
                ))?
                .to_str()
                .map_err(AppError::HeaderToStr)?
                .split(" ")
                .collect::<Vec<&str>>()[1]; // Strips prefix from the header

            keyset
                .verify(auth_header, &verifier)
                .await
                .map_err(AppError::JWTCognito)?;

            let fut = svc.call(req);
            let res = fut.await?;
            Ok(res)
        })
    }
}
