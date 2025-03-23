use actix_web::{web, HttpResponse};
use crate::models::user::{LoginDTO, UserDTO};
use crate::models::response::ResponseBody;
use crate::services::user_service::UserService;
use crate::repositories::user_repository::PgUserRepository;
use crate::errors::service_error::ServiceError;

pub async fn get_users(
    service: web::Data<UserService<PgUserRepository>>,
) -> Result<HttpResponse, ServiceError> {
    let users = service.get_all().await?;
    Ok(HttpResponse::Ok().json(ResponseBody::new("Users retrieved successfully", Some(users))))
}

pub async fn get_user_by_id(
    service: web::Data<UserService<PgUserRepository>>, 
    user_uid: web::Path<String>,
) -> Result<HttpResponse, ServiceError> {
    let user = service.get_by_id(&user_uid.into_inner()).await?;
    Ok(HttpResponse::Ok().json(ResponseBody::new("User retrieved successfully", Some(user))))
}

pub async fn signup(
    service: web::Data<UserService<PgUserRepository>>, 
    user_dto: web::Json<UserDTO>,
) -> Result<HttpResponse, ServiceError> {
    let user = service.signup(user_dto.0).await?;
    Ok(HttpResponse::Ok().json(ResponseBody::new("User created successfully", Some(user))))
}

pub async fn login(
    service: web::Data<UserService<PgUserRepository>>,
    login_dto: web::Json<LoginDTO>,
) -> Result<HttpResponse, ServiceError> {
    let response = service.login(login_dto.0).await?;
    Ok(HttpResponse::Ok().json(ResponseBody::new("User logged in successfully", Some(response))))
}