use crate::{
    actor::ActorId,
    handler::{HandleContext, Handler},
    message::Message,
    router::{ConcreteRouter, Router, SysRouter, SystemMessage},
};
use ::std::{any::TypeId, collections::HashMap, sync::Arc};
use ::tokio::{
    select, spawn,
    sync::mpsc::{channel, unbounded_channel, Receiver, Sender, UnboundedReceiver},
    sync::Mutex,
};

pub(crate) struct Mailbox<M: Message> {
    context: HandleContext,
    done: Sender<()>,
    handler: Arc<Mutex<dyn Handler<M>>>,
    recv: UnboundedReceiver<M>,
    sys_recv: UnboundedReceiver<SystemMessage>,
}

impl<M: Message> Mailbox<M> {
    pub(crate) async fn start(
        actor: Arc<Mutex<dyn Handler<M>>>,
        actor_id: ActorId,
        sys_router: &SysRouter,
        router: &ConcreteRouter<M>,
        publish_routers: HashMap<TypeId, Arc<dyn Router>>,
    ) -> Receiver<()> {
        let (sys_send, sys_recv) = unbounded_channel();
        sys_router.subscribe(Some(actor_id), sys_send).await;

        let (send, recv) = unbounded_channel();
        router.subscribe(Some(actor_id), send).await;
        let (done, result) = channel(1);

        let context = HandleContext::new(publish_routers);
        let mailbox = Mailbox {
            context,
            done,
            handler: actor,
            recv,
            sys_recv,
        };

        spawn(mailbox.run());
        result
    }

    async fn run(mut self) {
        // TODO: handle errors
        loop {
            select! {
                received = self.sys_recv.recv() => {
                    match received {
                        Some(message) => {
                            match message {
                                SystemMessage::Shutdown => {
                                    self.done.send(()).await.unwrap();
                                    break;
                                }
                            }
                        },
                        None => {},
                    }
                }
                received = self.recv.recv() => {
                    match received {
                        Some(message) => {
                            let mut guard = self.handler.lock().await;
                            guard.handle(&mut self.context, message).await;
                        },
                        None => {},
                    }
                }
            }
        }
    }
}
