use std::sync::Arc;
use tonic::{transport::Server, Request, Response, Status};
use sqlx::PgPool;
use shared::user_service_grpc::user_service_grpc_server::{UserServiceGrpc, UserServiceGrpcServer};
use shared::user_service_grpc::{UserRequest, UserResponse};
use crate::services::user_service::UserService;
use crate::repositories::user_repository::PgUserRepository;


pub struct UserGrpcService {
    user_service: Arc<UserService<PgUserRepository>>,
}

impl UserGrpcService {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self {
            user_service: Arc::new(UserService::new(pool))
        
        }
    }
}

#[tonic::async_trait]
impl UserServiceGrpc for UserGrpcService {
    async fn get_user_by_uid(&self, request: Request<UserRequest>) -> Result<Response<UserResponse>, Status> {
        let uid = request.into_inner().user_uid;
        
        let user = self.user_service.get_by_id(&uid).await
            .map_err(|e| Status::not_found(format!("User not found: {}", e)))?;

        let response = UserResponse {
            uid: user.uid.to_string(),
            username: user.username,
            email: user.email,
            created_at: user.created_at.timestamp(),
            updated_at: user.updated_at.timestamp(),
        };

        Ok(Response::new(response))
    }
}

pub async fn start_grpc_server(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let addr =  std::env::var("GRPC_USER_SERVICE_URL").parse()?;
    let user_service = UserGrpcService::new(Arc::new(pool));

    println!("Starting the gRPC user-service server on {}", addr);

    Server::builder()
        .add_service(UserServiceGrpcServer::new(user_service))
        .serve(addr)
        .await?;

    Ok(())
}