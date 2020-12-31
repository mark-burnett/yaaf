use crate::{
    actor::{Actor, ActorAddress, ActorId},
    message::detail::MessageList,
    router::{ConcreteRouter, Router, SysRouter, SystemMessage},
    source::{Source, SourceContext, SourceMeta},
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
    pub async fn new() -> Self {
        System {
            next_id: 0,
            routers: HashMap::new(),
            sys_router: ConcreteRouter::<SystemMessage>::new_system_router().await,
            done: Vec::new(),
        }
    }

    fn get_id(&mut self) -> ActorId {
        let result = ActorId(self.next_id);
        self.next_id += 1;
        result
    }

    pub async fn add_actor<'a, A: 'a + Actor>(&'a mut self, actor: A) -> ActorAddress<'a, A> {
        let publish_routers =
            A::Publishes::setup_routers(&self.sys_router, &mut self.routers).await;
        let handle_routers = A::Handles::setup_routers(&self.sys_router, &mut self.routers).await;

        let actor_id = self.get_id();
        self.done.extend(
            actor
                .setup_mailboxes(
                    actor_id,
                    &handle_routers,
                    &publish_routers,
                    &self.sys_router,
                )
                .await,
        );

        ActorAddress::new(actor_id, handle_routers)
    }

    pub async fn add_source<S: 'static + Source + SourceMeta>(&mut self, source: S) {
        let publish_routers =
            S::Publishes::setup_routers(&self.sys_router, &mut self.routers).await;
        let ctx: SourceContext = SourceContext::new(publish_routers);
        spawn(source.run(ctx));
    }

    pub async fn shutdown(&mut self) {
        self.sys_router.broadcast(SystemMessage::Shutdown);
        for r in &mut self.done {
            r.recv().await.unwrap();
        }
    }
}
