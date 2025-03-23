use std::sync::Arc;
use tonic::{transport::Server, Request, Response, Status};
use shared::user_service_grpc::user_service_grpc_server::{UserServiceGrpc, UserServiceGrpcServer};
use shared::user_service_grpc::{UserRequest, UserResponse};
use crate::services::user_service::UserService;
use crate::repositories::user_repository::PgUserRepository;
use log::{info, error};
use prost_types::Timestamp;
use chrono::{DateTime, Utc};

pub struct UserGrpcService {
    user_service: Arc<UserService<PgUserRepository>>,
}

impl UserGrpcService {
    pub fn new(user_service: Arc<UserService<PgUserRepository>>) -> Self {
        Self {
            user_service: user_service,
        }
    }
}

#[tonic::async_trait]
impl UserServiceGrpc for UserGrpcService {
    async fn get_user_by_uid(&self, request: Request<UserRequest>) -> Result<Response<UserResponse>, Status> {
        let uid = request.into_inner().uid;
        info!("gRPC request: get_user_by_uid {}", uid);
        
        let user = self.user_service.get_by_id(&uid).await
            .map_err(|e| {
                error!("gRPC error: {}", e);
                Status::not_found(format!("User not found: {}", e))
            })?;

        let response = UserResponse {
            uid: user.uid.to_string(),
            username: user.username,
            email: user.email,
            created_at: convert_to_prost_timestamp(user.created_at),
            updated_at: convert_to_prost_timestamp(user.updated_at),
        };

        Ok(Response::new(response))
    }
}

pub async fn start_grpc_server(addr: std::net::SocketAddr, user_service: Arc<UserService<PgUserRepository>>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Starting gRPC server on {}", addr);
    let user_service = UserGrpcService::new(user_service);

    Server::builder()
        .add_service(UserServiceGrpcServer::new(user_service))
        .serve(addr)
        .await?;

    Ok(())
}

fn convert_to_prost_timestamp(dt: DateTime<Utc>) -> Option<Timestamp> {
    Some(Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    })
}