use crate::{
    actor::{Actor, ActorAddress, ActorId},
    context::Context,
    error::SystemError,
    message::detail::MessageList,
    router::{ConcreteRouter, Router, SysRouter, SystemMessage},
    source::{Source, SourceMeta},
};
use ::std::{any::TypeId, collections::HashMap, sync::Arc};
use ::tokio::{spawn, sync::mpsc::Receiver};

pub struct System {
    next_id: u32,
    routers: HashMap<TypeId, Arc<dyn Router>>,
    sys_router: SysRouter,
    done: Vec<Receiver<()>>,
}

impl System {
    pub async fn new() -> Result<Self, SystemError> {
        Ok(System {
            next_id: 0,
            routers: HashMap::new(),
            sys_router: ConcreteRouter::<SystemMessage>::new_system_router()
                .await
                .map_err(|source| SystemError::CreateError { source })?,
            done: Vec::new(),
        })
    }

    fn get_id(&mut self) -> ActorId {
        let result = ActorId(self.next_id);
        self.next_id += 1;
        result
    }

    pub async fn add_actor<A: Actor>(&mut self, actor: A) -> Result<ActorAddress<A>, SystemError> {
        let publish_routers = A::Publishes::setup_routers(&self.sys_router, &mut self.routers)
            .await
            .map_err(|source| SystemError::AddActorFailure { source })?;
        let handle_routers = A::Handles::setup_routers(&self.sys_router, &mut self.routers)
            .await
            .map_err(|source| SystemError::AddActorFailure { source })?;

        let actor_id = self.get_id();
        self.done.extend(
            actor
                .setup_mailboxes(
                    actor_id,
                    &handle_routers,
                    &publish_routers,
                    &self.sys_router,
                )
                .await
                .map_err(|source| SystemError::AddActorFailure { source })?,
        );

        Ok(ActorAddress::new(actor_id, handle_routers))
    }

    pub async fn add_source<S: 'static + Source + SourceMeta>(
        &mut self,
        source: S,
    ) -> Result<(), SystemError> {
        let publish_routers = S::Publishes::setup_routers(&self.sys_router, &mut self.routers)
            .await
            .map_err(|source| SystemError::AddSourceFailure { source })?;
        let ctx = Context::new(publish_routers);
        spawn(source.run(ctx));
        Ok(())
    }

    pub async fn shutdown(&mut self) -> Result<(), SystemError> {
        self.sys_router
            .broadcast(SystemMessage::Shutdown)
            .map_err(|source| SystemError::ShutdownError { source })?;
        for r in &mut self.done {
            // TODO: log this error
            let _ = r.recv().await;
        }
        Ok(())
    }
}
