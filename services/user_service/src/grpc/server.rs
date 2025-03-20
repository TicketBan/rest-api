// use std::sync::Arc;
// use tonic::{transport::Server, Request, Response, Status};
// use sqlx::PgPool;
// use user_service_grpc::user_service_grpc_server::{UserServiceGrpc, UserServiceGrpcServer};
// use user_service_grpc::{UserRequest, UserResponse};
// use crate::services::user_service::UserService;
// crate::repositories::user_repository::PgUserRepository

// mod user_service_grpc {
//     tonic::include_proto!("user_service_grpc");
// }

// pub struct UserGrpcService {
//     user_service: Arc<UserService<PgUserRepository>>,
// }

// impl UserGrpcService {
//     pub fn new(pool: Arc<PgPool>) -> Self {
//         Self {
//             user_service: Arc::new(UserService::new(pool))
//         }
//     }
// }

// #[tonic::async_trait]
// impl UserServiceGrpc for UserGrpcService {
//     async fn get_user_by_uid(&self, request: Request<UserRequest>) -> Result<Response<UserResponse>, Status> {
//         let uid = request.into_inner().user_uid;
        
//         let user = self.user_service.get_by_id(&uid).await
//             .map_err(|e| Status::not_found(format!("User not found: {}", e)))?;

//         let response = UserResponse {
//             uid: user.uid.to_string(),
//             username: user.username,
//             email: user.email,
//             created_at: user.created_at.timestamp(),
//             updated_at: user.updated_at.timestamp(),
//         };

//         Ok(Response::new(response))
//     }
// }

// pub async fn start_grpc_server(pool: Arc<PgPool>) -> Result<(), Box<dyn std::error::Error>> {
//     let addr = "[::1]:50052".parse()?;
//     let user_service = UserGrpcService::new(pool);

//     println!("Starting the gRPC user-service server on {}", addr);

//     Server::builder()
//         .add_service(UserServiceGrpcServer::new(user_service))
//         .serve(addr)
//         .await?;

//     Ok(())
// }