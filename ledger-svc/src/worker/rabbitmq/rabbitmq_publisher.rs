use std::sync::Arc;

use amqprs::{
    BasicProperties,
    channel::{BasicPublishArguments, Channel},
    connection::Connection,
};
use axum::async_trait;
use tokio::{
    select,
    sync::{Mutex, Notify, mpsc::UnboundedReceiver},
};
use tracing::{debug, error, info};

use super::{
    OutgoingMessage, PublishRequest,
    config::RabbitmqPublishConfig,
    rabbitmq::{ChannelControl, ConnectionControl},
    rabbitmq_client::RabbitmqClient,
};
// ИЗМЕНИТЬ RESULT ОН НАМ ТУТ НЕ НУЖЕН ОТ DOMAIN
use crate::{
    domain::error::{LedgerError, Result},
    infra::config::ReconnectConfig,
};

pub(crate) struct RabbitmqPublisher<M>
where
    M: OutgoingMessage,
{
    config: RabbitmqPublishConfig,
    msg_receiver: UnboundedReceiver<PublishRequest<M>>,
    close_notify: Arc<Notify>,
    connection: Mutex<Option<(Connection, Channel)>>,
}

impl<M> RabbitmqPublisher<M>
where
    M: OutgoingMessage,
{
    pub(crate) fn new(config: RabbitmqPublishConfig, msg_receiver: UnboundedReceiver<PublishRequest<M>>) -> RabbitmqPublisher<M> {
        RabbitmqPublisher {
            config,
            msg_receiver,
            close_notify: Arc::new(Notify::new()),
            connection: Mutex::new(None),
        }
    }

    pub(crate) async fn publish_to_rabbitmq(&mut self) -> Result<()> {
        info!(
            "Rabbitmq messaging arguments are: exchange: {}, routing_key: {}",
            self.config.binding.exchange, self.config.binding.routing_key
        );
        self.init_connection().await?;
        let notify = self.close_notify.clone();
        loop {
            let msg = select! {
                _ = notify.notified() => {
                    self.init_connection().await?;
                    continue
                },
                msg = self.msg_receiver.recv() => msg
            };
            let Some(req) = msg else {
                return Ok(());
            };
            self.publish_msg(req).await;
        }
    }

    async fn publish_msg(&mut self, req: PublishRequest<M>) {
        let PublishRequest {
            msg,
            ack,
        } = req;
        let log_ctx = msg.context_id();
        let json_data = match serde_json::to_vec(&msg) {
            Ok(data) => data,
            Err(err) => {
                error!(
                    context_id = log_ctx.as_deref().unwrap_or(""),
                    error = %err,
                    "Failed to serialize message for RabbitMQ"
                );
                let _ = ack.send(Err(format!("serialize error: {err}")));
                return;
            }
        };

        debug!(
            context_id = log_ctx.as_deref().unwrap_or(""),
            message = ?msg,
            "Message to be sent"
        );
        let args = BasicPublishArguments::from(&self.config.binding);
        let guard = self.connection.lock().await;
        let Some((_, channel)) = guard.as_ref() else {
            error!(context_id = log_ctx.as_deref().unwrap_or(""), "RabbitMQ channel is not initialized");
            let _ = ack.send(Err("rabbitmq channel is not initialized".to_string()));
            return;
        };
        if let Err(err) = channel.basic_publish(BasicProperties::default(), json_data, args).await {
            error!(
                context_id = log_ctx.as_deref().unwrap_or(""),
                error = %err,
                "Failed to publish message to RabbitMQ"
            );
            let _ = ack.send(Err(format!("basic_publish failed: {err}")));
            return;
        }
        let _ = ack.send(Ok(()));
    }
}

#[async_trait]
impl<M> RabbitmqClient for RabbitmqPublisher<M>
where
    M: OutgoingMessage,
{
    type Error = LedgerError;

    async fn reconnect(&self) -> Result<()> {
        let conn_control = ConnectionControl::new(self.close_notify.clone());
        let conn = self.connect(&self.config.connect, conn_control).await?;
        let chann_control = ChannelControl::new(self.close_notify.clone());
        let channel = self.open_channel(&conn, chann_control).await?;
        self.connection.lock().await.replace((conn, channel));
        Ok(())
    }

    fn reconnect_config(&self) -> &ReconnectConfig {
        &self.config.reconnect
    }
}
