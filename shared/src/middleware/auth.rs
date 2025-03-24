use crate::models::user_token::UserToken;
use actix_web::{error::ErrorUnauthorized ,Error};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use futures_util::future::{LocalBoxFuture, ok};
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::Arc;
pub struct AuthenticationMiddleware<S> {
    service: Rc<S>,
    secret: Arc<String>,
    excluded_paths: HashSet<&'static str>,
}

impl<S, B> Service<ServiceRequest> for AuthenticationMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        if self.excluded_paths.contains(req.path()) {
            return Box::pin(self.service.call(req));
        }
    
        let auth_header = req.headers().get("Authorization");
        if let Some(auth_header) = auth_header {
            if let Ok(auth_str) = auth_header.to_str() {
                if auth_str.starts_with("Bearer ") {
                    
                    let token = &auth_str[7..];
                    match UserToken::validate_token(token, &self.secret) {
                        Ok(user_token) if user_token.is_valid() => {
                            let fut = self.service.call(req);
                            return Box::pin(async move {
                                match fut.await {
                                    Ok(res) => Ok(res),
                                    Err(err) => Err(err),
                                }
                            });
                        }
                        Ok(_) => {
                            return Box::pin(async {
                                Err(ErrorUnauthorized("Token expired"))
                            });
                        }
                        Err(_e) => {
                            return Box::pin(async {
                                Err(ErrorUnauthorized("Invalid token"))
                            });
                        }
                    }
                }
            }
        }
    
        Box::pin(async {
            Err(ErrorUnauthorized("Missing or invalid Authorization header"))
        })
    }
}

pub struct Authentication {
    secret: Arc<String>,
    excluded_paths: HashSet<&'static str>,
}


impl Authentication {
    pub fn new(secret: Arc<String>, excluded_paths: Option<HashSet<&'static str>>) -> Self {
        Self {
            secret,
            excluded_paths: excluded_paths.unwrap_or_else(HashSet::new),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for Authentication
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthenticationMiddleware<S>;
    type Future = futures_util::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthenticationMiddleware {
            service: Rc::new(service),
            secret: self.secret.clone(),
            excluded_paths: self.excluded_paths.clone(),
        })
    }
}
