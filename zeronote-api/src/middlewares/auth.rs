use crate::extractors::token::CognitoConfig;
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

// Verifies the JWT found in the req auth header

pub struct Authentication;

impl<S: 'static, B> Transform<S, ServiceRequest> for Authentication
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Error = Error;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;
    type InitError = ();
    type Response = ServiceResponse<B>;
    type Transform = AuthenticationMiddleware<S>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthenticationMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct AuthenticationMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthenticationMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;
    type Response = ServiceResponse<B>;

    fn poll_ready(
        &self,
        ctx: &mut core::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    // TODO: remove panics
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();

        Box::pin(async move {
            let config = req
                .app_data::<web::Data<CognitoConfig>>()
                .unwrap()
                .clone()
                .into_inner();
            let keyset =
                KeySet::new(config.keyset_region.clone(), config.keyset_pool_id.clone()).unwrap();
            let verifier = keyset
                .new_access_token_verifier(&[config.client_id.as_str()])
                .build()
                .unwrap();
            let auth_header = req
                .headers()
                .get("Authorization")
                .expect("No Authorization header found, TODO: error handling")
                .to_str()
                .unwrap();

            let _verified = keyset.verify(auth_header, &verifier).await.unwrap();

            let fut = svc.call(req);
            let res = fut.await.unwrap();
            Ok(res)
        })
    }
}
