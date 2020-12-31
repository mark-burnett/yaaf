use ::std::sync::Arc;
use ::tokio::sync::{
    mpsc::{channel, Sender},
    Mutex,
};
use ::yaaf::prelude::*;

#[derive(Clone, Debug)]
struct MessageA;

#[derive(Clone, Debug)]
struct MessageB;

#[derive(Actor)]
#[handle(MessageA, MessageB)]
struct MyActor {
    done: Sender<()>,
    got_a: Arc<Mutex<bool>>,
    got_b: Arc<Mutex<bool>>,
}

impl MyActor {
    async fn is_done(&self) -> bool {
        let got_a = self.got_a.lock().await;
        let got_b = self.got_b.lock().await;
        (*got_a) && (*got_b)
    }
}

#[async_trait]
impl Handler<MessageA> for MyActor {
    async fn handle(&mut self, _ctx: &mut Context<Self>, _message: MessageA) {
        {
            let mut got_a = self.got_a.lock().await;
            *got_a = true;
        }
        if self.is_done().await {
            self.done.send(()).await.unwrap();
        }
    }
}

#[async_trait]
impl Handler<MessageB> for MyActor {
    async fn handle(&mut self, _ctx: &mut Context<Self>, _message: MessageB) {
        println!("received B");
        {
            let mut got_b = self.got_b.lock().await;
            *got_b = true;
        }
        if self.is_done().await {
            self.done.send(()).await.unwrap();
        }
    }
}

#[tokio::test]
async fn multiple_handler() -> Result<(), Box<dyn ::std::error::Error>> {
    let mut system = System::new().await?;

    let (send, mut recv) = channel(1);
    let got_a = Arc::new(Mutex::new(false));
    let got_b = Arc::new(Mutex::new(false));
    let actor = MyActor {
        done: send,
        got_a: got_a.clone(),
        got_b: got_b.clone(),
    };

    let address = system.add_actor(actor).await?;

    address.tell(MessageA)?;
    address.tell(MessageB)?;

    recv.recv().await.unwrap();
    system.shutdown().await?;

    assert_eq!(true, *got_a.lock().await);
    assert_eq!(true, *got_b.lock().await);

    Ok(())
}
