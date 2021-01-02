use ::std::fmt::Debug;

pub trait Message: 'static + Clone + Debug + Send {}
impl<M> Message for M where M: 'static + Clone + Debug + Send {}

#[derive(Clone, Debug)]
pub enum SystemMessage {
    Shutdown,
}

pub(crate) mod detail {
    use super::*;
    use crate::{channel::BroadcastChannel, error::YaafInternalError};
    use ::async_trait::async_trait;
    use ::std::{any::TypeId, collections::HashMap};
    use ::tokio::sync::broadcast::{channel, Sender};

    #[async_trait]
    pub trait MessageList {
        async fn setup_channels(
            system_channel: Sender<SystemMessage>,
            broadcast_channels: &mut HashMap<TypeId, Box<dyn BroadcastChannel>>,
        ) -> Result<HashMap<TypeId, Box<dyn BroadcastChannel>>, YaafInternalError>;
        async fn setup_channels_impl(
            system_channel: Sender<SystemMessage>,
            broadcast_channels: &mut HashMap<TypeId, Box<dyn BroadcastChannel>>,
            result: HashMap<TypeId, Box<dyn BroadcastChannel>>,
        ) -> Result<HashMap<TypeId, Box<dyn BroadcastChannel>>, YaafInternalError>;
    }

    macro_rules! impl_message_list {
        ( $head:ident, $( $tail:ident, )* ) => {
            #[async_trait]
            impl<$head, $( $tail ),*> MessageList for ($head, $( $tail ),*)
            where
                $head: Message,
                $( $tail: Message),*
            {
                async fn setup_channels(
                    system_channel: Sender<SystemMessage>,
                    broadcast_channels: &mut HashMap<TypeId, Box<dyn BroadcastChannel>>,
                ) -> Result<HashMap<TypeId, Box<dyn BroadcastChannel>>, YaafInternalError> {
                    Self::setup_channels_impl(system_channel, broadcast_channels, HashMap::new()).await
                }

                async fn setup_channels_impl(
                    system_channel: Sender<SystemMessage>,
                    broadcast_channels: &mut HashMap<TypeId, Box<dyn BroadcastChannel>>,
                    mut result: HashMap<TypeId, Box<dyn BroadcastChannel>>,
                ) -> Result<HashMap<TypeId, Box<dyn BroadcastChannel>>, YaafInternalError> {
                    let type_id = TypeId::of::<$head>();
                    let r = broadcast_channels.entry(type_id).or_insert(Box::new(
                        channel::<$head>(1000).0
                    ));
                    result.insert(type_id, r.clone());
                    <($( $tail, )*) as MessageList>::setup_channels_impl(system_channel, broadcast_channels, result).await
                }
            }

            impl_message_list!($( $tail, )*);
        };
        () => {
            #[async_trait]
            impl MessageList for () {
                async fn setup_channels(
                    system_channel: Sender<SystemMessage>,
                    broadcast_channels: &mut HashMap<TypeId, Box<dyn BroadcastChannel>>,
                ) -> Result<HashMap<TypeId, Box<dyn BroadcastChannel>>, YaafInternalError> {
                    Self::setup_channels_impl(system_channel, broadcast_channels, HashMap::new()).await
                }

                async fn setup_channels_impl(
                    _system_channel: Sender<SystemMessage>,
                    _broadcast_channels: &mut HashMap<TypeId, Box<dyn BroadcastChannel>>,
                    result: HashMap<TypeId, Box<dyn BroadcastChannel>>,
                ) -> Result<HashMap<TypeId, Box<dyn BroadcastChannel>>, YaafInternalError> {
                    Ok(result)
                }
            }
        };
    }

    impl_message_list!(M10, M9, M8, M7, M6, M5, M4, M3, M2, M1,);
}
