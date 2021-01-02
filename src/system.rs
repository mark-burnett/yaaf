use crate::{
    actor::{Actor, ActorAddress},
    channel::BroadcastChannel,
    context::Context,
    error::SystemError,
    message::{detail::MessageList, SystemMessage},
    source::{Source, SourceMeta},
};
use ::std::{any::TypeId, collections::HashMap};
use ::tokio::{
    spawn,
    sync::{broadcast, mpsc},
};

pub struct System {
    broadcast_channels: HashMap<TypeId, Box<dyn BroadcastChannel>>,
    system_channel: broadcast::Sender<SystemMessage>,
    done: Vec<mpsc::Receiver<()>>,
}

impl Default for System {
    fn default() -> Self {
        Self::new()
    }
}

impl System {
    pub fn new() -> Self {
        System {
            broadcast_channels: HashMap::new(),
            system_channel: broadcast::channel(1000).0,
            done: Vec::new(),
        }
    }

    pub async fn add_actor<A: Actor>(&mut self, actor: A) -> Result<ActorAddress<A>, SystemError> {
        let publish_channels =
            A::Publishes::setup_channels(self.system_channel.clone(), &mut self.broadcast_channels)
                .await
                .map_err(|source| SystemError::AddActorFailure { source })?;
        let handle_channels =
            A::Handles::setup_channels(self.system_channel.clone(), &mut self.broadcast_channels)
                .await
                .map_err(|source| SystemError::AddActorFailure { source })?;

        let (direct_channels, done) = actor
            .setup_mailboxes(
                &handle_channels,
                &publish_channels,
                self.system_channel.clone(),
            )
            .await
            .map_err(|source| SystemError::AddActorFailure { source })?;
        self.done.extend(done);

        Ok(ActorAddress::new(direct_channels))
    }

    pub async fn add_source<S: 'static + Source + SourceMeta>(
        &mut self,
        source: S,
    ) -> Result<(), SystemError> {
        let publish_channels =
            S::Publishes::setup_channels(self.system_channel.clone(), &mut self.broadcast_channels)
                .await
                .map_err(|source| SystemError::AddSourceFailure { source })?;
        let ctx = Context::new(publish_channels);
        spawn(source.run(ctx));
        Ok(())
    }

    pub async fn shutdown(&mut self) -> Result<(), SystemError> {
        self.system_channel
            .send(SystemMessage::Shutdown)
            .map_err(|source| SystemError::ShutdownError {
                source: source.into(),
            })?;
        for r in &mut self.done {
            // TODO: log this error
            let _ = r.recv().await;
        }
        Ok(())
    }
}
