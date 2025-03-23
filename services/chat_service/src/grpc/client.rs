use tonic::transport::{Channel, Uri};
use shared::user_service_grpc::{UserResponse, UserRequest};
use shared::user_service_grpc::user_service_grpc_client::UserServiceGrpcClient;
use crate::errors::service_error::ServiceError;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct GrpcClientConfig {
    url: String,
    timeout: std::time::Duration,

}

#[derive(Clone)]
pub struct UserGrpcClient {
    inner: UserServiceGrpcClient<Channel>,
}

impl UserGrpcClient {
    pub async fn new(config: GrpcClientConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let uri = Uri::try_from(config.url.clone())
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        
        let channel = Channel::builder(uri)
            .timeout(config.timeout)
            .connect()
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

        let client = UserServiceGrpcClient::new(channel);

        Ok(Self { inner: client })
    }

    pub async fn get_user_by_uid(&self, user_uid: Uuid) -> Result<UserResponse, ServiceError> {
        let request = tonic::Request::new(UserRequest {
            uid: user_uid.to_string(),
        });

        self.inner
            .clone()
            .get_user_by_uid(request)
            .await
            .map_err(|e| {
                log::error!("Failed to get user by UID {}: {:?}", user_uid, e);
                match e.code() {
                    tonic::Code::NotFound => ServiceError::not_found(&format!("User {} not found", user_uid)),
                    tonic::Code::Unavailable => ServiceError::internal_error("gRPC server unavailable"),
                    _ => ServiceError::internal_error(&format!("gRPC error: {}", e)),
                }
            })
            .map(|resp| resp.into_inner())
    }
}

pub async fn init_grpc_client(url: String, timeout: std::time::Duration ) -> Result<Arc<UserGrpcClient>, ServiceError> {
    let config = GrpcClientConfig { url, timeout };
    let client = UserGrpcClient::new(config)
        .await
        .map_err(|e| ServiceError::internal_error(&format!("Failed to init gRPC client: {}", e)))?;
    Ok(Arc::new(client))
}