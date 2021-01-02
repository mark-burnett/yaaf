use ::async_trait::async_trait;
use ::std::sync::Arc;
use ::tokio::sync::{
    mpsc::{channel, Sender},
    Mutex,
};
use ::yaaf::prelude::*;

#[derive(Clone, Debug)]
struct Communication(String);

#[derive(Actor)]
#[handle(Communication)]
struct MyActor {
    done: Sender<()>,
    visited: Arc<Mutex<bool>>,
}

#[async_trait]
impl Handler<Communication> for MyActor {
    async fn handle(&mut self, _ctx: &mut Context<Self>, message: Communication) {
        println!("Received Communication(\"{}\")", message.0);
        {
            let mut visited = self.visited.lock().await;
            *visited = true;
        }
        self.done.send(()).await.unwrap();
    }
}

#[tokio::test]
async fn simple_tell() -> Result<(), Box<dyn ::std::error::Error>> {
    let mut system = System::new();

    let (send, mut recv) = channel(1);
    let visited = Arc::new(Mutex::new(false));
    let actor = MyActor {
        done: send,
        visited: visited.clone(),
    };

    let address = system.add_actor(actor).await?;

    address.tell(Communication("Hello".into()))?;

    recv.recv().await.unwrap();
    system.shutdown().await?;

    assert_eq!(true, *visited.lock().await);

    Ok(())
}
