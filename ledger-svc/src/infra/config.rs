use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct ServerConfig {
    pub addr: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DbConfig {
    pub url: String,
    pub max_connections: u32,
    pub timeout: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub max_size: u32,
    pub skew_secs: i64,
}
