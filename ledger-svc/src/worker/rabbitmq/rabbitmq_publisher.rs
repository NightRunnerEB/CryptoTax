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
use tracing::info;

use super::{
    LedgerMsg, OutgoingMessage,
    config::RabbitmqPublishConfig,
    rabbitmq::{ChannelControl, ConnectionControl},
    rabbitmq_client::RabbitmqClient,
};
use crate::{
    domain::error::{LedgerError, Result},
    infra::config::ReconnectConfig,
};

pub(super) struct RabbitmqPublisher<M>
where
    M: OutgoingMessage,
{
    config: RabbitmqPublishConfig,
    msg_receiver: UnboundedReceiver<M>,
    buffered_msg: Option<M>,
    close_notify: Arc<Notify>,
    connection: Mutex<Option<(Connection, Channel)>>,
}

impl<M> RabbitmqPublisher<M>
where
    M: OutgoingMessage,
{
    pub(super) fn new(config: RabbitmqPublishConfig, msg_receiver: UnboundedReceiver<M>) -> RabbitmqPublisher<M> {
        RabbitmqPublisher {
            config,
            msg_receiver,
            buffered_msg: None,
            close_notify: Arc::new(Notify::new()),
            connection: Mutex::new(None),
        }
    }

    pub(super) async fn publish_to_rabbitmq(&mut self) -> Result<()> {
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
                msg = self.next_msg_to_process() => msg
            };
            let Some(msg) = msg else {
                return Ok(());
            };
            self.publish_msg(msg).await;
        }
    }

    async fn publish_msg(&mut self, msg: M) {
        let log_ctx = msg.context_id();
        let Ok(json_data) = serde_json::to_vec(&msg).map_err(|err| {
            // Here you can log with context
            // log_with_ctx!(error, log_ctx, "Failed to encode message: {}", err);
        }) else {
            return;
        };
        // log_with_ctx!(
        //     debug,
        //     log_ctx,
        //     "transmitter_msg to be sent: {}",
        //     String::from_utf8_lossy(&json_data)
        // );
        let args = BasicPublishArguments::from(&self.config.binding);
        let guard = self.connection.lock().await;
        let (_, channel) = guard.as_ref().expect("Expected rabbitmq channel to be set");
        let res = channel.basic_publish(BasicProperties::default(), json_data, args.clone()).await;
        let _ = res.map_err(|err| {
            self.buffered_msg = Some(msg);
            // log_with_ctx!(
            //     error,
            //     log_ctx,
            //     "Failed to publish operation_data message, error: {}",
            //     err
            // );
        });
    }

    async fn next_msg_to_process(&mut self) -> Option<M> {
        if self.buffered_msg.is_some() {
            self.buffered_msg.take()
        } else {
            self.msg_receiver.recv().await
        }
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
