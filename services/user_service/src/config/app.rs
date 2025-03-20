use crate::controllers::*;
use actix_web::web;

pub fn config_services(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(
                web::scope("/users")
                    .route("", web::get().to(user_controller::get_users))
                    .route("/{uid}", web::get().to(user_controller::get_user_by_id))
                    .service(
                        web::scope("/auth") 
                            .route("/signup", web::post().to(user_controller::signup))
                            .route("/login", web::post().to(user_controller::login))
                    )
            )
    );
}
