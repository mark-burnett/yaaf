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
    visited: Arc<Mutex<i32>>,
}

#[async_trait]
impl Handler<Communication> for MyActor {
    async fn handle(&mut self, _ctx: &mut Context<Self>, message: Communication) {
        println!("Received Communication(\"{}\")", message.0);
        {
            let mut visited = self.visited.lock().await;
            *visited += 1;
        }
        self.done.send(()).await.unwrap();
    }
}

#[tokio::test]
async fn cloned_tell() -> Result<(), Box<dyn ::std::error::Error>> {
    let mut system = System::new().await?;

    let (send, mut recv) = channel(1);
    let visited = Arc::new(Mutex::new(0));
    let actor = MyActor {
        done: send,
        visited: visited.clone(),
    };

    let address = system.add_actor(actor).await?;
    let cloned_address = address.clone();

    address.tell(Communication("Hello".into()))?;
    cloned_address.tell(Communication("Hello".into()))?;

    recv.recv().await.unwrap();
    recv.recv().await.unwrap();
    system.shutdown().await?;

    assert_eq!(2, *visited.lock().await);

    Ok(())
}
