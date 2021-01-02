use crate::{
    actor::Actor,
    context::Context,
    error::YaafInternalError,
    handler::Handler,
    message::Message,
    router::{Router, SystemMessage},
};
use ::std::{any::TypeId, collections::HashMap, sync::Arc};
use ::tokio::{
    select, spawn,
    sync::{broadcast, mpsc, Mutex},
};

pub(crate) struct Mailbox<A: Actor + Handler<M>, M: Message> {
    context: Context<A>,
    done: mpsc::Sender<()>,
    handler: Arc<Mutex<A>>,
    recv_broadcast: broadcast::Receiver<M>,
    recv_system: broadcast::Receiver<SystemMessage>,
    recv_tell: mpsc::UnboundedReceiver<M>,
}

impl<A: 'static + Actor + Handler<M>, M: Message> Mailbox<A, M> {
    pub(crate) async fn start(
        actor: Arc<Mutex<A>>,
        recv_broadcast: broadcast::Receiver<M>,
        recv_system: broadcast::Receiver<SystemMessage>,
        publish_routers: HashMap<TypeId, Box<dyn Router>>,
    ) -> Result<(mpsc::UnboundedSender<M>, mpsc::Receiver<()>), YaafInternalError> {
        let (done, result) = mpsc::channel(1);
        let (send_tell, recv_tell) = mpsc::unbounded_channel();

        let context = Context::new(publish_routers);
        let mailbox = Mailbox {
            context,
            done,
            handler: actor,
            recv_broadcast,
            recv_system,
            recv_tell,
        };

        spawn(mailbox.run());
        Ok((send_tell, result))
    }

    async fn run(mut self) {
        loop {
            select! {
                received = self.recv_system.recv() => {
                    if let Ok(message) = received {
                        match message {
                            SystemMessage::Shutdown => {
                                // TODO: log the error
                                let _ = self.done.send(()).await;
                                break;
                            }
                        }
                    }
                }
                received = self.recv_tell.recv() => {
                    if let Some(message) = received {
                        let mut guard = self.handler.lock().await;
                        guard.handle(&mut self.context, message).await;
                    }
                }
                received = self.recv_broadcast.recv() => {
                    if let Ok(message) = received {
                        let mut guard = self.handler.lock().await;
                        guard.handle(&mut self.context, message).await;
                    }
                }
            }
        }
    }
}
