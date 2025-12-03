use serde::Deserialize;

use super::rabbitmq_client::{RabbitmqBindingConfig, RabbitmqConnectConfig};
use crate::infra::config::ReconnectConfig;

#[derive(Deserialize)]
pub(crate) struct RabbitmqPublishConfig {
    #[serde(flatten)]
    pub(crate) connect: RabbitmqConnectConfig,
    #[serde(flatten)]
    pub(crate) binding: RabbitmqBindingConfig,
    #[serde(flatten)]
    pub(crate) reconnect: ReconnectConfig,
}
