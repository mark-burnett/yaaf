use crate::{actor::Actor, context::Context, message::Message};
use ::async_trait::async_trait;

pub trait HandlerRegistered<M: Message> {}

#[async_trait]
pub trait Handler<M: Message>: Actor + HandlerRegistered<M> + Send {
    async fn handle(&mut self, ctx: &mut Context<Self>, message: M);
}

pub(crate) mod detail {
    use super::*;
    use crate::{
        actor::{Actor, ActorId},
        error::YaafInternalError,
        mailbox::Mailbox,
        message::{detail::MessageList, Message},
        router::{ConcreteRouter, Router, SysRouter},
    };
    use ::async_trait::async_trait;
    use std::{any::TypeId, collections::HashMap, sync::Arc};
    use tokio::sync::{mpsc::Receiver, Mutex};

    #[async_trait]
    pub trait HandlesList<ML: MessageList + ?Sized> {
        async fn setup_mailboxes(
            self,
            actor_id: ActorId,
            handle_routers: &HashMap<TypeId, Arc<dyn Router>>,
            publish_routers: &HashMap<TypeId, Arc<dyn Router>>,
            sys_router: &SysRouter,
        ) -> Result<Vec<Receiver<()>>, YaafInternalError>;
    }

    macro_rules! start_mailbox {
        ( $actor:ident, $actor_id:ident, $sys_router:ident, $handle_routers:ident, $publish_routers:ident, $done:ident, $head:ident, $( $tail:ident, )* ) => {
            let type_id = TypeId::of::<$head>();
            let router = $handle_routers
                .get(&type_id)
                .ok_or(YaafInternalError::RouterLookupFailure)?
                .as_any()
                .downcast_ref::<ConcreteRouter<$head>>()
                .ok_or(YaafInternalError::RouterLookupFailure)?;
            $done.push(Mailbox::start($actor.clone(), $actor_id, $sys_router, router, $publish_routers.clone()).await?);

            start_mailbox!($actor, $actor_id, $sys_router, $handle_routers, $publish_routers, $done, $( $tail, )*);
        };
        ($actor:ident, $actor_id:ident, $sys_router:ident, $handle_routers:ident, $publish_routers:ident, $done:ident,) => {};
    }

    macro_rules! impl_handles_list {
        ( $head:ident, $( $tail:ident, )* ) => {
            #[async_trait]
            impl<A, $head, $( $tail ),*> HandlesList<($head, $( $tail ),*)> for A
            where
                A: 'static + Actor + Handler<$head>$( + Handler<$tail>)*,
                $head: Message, $( $tail: Message ),*
            {
                async fn setup_mailboxes(
                    self,
                    actor_id: ActorId,
                    handle_routers: &HashMap<TypeId, Arc<dyn Router>>,
                    publish_routers: &HashMap<TypeId, Arc<dyn Router>>,
                    sys_router: &SysRouter,
                ) -> Result<Vec<Receiver<()>>, YaafInternalError> {
                    let actor = Arc::new(Mutex::new(self));
                    let mut done = Vec::new();
                    start_mailbox!(
                        actor,
                        actor_id,
                        sys_router,
                        handle_routers,
                        publish_routers,
                        done,
                        $head,
                        $( $tail, )*
                    );
                    Ok(done)
                }
            }

            impl_handles_list!($( $tail, )*);
        };
        () => {};
    }

    impl_handles_list!(M10, M9, M8, M7, M6, M5, M4, M3, M2, M1,);
}
