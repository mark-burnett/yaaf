use crate::message::Message;
use ::dyn_clone::{clone_trait_object, DynClone};
use ::std::{any::Any, fmt::Debug};
use ::tokio::sync::{broadcast, mpsc};

pub trait DirectChannel: Any + DynClone + Debug + Send + Sync {
    fn as_any(&self) -> &dyn Any;
}

clone_trait_object!(DirectChannel);

impl<M: 'static + Message> DirectChannel for mpsc::UnboundedSender<M> {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub trait BroadcastChannel: Any + DynClone + Debug + Send + Sync {
    fn as_any(&self) -> &dyn Any;
}

clone_trait_object!(BroadcastChannel);

impl<M: 'static + Message> BroadcastChannel for broadcast::Sender<M> {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
