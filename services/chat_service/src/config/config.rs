use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use std::fmt;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: Arc<String>,
    pub http_addr: SocketAddr,
    pub grpc_addr: String,
    pub log_level: log::LevelFilter,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let database_url = env::var("DATABASE_URL_CHAT_SERVICE")
            .map_err(|_| "DATABASE_URL_CHAT_SERVICE must be set")?;
        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| "JWT_SECRET must be set")?;
        let host = env::var("CHAT_SERVICE_HOST")
            .map_err(|_| "CHAT_SERVICE_HOST must be set")?;
        let port = env::var("CHAT_SERVICE_PORT")
            .map_err(|_| "CHAT_SERVICE_PORT must be set")?
            .parse::<u16>()
            .map_err(|_| "CHAT_SERVICE_PORT must be a valid port number")?;
        let grpc_port = env::var("USER_SERVICE_GRPC_PORT")
            .unwrap_or("50052".into())
            .parse::<u16>()
            .map_err(|_| "USER_SERVICE_GRPC_PORT must be a valid port number")?;
        let log_level = env::var("LOG_LEVEL")
            .unwrap_or("info".into())
            .parse::<log::LevelFilter>()
            .map_err(|_| "LOG_LEVEL must be a valid log level (e.g., info, debug)")?;

        Ok(Self {
            database_url,
            jwt_secret: Arc::new(jwt_secret),
            http_addr: format!("{}:{}", host, port).parse().map_err(|e| format!("Invalid HTTP address: {}", e))?,
            grpc_addr: format!("http://[::1]:{}", grpc_port).parse().map_err(|e| format!("Invalid gRPC address: {}", e))?,
            log_level,
        })
    }
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("database_url", &self.database_url)
            .field("http_addr", &self.http_addr)
            .field("grpc_addr", &self.grpc_addr)
            .field("log_level", &self.log_level)
            .finish() 
    }
}

