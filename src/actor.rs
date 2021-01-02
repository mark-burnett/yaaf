use crate::{
    error::AddressError,
    handler::{detail::HandlesList, Handler},
    message::{detail::MessageList, Message},
    router::Router,
};
use ::std::{any::TypeId, collections::HashMap, marker::PhantomData, sync::Arc};
use ::tokio::sync::mpsc::UnboundedSender;

pub trait Actor: Sized + HandlesList<<Self as Actor>::Handles> {
    type Publishes: MessageList;
    type Handles: MessageList;
}

#[derive(Debug)]
pub struct ActorAddress<A: Actor> {
    routers: HashMap<TypeId, Box<dyn Router>>,
    _actor: PhantomData<Arc<A>>,
}

impl<A: Actor> Clone for ActorAddress<A> {
    fn clone(&self) -> Self {
        ActorAddress {
            routers: self.routers.clone(),
            _actor: PhantomData,
        }
    }
}

impl<A: Actor> ActorAddress<A> {
    pub(crate) fn new(routers: HashMap<TypeId, Box<dyn Router>>) -> Self {
        ActorAddress {
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
            .ok_or(AddressError::RouterLookupError)?
            .as_any()
            .downcast_ref::<UnboundedSender<M>>()
            .ok_or(AddressError::RouterLookupError)?;
        router
            .send(message)
            .map_err(|source| AddressError::TellFailure {
                source: source.into(),
            })?;
        Ok(())
    }
}
