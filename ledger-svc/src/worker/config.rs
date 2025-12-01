use std::{fs::File, path::Path};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::{
    infra::config::DbConfig,
    worker::{outbox::OutboxWorkerConfig, rabbitmq::config::RabbitmqPublishConfig},
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkerYamlRoot {
    pub rabbitmq_publisher: RabbitmqPublishConfig,
    pub outbox_worker: OutboxWorkerConfig,
}

#[derive(Deserialize)]
pub struct WorkerConfig {
    pub db: DbConfig,
    pub rabbitmq: RabbitmqPublishConfig,
    pub outbox_worker: OutboxWorkerConfig,
}

impl WorkerConfig {
    pub fn build_config<P: AsRef<Path>>(yaml_path: P) -> Result<Self> {
        let db = Self::db_from_env()?;
        let rabbitmq = Self::from_yaml(yaml_path.as_ref())?.rabbitmq_publisher;
        let outbox_worker = Self::from_yaml(yaml_path.as_ref())?.outbox_worker;
        Ok(Self {
            db,
            rabbitmq,
            outbox_worker,
        })
    }

    fn from_yaml(yaml_path: &Path) -> Result<WorkerYamlRoot> {
        let file = File::open(yaml_path).with_context(|| format!("failed to open worker YAML at {}", yaml_path.display()))?;
        let root: WorkerYamlRoot =
            serde_yaml::from_reader(file).with_context(|| format!("failed to parse worker YAML at {}", yaml_path.display()))?;

        Ok(root)
    }

    fn db_from_env() -> Result<DbConfig> {
        dotenvy::dotenv().ok();
        let get = |k: &str, d: &str| std::env::var(k).unwrap_or_else(|_| d.to_string());

        // УБРАТЬ ЭТОТ GET и КОНФ ДАННЫЕ
        let url = get("DATABASE_URL", "postgres://ledger_user:2803@127.0.0.1:5433/ledger");
        let max_connections = get("DB_MAX_CONNS", "10").parse().unwrap_or(10);
        let timeout = get("DB_CONN_TIMEOUT", "5").parse().unwrap_or(5);

        Ok(DbConfig {
            url,
            max_connections,
            timeout,
        })
    }
}
