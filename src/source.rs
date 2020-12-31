use crate::{context::Context, message::detail::MessageList};
use ::async_trait::async_trait;

pub trait SourceMeta {
    type Publishes: MessageList;
}

#[async_trait]
pub trait Source: Sized + SourceMeta {
    async fn run(mut self, ctx: Context<Self>);
}
