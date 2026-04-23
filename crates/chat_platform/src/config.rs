use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub security: SecurityConfig,
    pub storage: StorageConfig,
    pub logging: LoggingConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub token_expiry_secs: u64,
    pub max_file_size_mb: u64,
    pub rate_limit_per_second: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StorageConfig {
    pub file_storage_path: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LoggingConfig {
    pub dir: String,
    pub file_prefix: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv::dotenv().ok();

        let server = ServerConfig {
            host: std::env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: std::env::var("SERVER_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            workers: std::env::var("SERVER_WORKERS")
                .ok()
                .and_then(|w| w.parse().ok())
                .unwrap_or(4),
        };

        let database = DatabaseConfig {
            url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite://chat_platform.db".to_string()),
            max_connections: std::env::var("DB_MAX_CONNECTIONS")
                .ok()
                .and_then(|c| c.parse().ok())
                .unwrap_or(10),
            min_connections: std::env::var("DB_MIN_CONNECTIONS")
                .ok()
                .and_then(|c| c.parse().ok())
                .unwrap_or(2),
        };

        let security = SecurityConfig {
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "your-secret-key".to_string()),
            token_expiry_secs: std::env::var("TOKEN_EXPIRY_SECS")
                .ok()
                .and_then(|t| t.parse().ok())
                .unwrap_or(86400), // 24 hours
            max_file_size_mb: std::env::var("MAX_FILE_SIZE_MB")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(100),
            rate_limit_per_second: std::env::var("RATE_LIMIT_PER_SECOND")
                .ok()
                .and_then(|r| r.parse().ok())
                .unwrap_or(10),
        };

        let storage = StorageConfig {
            file_storage_path: std::env::var("FILE_STORAGE_PATH")
                .unwrap_or_else(|_| "./assets/files/".to_string()),
        };

        let logging = LoggingConfig {
            dir: std::env::var("LOG_DIR").unwrap_or_else(|_| "./logs".to_string()),
            file_prefix: std::env::var("LOG_FILE_PREFIX")
                .unwrap_or_else(|_| "chat_platform".to_string()),
        };

        Config {
            server,
            database,
            security,
            storage,
            logging,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                workers: 4,
            },
            database: DatabaseConfig {
                url: "sqlite://chat_platform.db".to_string(),
                max_connections: 10,
                min_connections: 2,
            },
            security: SecurityConfig {
                jwt_secret: "your-secret-key".to_string(),
                token_expiry_secs: 86400,
                max_file_size_mb: 100,
                rate_limit_per_second: 10,
            },
            storage: StorageConfig {
                file_storage_path: "./assets/files/".to_string(),
            },
            logging: LoggingConfig {
                dir: "./logs".to_string(),
                file_prefix: "chat_platform".to_string(),
            },
        }
    }
}
