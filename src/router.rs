use crate::message::Message;
use ::dyn_clone::{clone_trait_object, DynClone};
use ::std::{any::Any, fmt::Debug};
use ::tokio::sync::{broadcast, mpsc};

pub trait Router: Any + DynClone + Debug + Send + Sync {
    fn as_any(&self) -> &dyn Any;
}

clone_trait_object!(Router);

impl<M: 'static + Message> Router for broadcast::Sender<M> {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<M: 'static + Message> Router for mpsc::UnboundedSender<M> {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
