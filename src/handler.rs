use crate::{actor::Actor, context::Context, message::Message};
use ::async_trait::async_trait;

#[doc(hidden)]
pub trait HandlerRegistered<M: Message> {}

#[async_trait]
pub trait Handler<M: Message>: Actor + HandlerRegistered<M> + Send {
    async fn handle(&mut self, ctx: &mut Context<Self>, message: M);
}

pub(crate) mod detail {
    use super::*;
    use crate::{
        actor::Actor,
        error::YaafInternalError,
        mailbox::Mailbox,
        message::{detail::MessageList, Message, SystemMessage},
        router::Router,
    };
    use ::async_trait::async_trait;
    use std::{any::TypeId, collections::HashMap, sync::Arc};
    use tokio::sync::{broadcast::Sender, mpsc::Receiver, Mutex};

    #[async_trait]
    pub trait HandlesList<ML: MessageList + ?Sized> {
        async fn setup_mailboxes(
            self,
            handle_routers: &HashMap<TypeId, Box<dyn Router>>,
            publish_routers: &HashMap<TypeId, Box<dyn Router>>,
            sys_router: Sender<SystemMessage>,
        ) -> Result<(HashMap<TypeId, Box<dyn Router>>, Vec<Receiver<()>>), YaafInternalError>;
    }

    macro_rules! start_mailbox {
        ( $actor:ident, $sys_router:ident, $handle_routers:ident, $publish_routers:ident, $tell_routers:ident, $done:ident, $head:ident, $( $tail:ident, )* ) => {
            let type_id = TypeId::of::<$head>();
            let router = $handle_routers
                .get(&type_id)
                .ok_or(YaafInternalError::RouterLookupFailure)?
                .as_any()
                .downcast_ref::<Sender<$head>>()
                .ok_or(YaafInternalError::RouterLookupFailure)?;
            let (tell, done) = Mailbox::start($actor.clone(), router.subscribe(), $sys_router.subscribe(), $publish_routers.clone()).await?;
            $done.push(done);
            $tell_routers.insert(type_id, Box::new(tell));

            start_mailbox!($actor, $sys_router, $handle_routers, $publish_routers, $tell_routers, $done, $( $tail, )*);
        };
        ($actor:ident, $sys_router:ident, $handle_routers:ident, $publish_routers:ident, $tell_routers:ident, $done:ident,) => {};
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
                    handle_routers: &HashMap<TypeId, Box<dyn Router>>,
                    publish_routers: &HashMap<TypeId, Box<dyn Router>>,
                    sys_router: Sender<SystemMessage>,
                ) -> Result<(HashMap<TypeId, Box<dyn Router>>, Vec<Receiver<()>>), YaafInternalError> {
                    let actor = Arc::new(Mutex::new(self));
                    let mut tell_routers: HashMap<TypeId, Box<dyn Router>> = HashMap::new();
                    let mut done = Vec::new();
                    start_mailbox!(
                        actor,
                        sys_router,
                        handle_routers,
                        publish_routers,
                        tell_routers,
                        done,
                        $head,
                        $( $tail, )*
                    );
                    Ok((tell_routers, done))
                }
            }

            impl_handles_list!($( $tail, )*);
        };
        () => {};
    }

    impl_handles_list!(M10, M9, M8, M7, M6, M5, M4, M3, M2, M1,);
}
