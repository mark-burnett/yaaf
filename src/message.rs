use ::std::fmt::Debug;

pub trait Message: 'static + Clone + Debug + Send {}
impl<M> Message for M where M: 'static + Clone + Debug + Send {}

pub(crate) mod detail {
    use super::*;
    use crate::{
        error::YaafInternalError,
        router::{ConcreteRouter, Router, SysRouter},
    };
    use ::async_trait::async_trait;
    use ::std::{any::TypeId, collections::HashMap, sync::Arc};

    #[async_trait]
    pub trait MessageList {
        async fn setup_routers(
            sys_router: &SysRouter,
            existing_routers: &mut HashMap<TypeId, Arc<dyn Router>>,
        ) -> Result<HashMap<TypeId, Arc<dyn Router>>, YaafInternalError>;
        async fn setup_routers_impl(
            sys_router: &SysRouter,
            existing_routers: &mut HashMap<TypeId, Arc<dyn Router>>,
            result: HashMap<TypeId, Arc<dyn Router>>,
        ) -> Result<HashMap<TypeId, Arc<dyn Router>>, YaafInternalError>;
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
                    sys_router: &SysRouter,
                    existing_routers: &mut HashMap<TypeId, Arc<dyn Router>>,
                ) -> Result<HashMap<TypeId, Arc<dyn Router>>, YaafInternalError> {
                    Self::setup_routers_impl(sys_router, existing_routers, HashMap::new()).await
                }

                async fn setup_routers_impl(
                    sys_router: &SysRouter,
                    existing_routers: &mut HashMap<TypeId, Arc<dyn Router>>,
                    mut result: HashMap<TypeId, Arc<dyn Router>>,
                ) -> Result<HashMap<TypeId, Arc<dyn Router>>, YaafInternalError> {
                    let type_id = TypeId::of::<$head>();
                    if ! existing_routers.contains_key(&type_id) {
                        existing_routers.insert(type_id, Arc::new(ConcreteRouter::<$head>::new(sys_router).await?));
                    }
                    let r = existing_routers.get(&type_id).ok_or(YaafInternalError::RouterLookupFailure)?;
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
                    sys_router: &SysRouter,
                    existing_routers: &mut HashMap<TypeId, Arc<dyn Router>>,
                ) -> Result<HashMap<TypeId, Arc<dyn Router>>, YaafInternalError> {
                    Self::setup_routers_impl(sys_router, existing_routers, HashMap::new()).await
                }

                async fn setup_routers_impl(
                    _sys_router: &SysRouter,
                    _existing_routers: &mut HashMap<TypeId, Arc<dyn Router>>,
                    result: HashMap<TypeId, Arc<dyn Router>>,
                ) -> Result<HashMap<TypeId, Arc<dyn Router>>, YaafInternalError> {
                    Ok(result)
                }
            }
        };
    }

    impl_message_list!(M10, M9, M8, M7, M6, M5, M4, M3, M2, M1,);
}
