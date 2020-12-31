use crate::actor::ActorId;
use crate::message::Message;
use ::dyn_clone::{clone_trait_object, DynClone};
use ::std::{any::Any, collections::HashMap};
use ::tokio::{
    select, spawn,
    sync::{
        mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
        oneshot,
    },
};

pub(crate) enum SubscriptionMessage<M: Message> {
    Subscribe((Option<ActorId>, UnboundedSender<M>, oneshot::Sender<()>)),
}

#[derive(Clone)]
pub enum SystemMessage {
    Shutdown,
}

#[derive(Clone)]
pub(crate) enum DistributionType {
    Broadcast,
    Direct(ActorId),
}

#[derive(Clone)]
pub(crate) struct Envelope<M>
where
    M: Message,
{
    distribution_type: DistributionType,
    message: M,
}

#[derive(Clone)]
pub struct ConcreteRouter<M>
where
    M: 'static + Message,
{
    send: UnboundedSender<Envelope<M>>,
    sub_send: UnboundedSender<SubscriptionMessage<M>>,
}

pub(crate) type SysRouter = ConcreteRouter<SystemMessage>;

pub trait Router: Any + DynClone + Send + Sync {
    fn as_any(&self) -> &dyn Any;
}
impl<M: 'static + Message> Router for ConcreteRouter<M> {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
clone_trait_object!(Router);

impl<M: Message> ConcreteRouter<M> {
    pub(crate) async fn new_system_router() -> SysRouter {
        let (send, recv) = unbounded_channel();
        let (sub_send, sub_recv) = unbounded_channel();
        let (sys_send, sys_recv) = unbounded_channel();

        let sys_router = SysRouter { send, sub_send };

        let sub_future = sys_router.subscribe(None, sys_send);

        let router_impl = RouterImpl {
            anonymous_subscribers: Vec::new(),
            recv,
            sub_recv,
            subscribers: HashMap::new(),
            sys_recv,
        };
        spawn(router_impl.run());
        sub_future.await;

        sys_router
    }

    pub(crate) async fn new(sys_router: &SysRouter) -> Self {
        let (send, recv) = unbounded_channel();
        let (sub_send, sub_recv) = unbounded_channel();
        let (sys_send, sys_recv) = unbounded_channel();

        sys_router.subscribe(None, sys_send).await;

        let router_impl = RouterImpl {
            anonymous_subscribers: Vec::new(),
            recv,
            subscribers: HashMap::new(),
            sub_recv,
            sys_recv,
        };
        spawn(router_impl.run());

        Self { send, sub_send }
    }

    pub(crate) fn broadcast(&self, message: M) {
        self.send.send(Envelope {
            distribution_type: DistributionType::Broadcast,
            message: message,
        });
    }

    // TODO: add error here.
    pub(crate) fn tell(&self, recipient: ActorId, message: M) {
        self.send.send(Envelope {
            distribution_type: DistributionType::Direct(recipient),
            message: message,
        });
    }

    // TODO: add error
    pub(crate) async fn subscribe(&self, recipient: Option<ActorId>, mailbox: UnboundedSender<M>) {
        let (s, r) = oneshot::channel();
        self.sub_send
            .send(SubscriptionMessage::Subscribe((recipient, mailbox, s)));
        r.await.unwrap();
    }
}

pub(crate) struct RouterImpl<M>
where
    M: Message,
{
    anonymous_subscribers: Vec<UnboundedSender<M>>,
    recv: UnboundedReceiver<Envelope<M>>,
    sub_recv: UnboundedReceiver<SubscriptionMessage<M>>,
    subscribers: HashMap<ActorId, UnboundedSender<M>>,
    sys_recv: UnboundedReceiver<SystemMessage>,
}

impl<M> RouterImpl<M>
where
    M: Message,
{
    pub(crate) async fn run(mut self) {
        loop {
            select! {
                received = self.sys_recv.recv() => {
                    match received {
                        Some(message) => {
                            match message {
                                SystemMessage::Shutdown => break,
                            }
                        },
                        None => {},
                    }
                },
                receive = self.sub_recv.recv() => {
                    match receive {
                        Some(message) => {
                            match message {
                                SubscriptionMessage::Subscribe((recipient, send, done)) => {
                                    match recipient {
                                        Some(recipient_id) => {
                                            self.subscribers.insert(recipient_id, send);
                                        },
                                        None => self.anonymous_subscribers.push(send),
                                    }
                                    done.send(()).unwrap();
                                },
                            }
                        },
                        None => {}
                    }
                },
                received = self.recv.recv() => {
                    match received {
                        Some(envelope) => {
                            match envelope.distribution_type {
                                DistributionType::Broadcast => {
                                    for (_recipient_id, send) in &self.subscribers {
                                        // TODO: handle errors
                                        send.send(envelope.message.clone());
                                    }
                                    for send in &self.anonymous_subscribers {
                                        send.send(envelope.message.clone());
                                    }
                                }
                                DistributionType::Direct(recipient_id) => {
                                    self.subscribers.get(&recipient_id).unwrap().send(envelope.message);
                                }
                            }
                        },
                        None => {},
                    }
                },
            }
        }
    }
}
