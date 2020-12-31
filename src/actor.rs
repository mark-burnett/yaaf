use crate::{
    error::AddressError,
    handler::{detail::HandlesList, Handler},
    message::{detail::MessageList, Message},
    router::{ConcreteRouter, Router},
};
use ::std::{any::TypeId, collections::HashMap, marker::PhantomData, sync::Arc};

pub trait Actor: Sized + HandlesList<<Self as Actor>::Handles> {
    type Publishes: MessageList;
    type Handles: MessageList;
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ActorId(pub(crate) u32);

#[derive(Debug)]
pub struct ActorAddress<A: Actor> {
    actor_id: ActorId,
    routers: HashMap<TypeId, Arc<dyn Router>>,
    _actor: PhantomData<Arc<A>>,
}

impl<A: Actor> Clone for ActorAddress<A> {
    fn clone(&self) -> Self {
        ActorAddress {
            actor_id: self.actor_id,
            routers: self.routers.clone(),
            _actor: PhantomData,
        }
    }
}

impl<A: Actor> ActorAddress<A> {
    pub(crate) fn new(actor_id: ActorId, routers: HashMap<TypeId, Arc<dyn Router>>) -> Self {
        ActorAddress {
            actor_id,
            routers,
            _actor: PhantomData,
        }
    }
}

pub trait Tell<M: Message> {
    fn tell(&self, message: M) -> Result<(), AddressError>;
}

impl<H, M> Tell<M> for ActorAddress<H>
where
    H: Handler<M>,
    M: Message,
{
    fn tell(&self, message: M) -> Result<(), AddressError> {
        let router = self
            .routers
            .get(&TypeId::of::<M>())
            .ok_or(AddressError::RouterLookupError)?;
        let r: &ConcreteRouter<M> = router
            .as_any()
            .downcast_ref()
            .ok_or(AddressError::RouterLookupError)?;
        r.tell(self.actor_id, message)
            .map_err(|source| AddressError::TellFailure {
                source: source.into(),
            })?;
        Ok(())
    }
}
