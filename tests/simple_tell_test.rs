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
struct Bob {
    done: Sender<()>,
    visited: Arc<Mutex<bool>>,
}

#[async_trait]
impl Handler<Communication> for Bob {
    async fn handle(&mut self, _ctx: &mut HandleContext, message: Communication) {
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
    let mut system = System::new().await;

    let (send, mut recv) = channel(1);
    let visited = Arc::new(Mutex::new(false));
    let bob = Bob {
        done: send,
        visited: visited.clone(),
    };

    let bob_addr = system.add_actor(bob).await;
    let cloned_address = bob_addr.clone();

    bob_addr.tell(Communication("Hello".into()));
    cloned_address.tell(Communication("Hello".into()));

    recv.recv().await;
    recv.recv().await;
    system.shutdown().await;

    assert_eq!(true, *visited.lock().await);

    Ok(())
}
