use crate::{
    config::{Channel, Config},
    nats::Conn,
};

use super::{ChannelSupervisor, Error};
use crate::channel::messages::outgoing;
use act_zero::*;
use async_nats::{Message, Subscription};
use async_trait::async_trait;
use log::{debug, error, info};
use std::sync::Arc;

#[async_trait]
impl Actor for Info {
    async fn started(&mut self, pid: Addr<Self>) -> ActorResult<()> {
        send!(pid.listen());
        Produces::ok(())
    }

    async fn error(&mut self, error: ActorError) -> bool {
        error!("Info Actor Error: {:?}", error);

        // do not stop on actor error
        false
    }
}
pub struct Info {
    config: Arc<Config>,
    parent: WeakAddr<ChannelSupervisor>,
    channel: Subscription,
}

impl Info {
    pub async fn new(
        parent: WeakAddr<ChannelSupervisor>,
        config: &Arc<Config>,
        conn: &Conn,
    ) -> Self {
        Self {
            parent,
            channel: conn
                .subscribe(&Channel::DiscoverTargeted.channel_to_string(&config))
                .await
                .unwrap(),
            config: Arc::clone(config),
        }
    }

    // INFO packets received when a new client connects and broadcasts it's INFO
    pub async fn listen(&mut self) {
        info!("Listening for INFO messages");

        while let Some(msg) = self.channel.next().await {
            match self.handle_message(msg).await {
                Ok(_) => debug!("Successfully handled INFO message"),
                Err(e) => error!("Unable to handle INFO message: {}", e),
            }
        }
    }

    pub async fn broadcast_info(&self) -> ActorResult<()> {
        let info = outgoing::InfoMessage::new(&self.config);

        send!(self
            .parent
            .publish(Channel::Info, self.config.serialize(info)?));

        Produces::ok(())
    }

    async fn handle_message(&self, msg: Message) -> Result<(), Error> {
        // TODO: save to registry
        Ok(())
    }
}

#[async_trait]
impl Actor for InfoTargeted {
    async fn started(&mut self, pid: Addr<Self>) -> ActorResult<()> {
        send!(pid.listen());
        Produces::ok(())
    }

    async fn error(&mut self, error: ActorError) -> bool {
        error!("InfoTargeted Actor Error: {:?}", error);

        // do not stop on actor error
        false
    }
}
pub struct InfoTargeted {
    config: Arc<Config>,
    parent: WeakAddr<ChannelSupervisor>,
    channel: Subscription,
}

impl InfoTargeted {
    pub async fn new(
        parent: WeakAddr<ChannelSupervisor>,
        config: &Arc<Config>,
        conn: &Conn,
    ) -> Self {
        Self {
            parent,
            channel: conn
                .subscribe(&Channel::DiscoverTargeted.channel_to_string(&config))
                .await
                .unwrap(),
            config: Arc::clone(config),
        }
    }
    // INFO packets received are responses to DISCOVER packet sent by current client
    pub async fn listen(&mut self) {
        info!("Listening for INFO (targeted) messages");

        while let Some(msg) = self.channel.next().await {
            match self.handle_message(msg).await {
                Ok(_) => debug!("Successfully handled INFO message in response to DISCOVER"),
                Err(e) => error!(
                    "Unable to handle INFO message in response to DISCOVER: {}",
                    e
                ),
            }
        }
    }

    async fn handle_message(&self, msg: Message) -> Result<(), Error> {
        // TODO: save to registry
        Ok(())
    }
}
