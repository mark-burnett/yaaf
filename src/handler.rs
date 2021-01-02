use crate::{actor::Actor, context::Context, message::Message};
use ::async_trait::async_trait;

#[doc(hidden)]
pub trait HandlerRegistered<M: Message> {}

#[async_trait]
pub trait Handler<M: Message>: Actor + HandlerRegistered<M> + Send {
    async fn handle(&mut self, ctx: &mut Context<Self>, message: M);
}

pub(crate) mod detail {
    use super::*;
    use crate::{
        actor::Actor,
        channel::{BroadcastChannel, DirectChannel},
        error::YaafInternalError,
        mailbox::Mailbox,
        message::{detail::MessageList, Message, SystemMessage},
    };
    use ::async_trait::async_trait;
    use std::{any::TypeId, collections::HashMap, sync::Arc};
    use tokio::sync::{broadcast::Sender, mpsc::Receiver, Mutex};

    #[async_trait]
    pub trait HandlesList<ML: MessageList + ?Sized> {
        async fn setup_mailboxes(
            self,
            handle_channels: &HashMap<TypeId, Box<dyn BroadcastChannel>>,
            publish_channels: &HashMap<TypeId, Box<dyn BroadcastChannel>>,
            system_channel: Sender<SystemMessage>,
        ) -> Result<(HashMap<TypeId, Box<dyn DirectChannel>>, Vec<Receiver<()>>), YaafInternalError>;
    }

    macro_rules! start_mailbox {
        ( $actor:ident, $system_channel:ident, $handle_channels:ident, $publish_channels:ident, $direct_channels:ident, $done:ident, $head:ident, $( $tail:ident, )* ) => {
            let type_id = TypeId::of::<$head>();
            let channel = $handle_channels
                .get(&type_id)
                .ok_or(YaafInternalError::ChannelLookupFailure)?
                .as_any()
                .downcast_ref::<Sender<$head>>()
                .ok_or(YaafInternalError::ChannelLookupFailure)?;
            let (tell, done) = Mailbox::start($actor.clone(), channel.subscribe(), $system_channel.subscribe(), $publish_channels.clone()).await?;
            $done.push(done);
            $direct_channels.insert(type_id, Box::new(tell));

            start_mailbox!($actor, $system_channel, $handle_channels, $publish_channels, $direct_channels, $done, $( $tail, )*);
        };
        ($actor:ident, $system_channel:ident, $handle_channels:ident, $publish_channels:ident, $direct_channels:ident, $done:ident,) => {};
    }

    macro_rules! impl_handles_list {
        ( $head:ident, $( $tail:ident, )* ) => {
            #[async_trait]
            impl<A, $head, $( $tail ),*> HandlesList<($head, $( $tail ),*)> for A
            where
                A: 'static + Actor + Handler<$head>$( + Handler<$tail>)*,
                $head: Message, $( $tail: Message ),*
            {
                async fn setup_mailboxes(
                    self,
                    handle_channels: &HashMap<TypeId, Box<dyn BroadcastChannel>>,
                    publish_channels: &HashMap<TypeId, Box<dyn BroadcastChannel>>,
                    system_channel: Sender<SystemMessage>,
                ) -> Result<(HashMap<TypeId, Box<dyn DirectChannel>>, Vec<Receiver<()>>), YaafInternalError> {
                    let actor = Arc::new(Mutex::new(self));
                    let mut direct_channels: HashMap<TypeId, Box<dyn DirectChannel>> = HashMap::new();
                    let mut done = Vec::new();
                    start_mailbox!(
                        actor,
                        system_channel,
                        handle_channels,
                        publish_channels,
                        direct_channels,
                        done,
                        $head,
                        $( $tail, )*
                    );
                    Ok((direct_channels, done))
                }
            }

            impl_handles_list!($( $tail, )*);
        };
        () => {};
    }

    impl_handles_list!(M10, M9, M8, M7, M6, M5, M4, M3, M2, M1,);
}
