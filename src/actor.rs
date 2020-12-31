use crate::{
    handler::{detail::HandlesList, Handler},
    message::{detail::MessageList, Message},
    router::{ConcreteRouter, Router},
};
use ::std::{any::TypeId, collections::HashMap, marker::PhantomData, sync::Arc};

pub trait Actor: HandlesList<<Self as Actor>::Handles> {
    type Publishes: MessageList;
    type Handles: MessageList;
}

#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ActorId(pub(crate) u32);

#[derive(Clone)]
pub struct ActorAddress<'a, A: Actor> {
    actor_id: ActorId,
    routers: HashMap<TypeId, Arc<dyn Router>>,
    _actor: PhantomData<&'a A>,
}

impl<'a, A: Actor> ActorAddress<'a, A> {
    pub(crate) fn new(actor_id: ActorId, routers: HashMap<TypeId, Arc<dyn Router>>) -> Self {
        ActorAddress {
            actor_id,
            routers,
            _actor: PhantomData,
        }
    }
}

pub trait Tell<M: Message>: Send {
    fn tell(&self, message: M);
}

impl<'a, A, M> Tell<M> for ActorAddress<'a, A>
where
    A: Actor + Handler<M> + Send + Sync,
    M: Message,
{
    fn tell(&self, message: M) {
        let router = self.routers.get(&TypeId::of::<M>()).unwrap();
        let r: &ConcreteRouter<M> = router.as_any().downcast_ref().unwrap();
        r.tell(self.actor_id, message);
    }
}
