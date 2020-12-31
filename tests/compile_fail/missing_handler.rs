use ::async_trait::async_trait;
use ::std::sync::Arc;
use ::tokio::sync::{
    oneshot::{channel, Sender},
    Mutex,
};
use ::yaaf::prelude::*;

#[derive(Clone)]
struct Communication(String);

#[derive(Actor)]
#[handle(Communication)]
struct Bob {
    done: Option<Sender<()>>,
    visited: Arc<Mutex<bool>>,
}

impl Bob {
    fn complete(&mut self) {
        let done = self.done.take();
        if let Some(done) = done {
            done.send(()).unwrap();
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn ::std::error::Error>> {
    let mut system = System::new().await;

    let (send, recv) = channel();
    let visited = Arc::new(Mutex::new(false));
    let bob = Bob {
        done: Some(send),
        visited: visited.clone(),
    };

    let bob_addr = system.add_actor(bob).await;

    bob_addr.tell(Communication("Hello".into()));

    recv.await.unwrap();
    system.shutdown().await;

    assert_eq!(true, *visited.lock().await);

    Ok(())
}
