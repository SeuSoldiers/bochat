use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
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
pub struct RedisConfig {
    pub url: String,
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
            url: std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://chat_user:chat_pass@localhost:5432/chat_platform".to_string()
            }),
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

        let redis = RedisConfig {
            url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
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
            redis,
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
                url: "postgres://chat_user:chat_pass@localhost:5432/chat_platform".to_string(),
                max_connections: 10,
                min_connections: 2,
            },
            security: SecurityConfig {
                jwt_secret: "your-secret-key".to_string(),
                token_expiry_secs: 86400,
                max_file_size_mb: 100,
                rate_limit_per_second: 10,
            },
            redis: RedisConfig {
                url: "redis://localhost:6379".to_string(),
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

#[cfg(test)]
mod tests {
    use super::Config;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn clear_test_env() {
        unsafe {
            std::env::remove_var("SERVER_HOST");
            std::env::remove_var("SERVER_PORT");
            std::env::remove_var("SERVER_WORKERS");
            std::env::remove_var("DATABASE_URL");
            std::env::remove_var("DB_MAX_CONNECTIONS");
            std::env::remove_var("DB_MIN_CONNECTIONS");
            std::env::remove_var("JWT_SECRET");
            std::env::remove_var("TOKEN_EXPIRY_SECS");
            std::env::remove_var("MAX_FILE_SIZE_MB");
            std::env::remove_var("RATE_LIMIT_PER_SECOND");
            std::env::remove_var("REDIS_URL");
            std::env::remove_var("FILE_STORAGE_PATH");
            std::env::remove_var("LOG_DIR");
            std::env::remove_var("LOG_FILE_PREFIX");
        }
    }

    #[test]
    fn default_config_has_expected_values() {
        let _guard = env_lock().lock().expect("env lock");
        let cfg = Config::default();
        assert_eq!(cfg.server.host, "127.0.0.1");
        assert_eq!(cfg.server.port, 8080);
        assert_eq!(cfg.database.max_connections, 10);
        assert_eq!(cfg.redis.url, "redis://localhost:6379");
        assert_eq!(cfg.storage.file_storage_path, "./assets/files/");
    }

    #[test]
    fn from_env_uses_defaults_on_missing_or_invalid_values() {
        let _guard = env_lock().lock().expect("env lock");
        clear_test_env();
        let cfg = Config::from_env();
        assert_eq!(cfg.server.port, 8080);
        assert_eq!(cfg.server.workers, 4);
        assert_eq!(cfg.database.min_connections, 2);
        assert_eq!(cfg.security.token_expiry_secs, 86400);
    }

    #[test]
    fn from_env_overrides_values() {
        let _guard = env_lock().lock().expect("env lock");
        clear_test_env();
        unsafe {
            std::env::set_var("SERVER_HOST", "0.0.0.0");
            std::env::set_var("SERVER_PORT", "50080");
            std::env::set_var("SERVER_WORKERS", "8");
            std::env::set_var("DATABASE_URL", "postgres://x");
            std::env::set_var("DB_MAX_CONNECTIONS", "22");
            std::env::set_var("DB_MIN_CONNECTIONS", "5");
            std::env::set_var("JWT_SECRET", "jwt");
            std::env::set_var("TOKEN_EXPIRY_SECS", "7200");
            std::env::set_var("MAX_FILE_SIZE_MB", "12");
            std::env::set_var("RATE_LIMIT_PER_SECOND", "77");
            std::env::set_var("REDIS_URL", "redis://r");
            std::env::set_var("FILE_STORAGE_PATH", "/tmp/files");
            std::env::set_var("LOG_DIR", "/tmp/logs");
            std::env::set_var("LOG_FILE_PREFIX", "cp");
        }
        let cfg = Config::from_env();
        assert_eq!(cfg.server.host, "0.0.0.0");
        assert_eq!(cfg.server.port, 50080);
        assert_eq!(cfg.server.workers, 8);
        assert_eq!(cfg.database.url, "postgres://x");
        assert_eq!(cfg.database.max_connections, 22);
        assert_eq!(cfg.database.min_connections, 5);
        assert_eq!(cfg.security.jwt_secret, "jwt");
        assert_eq!(cfg.security.token_expiry_secs, 7200);
        assert_eq!(cfg.security.max_file_size_mb, 12);
        assert_eq!(cfg.security.rate_limit_per_second, 77);
        assert_eq!(cfg.redis.url, "redis://r");
        assert_eq!(cfg.storage.file_storage_path, "/tmp/files");
        assert_eq!(cfg.logging.dir, "/tmp/logs");
        assert_eq!(cfg.logging.file_prefix, "cp");
        clear_test_env();
    }
}
