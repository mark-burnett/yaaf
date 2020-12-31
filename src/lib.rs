#![forbid(unsafe_code)]

mod actor;
mod handler;
mod mailbox;
mod message;
mod publisher;
mod router;
mod source;
mod system;

pub mod error;
pub mod prelude;

pub use crate::actor::ActorAddress;
pub use crate::message::Message;
pub use crate::prelude::*;
pub use crate::publisher::Publisher;
pub use crate::source::SourceMeta;
