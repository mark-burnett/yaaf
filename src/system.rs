use crate::{
    actor::{Actor, ActorAddress},
    context::Context,
    error::SystemError,
    message::{detail::MessageList, SystemMessage},
    router::Router,
    source::{Source, SourceMeta},
};
use ::std::{any::TypeId, collections::HashMap};
use ::tokio::{
    spawn,
    sync::{broadcast, mpsc},
};

pub struct System {
    routers: HashMap<TypeId, Box<dyn Router>>,
    sys_router: broadcast::Sender<SystemMessage>,
    done: Vec<mpsc::Receiver<()>>,
}

impl System {
    pub fn new() -> Self {
        System {
            routers: HashMap::new(),
            sys_router: broadcast::channel(1000).0,
            done: Vec::new(),
        }
    }

    pub async fn add_actor<A: Actor>(&mut self, actor: A) -> Result<ActorAddress<A>, SystemError> {
        let publish_routers =
            A::Publishes::setup_routers(self.sys_router.clone(), &mut self.routers)
                .await
                .map_err(|source| SystemError::AddActorFailure { source })?;
        let handle_routers = A::Handles::setup_routers(self.sys_router.clone(), &mut self.routers)
            .await
            .map_err(|source| SystemError::AddActorFailure { source })?;

        let (tell_routers, done) = actor
            .setup_mailboxes(&handle_routers, &publish_routers, self.sys_router.clone())
            .await
            .map_err(|source| SystemError::AddActorFailure { source })?;
        self.done.extend(done);

        Ok(ActorAddress::new(tell_routers))
    }

    pub async fn add_source<S: 'static + Source + SourceMeta>(
        &mut self,
        source: S,
    ) -> Result<(), SystemError> {
        let publish_routers =
            S::Publishes::setup_routers(self.sys_router.clone(), &mut self.routers)
                .await
                .map_err(|source| SystemError::AddSourceFailure { source })?;
        let ctx = Context::new(publish_routers);
        spawn(source.run(ctx));
        Ok(())
    }

    pub async fn shutdown(&mut self) -> Result<(), SystemError> {
        self.sys_router
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
