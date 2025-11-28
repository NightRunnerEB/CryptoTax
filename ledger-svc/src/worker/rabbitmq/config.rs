use serde::Deserialize;

use crate::infra::config::ReconnectConfig;

use super::rabbitmq_client::{RabbitmqBindingConfig, RabbitmqConnectConfig};

#[derive(Deserialize)]
pub(crate) struct RabbitmqPublishConfig {
    #[serde(flatten)]
    pub(crate) connect: RabbitmqConnectConfig,
    #[serde(flatten)]
    pub(crate) binding: RabbitmqBindingConfig,
    #[serde(flatten)]
    pub(crate) reconnect: ReconnectConfig,
}
