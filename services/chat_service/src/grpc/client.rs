use tonic::Status;
use tonic::transport::Channel;
use shared::user_service_grpc::{UserResponse, UserRequest};
use shared::user_service_grpc::user_service_grpc_client::UserServiceGrpcClient;
use crate::errors::service_error::ServiceError;
use log;

pub async fn connect_to_grpc_server() -> Result<UserServiceGrpcClient<Channel>, Status> {
    let url = "http://[::1]:50052"; 

    match UserServiceGrpcClient::connect(url).await {
        Ok(client) => Ok(client),
        Err(e) => {
            eprintln!("Failed to connect to gRPC server: {:?}", e);
            Err(Status::unavailable("Failed to connect to the gRPC server"))
        }
    }
}
pub async fn get_user_by_uid(user_uid: String) -> Result<UserResponse, ServiceError> {
    let mut client = match connect_to_grpc_server().await {
        Ok(client) => client,
        Err(e) => {
            log::error!("gRPC connection failed: {:?}", e);
            return Err(ServiceError::internal_error("Failed to connect to gRPC server"));
        },
    };

    let request = UserRequest { user_uid };
    let grpc_request = tonic::Request::new(request);

    let response = client.get_user_by_uid(grpc_request).await.map_err(|e| {
        log::error!("gRPC error: {:?}", e);
        ServiceError::internal_error("Failed to get user by UID")
    })?;

    Ok(response.into_inner())
}
