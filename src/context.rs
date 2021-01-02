use crate::{
    channel::BroadcastChannel, error::ContextError, message::Message, publisher::Publisher,
};
use ::std::{any::TypeId, collections::HashMap, marker::PhantomData, sync::atomic::AtomicPtr};
use ::tokio::sync::broadcast::Sender;

pub struct Context<A> {
    channels: HashMap<TypeId, Box<dyn BroadcastChannel>>,
    _actor: PhantomData<AtomicPtr<A>>,
}

impl<A> Context<A> {
    pub(crate) fn new(channels: HashMap<TypeId, Box<dyn BroadcastChannel>>) -> Self {
        Context {
            channels: channels,
            _actor: PhantomData,
        }
    }
}

pub trait Publish<M: Message> {
    fn publish(&mut self, message: M) -> Result<(), ContextError>;
}

impl<P, M> Publish<M> for Context<P>
where
    P: Publisher<M>,
    M: Message,
{
    fn publish(&mut self, message: M) -> Result<(), ContextError> {
        let type_id = TypeId::of::<M>();
        let channel = self
            .channels
            .get(&type_id)
            .ok_or(ContextError::ChannelLookupError)?
            .as_any()
            .downcast_ref::<Sender<M>>()
            .ok_or(ContextError::ChannelLookupError)?;

        channel
            .send(message)
            .map_err(|source| ContextError::BroadcastFailure {
                source: source.into(),
            })?;
        Ok(())
    }
}
