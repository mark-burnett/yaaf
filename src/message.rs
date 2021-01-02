use ::std::fmt::Debug;

pub trait Message: 'static + Clone + Debug + Send {}
impl<M> Message for M where M: 'static + Clone + Debug + Send {}

#[derive(Clone, Debug)]
pub enum SystemMessage {
    Shutdown,
}

pub(crate) mod detail {
    use super::*;
    use crate::{error::YaafInternalError, router::Router};
    use ::async_trait::async_trait;
    use ::std::{any::TypeId, collections::HashMap};
    use ::tokio::sync::broadcast::{channel, Sender};

    #[async_trait]
    pub trait MessageList {
        async fn setup_routers(
            sys_router: Sender<SystemMessage>,
            existing_routers: &mut HashMap<TypeId, Box<dyn Router>>,
        ) -> Result<HashMap<TypeId, Box<dyn Router>>, YaafInternalError>;
        async fn setup_routers_impl(
            sys_router: Sender<SystemMessage>,
            existing_routers: &mut HashMap<TypeId, Box<dyn Router>>,
            result: HashMap<TypeId, Box<dyn Router>>,
        ) -> Result<HashMap<TypeId, Box<dyn Router>>, YaafInternalError>;
    }

    macro_rules! impl_message_list {
        ( $head:ident, $( $tail:ident, )* ) => {
            #[async_trait]
            impl<$head, $( $tail ),*> MessageList for ($head, $( $tail ),*)
            where
                $head: Message,
                $( $tail: Message),*
            {
                async fn setup_routers(
                    sys_router: Sender<SystemMessage>,
                    existing_routers: &mut HashMap<TypeId, Box<dyn Router>>,
                ) -> Result<HashMap<TypeId, Box<dyn Router>>, YaafInternalError> {
                    Self::setup_routers_impl(sys_router, existing_routers, HashMap::new()).await
                }

                async fn setup_routers_impl(
                    sys_router: Sender<SystemMessage>,
                    existing_routers: &mut HashMap<TypeId, Box<dyn Router>>,
                    mut result: HashMap<TypeId, Box<dyn Router>>,
                ) -> Result<HashMap<TypeId, Box<dyn Router>>, YaafInternalError> {
                    let type_id = TypeId::of::<$head>();
                    let r = existing_routers.entry(type_id).or_insert(Box::new(
                        channel::<$head>(1000).0
                    ));
                    result.insert(type_id, r.clone());
                    <($( $tail, )*) as MessageList>::setup_routers_impl(sys_router, existing_routers, result).await
                }
            }

            impl_message_list!($( $tail, )*);
        };
        () => {
            #[async_trait]
            impl MessageList for () {
                async fn setup_routers(
                    sys_router: Sender<SystemMessage>,
                    existing_routers: &mut HashMap<TypeId, Box<dyn Router>>,
                ) -> Result<HashMap<TypeId, Box<dyn Router>>, YaafInternalError> {
                    Self::setup_routers_impl(sys_router, existing_routers, HashMap::new()).await
                }

                async fn setup_routers_impl(
                    _sys_router: Sender<SystemMessage>,
                    _existing_routers: &mut HashMap<TypeId, Box<dyn Router>>,
                    result: HashMap<TypeId, Box<dyn Router>>,
                ) -> Result<HashMap<TypeId, Box<dyn Router>>, YaafInternalError> {
                    Ok(result)
                }
            }
        };
    }

    impl_message_list!(M10, M9, M8, M7, M6, M5, M4, M3, M2, M1,);
}
