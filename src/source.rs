use crate::{
    message::detail::MessageList, message::Message, publisher::Publisher, router::ConcreteRouter,
    router::Router,
};
use ::async_trait::async_trait;
use ::std::{any::TypeId, collections::HashMap, sync::Arc};

pub struct SourceContext {
    routers: HashMap<TypeId, Arc<dyn Router>>,
}

impl SourceContext {
    pub(crate) fn new(routers: HashMap<TypeId, Arc<dyn Router>>) -> Self {
        SourceContext { routers }
    }

    pub fn publish<P, M>(&mut self, _publisher: &P, message: M)
    where
        P: Source + Publisher<M>,
        M: Message,
    {
        let type_id = TypeId::of::<M>();
        let router = self
            .routers
            .get(&type_id)
            .unwrap()
            .as_any()
            .downcast_ref::<ConcreteRouter<M>>()
            .unwrap();
        router.broadcast(message)
    }
}

pub trait SourceMeta {
    type Publishes: MessageList;
}

#[async_trait]
pub trait Source: SourceMeta {
    async fn run(mut self, ctx: SourceContext);
}
