use std::{
    collections::HashMap,
    fs::{self, File},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::{application::exchanges::ExchangeCfg, domain::models::exchange::ExchangeId, infra::config::*};

#[derive(Deserialize)]
pub struct AppConfig {
    pub exchange_cfg_paths: HashMap<ExchangeId, PathBuf>,
    pub infra: InfraConfig,
}

#[derive(Deserialize)]
pub struct InfraConfig {
    pub server: ServerConfig,
    pub db: DbConfig,
    // pub cache: RedisConfig,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct YamlRoot {
    exchange_cfg_paths: HashMap<ExchangeId, PathBuf>,
}

impl AppConfig {
    pub fn build_config<P: AsRef<Path>>(yaml_path: P) -> Result<Self> {
        let exchange_cfg_paths = Self::from_yaml(yaml_path.as_ref())?;
        let infra = Self::from_env()?;
        Ok(Self {
            exchange_cfg_paths,
            infra,
        })
    }

    // НУЖНО ЧИТАТЬ ВСЕ ДАННЫЕ ИЗ YAML, УБРАТЬ ENV, ВСЕ ПАРОЛИ ЧИТАЕТ через Нормализуем, расширяем ENV
    pub fn from_yaml(yaml_path: &Path) -> Result<HashMap<ExchangeId, PathBuf>> {
        let file = File::open(yaml_path).with_context(|| format!("failed to open YAML at {}", yaml_path.display()))?;
        let mut yaml: YamlRoot = serde_yaml::from_reader(file).with_context(|| format!("failed to parse YAML at {}", yaml_path.display()))?;
        let base_dir = yaml_path.parent().map(Path::to_path_buf).unwrap_or_else(|| PathBuf::from("."));

        for p in yaml.exchange_cfg_paths.values_mut() {
            if p.is_relative() {
                *p = base_dir.join(&p);
            }
        }

        Ok(yaml.exchange_cfg_paths)
    }

    fn from_env() -> Result<InfraConfig> {
        dotenvy::dotenv().ok();
        let get = |k: &str, d: &str| std::env::var(k).unwrap_or_else(|_| d.to_string());

        let server = ServerConfig {
            addr: get("APP_ADDR", "0.0.0.0:8085"),
        };

        let db = DbConfig {
            url: get("DATABASE_URL", "postgres://auth_user:2803@127.0.0.1:5433/ledger"),
            max_connections: get("DB_MAX_CONNS", "10").parse().unwrap_or(10),
            timeout: get("DB_CONN_TIMEOUT", "5").parse().unwrap_or(5),
            batch_size: get("DB_BATCH_SIZE", "1000").parse().unwrap_or(1000),
        };

        // let cache = RedisConfig {
        //     url: get("REDIS_URL", "redis://127.0.0.1:6379"),
        //     max_size: get("REDIS_MAX_CONNS", "4").parse().unwrap_or(4),
        //     skew_secs: get("REDIS_SKEW_SECS", "120").parse().unwrap_or(120),
        // };

        return Ok(InfraConfig {
            server,
            db,
            // cache,
        });
    }
}

pub fn load_exchange_cfg(path: impl AsRef<Path>) -> anyhow::Result<ExchangeCfg> {
    let s = fs::read_to_string(&path).with_context(|| format!("read mexc config at {}", path.as_ref().display()))?;
    let cfg = serde_yaml::from_str(&s).with_context(|| "parse mexc config yaml")?;
    Ok(cfg)
}
