use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use std::sync::Arc;
use crate::models::user::{LoginDTO, UserDTO};
use crate::models::response::ResponseBody;
use crate::services::user_service::UserService;
use crate::repositories::user_repository::PgUserRepository;
use crate::errors::service_error::ServiceError;

pub async fn get_users(pool: web::Data<PgPool>) -> Result<HttpResponse, ServiceError> {
    let service = UserService::<PgUserRepository>::new(Arc::new(pool.get_ref().clone()));
    
    let users = service.get_all().await?;
    Ok(HttpResponse::Ok().json(ResponseBody::new("Users retrieved successfully", Some(users))))
}

pub async fn get_user_by_id(
    pool: web::Data<PgPool>, 
    user_uid: web::Path<String>
) -> Result<HttpResponse, ServiceError> {
    let service = UserService::<PgUserRepository>::new(Arc::new(pool.get_ref().clone()));
    
    let user = service.get_by_id(&user_uid.into_inner()).await?;
    Ok(HttpResponse::Ok().json(ResponseBody::new("User retrieved successfully", Some(user))))
}

pub async fn signup(
    pool: web::Data<PgPool>, 
    user_dto:web::Json<UserDTO>,
) -> Result<HttpResponse, ServiceError> {
    let service = UserService::<PgUserRepository>::new(Arc::new(pool.get_ref().clone()));
    
    let message = service.signup(user_dto.0).await?;
    Ok(HttpResponse::Ok().json(ResponseBody::<()>::new(&message, None)))
}

pub async fn login(
    pool: web::Data<PgPool>,
    login_dto: web::Json<LoginDTO>,
) -> Result<HttpResponse, ServiceError> {
    let service = UserService::<PgUserRepository>::new(Arc::new(pool.get_ref().clone()));
    
    let user = service.login(login_dto.0).await?;

    Ok(HttpResponse::Ok().json(ResponseBody::new("User retrieved successfully", Some(user))))
}