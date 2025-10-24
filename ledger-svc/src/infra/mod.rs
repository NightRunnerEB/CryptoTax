pub mod config;
pub mod db;
pub mod rabbitmq_client;

use std::str::FromStr;

use serde::{Deserialize, Deserializer};

#[derive(Clone, Debug, Deserialize)]
pub struct ReconnectConfig {
    #[serde(default = "ReconnectConfig::default_reconnect_attempts", rename = "reconnect_attempts")]
    pub attempts: usize,
    #[serde(default = "ReconnectConfig::default_reconnect_timeout_ms", rename = "reconnect_timeout_ms")]
    pub timeout_ms: u64,
}

impl ReconnectConfig {
    fn default_reconnect_timeout_ms() -> u64 {
        500
    }
    fn default_reconnect_attempts() -> usize {
        20
    }
}

pub fn deserialize_u128<'de, D>(deserializer: D) -> Result<u128, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    u128::from_str(&s).map_err(serde::de::Error::custom)
}
