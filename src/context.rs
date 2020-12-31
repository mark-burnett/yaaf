use crate::{
    error::ContextError,
    message::Message,
    publisher::Publisher,
    router::{ConcreteRouter, Router},
};
use std::{
    any::TypeId,
    collections::HashMap,
    marker::PhantomData,
    sync::{atomic::AtomicPtr, Arc},
};

pub struct Context<A> {
    routers: HashMap<TypeId, Arc<dyn Router>>,
    _actor: PhantomData<AtomicPtr<Box<A>>>,
}

impl<A> Context<A> {
    pub(crate) fn new(routers: HashMap<TypeId, Arc<dyn Router>>) -> Self {
        Context {
            routers,
            _actor: PhantomData,
        }
    }
}

pub trait PublishContext<M: Message> {
    fn publish(&mut self, message: M) -> Result<(), ContextError>;
}

impl<A, M> PublishContext<M> for Context<A>
where
    A: Publisher<M>,
    M: Message,
{
    fn publish(&mut self, message: M) -> Result<(), ContextError> {
        let type_id = TypeId::of::<M>();
        let router = self
            .routers
            .get(&type_id)
            .ok_or(ContextError::RouterLookupError)?
            .as_any()
            .downcast_ref::<ConcreteRouter<M>>()
            .ok_or(ContextError::RouterLookupError)?;
        router
            .broadcast(message)
            .map_err(|source| ContextError::BroadcastFailure {
                source: source.into(),
            })?;
        Ok(())
    }
}