use actix_web::body::EitherBody;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{Error, HttpResponse};
use futures_util::future::{ok, LocalBoxFuture, Ready, FutureExt};
use std::rc::Rc;
use log::{error, info};
use crate::models::user_token::UserToken;
use std::task::{Context, Poll};
use std::env;

pub struct AuthenticationMiddleware<S> {
    service: Rc<S>,
    secret: String,
}

impl<S, B> Service<ServiceRequest> for AuthenticationMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let secret = self.secret.clone(); 
    
        let excluded_paths = vec!["/api/users/auth", "/api/users/auth"];

        if excluded_paths.iter().any(|&path| req.path().starts_with(path)) {
            return Box::pin(async move {
                let res = service.call(req).await?;
                Ok(res.map_into_left_body())
            });
        }
    

        let auth_header = req.headers().get("Authorization");
    
        if let Some(auth_header) = auth_header {
            if let Ok(auth_str) = auth_header.to_str() {
                if auth_str.starts_with("Bearer ") {
                    let token = &auth_str[7..];
                    match UserToken::validate_token(token, &secret) {
                        Ok(user_token) if user_token.is_valid() => {
                            let fut = self.service.call(req);
                            return Box::pin(async move {
                                match fut.await {
                                    Ok(res) => Ok(res.map_into_left_body()),
                                    Err(err) => Err(err),
                                }
                            });
                        }
                        Ok(_) => error!("Token expired"),
                        Err(e) => error!("Invalid token"),
                    }
                }
            }
        }
    
        let (req, _pl) = req.into_parts();
        let response = HttpResponse::Unauthorized().finish().map_into_right_body();
        Box::pin(async { Ok(ServiceResponse::new(req, response)) })
    }
}

pub struct Authentication{
    secret: String
}

impl Authentication {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }
}

impl<S, B> Transform<S, ServiceRequest> for Authentication
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthenticationMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthenticationMiddleware {
            service: Rc::new(service),
            secret: self.secret.clone(),
        })
    }
}
