use ::async_trait::async_trait;
use ::std::sync::Arc;
use ::tokio::sync::{
    oneshot::{channel, Sender},
    Mutex,
};
use ::yaaf::prelude::*;

#[derive(Clone, Debug)]
struct Communication(String);

#[derive(Source)]
#[publish(Communication)]
struct Alice;

#[async_trait]
impl Source for Alice {
    async fn run(mut self, mut ctx: SourceContext) {
        ctx.publish::<Alice, Communication>(&self, Communication("hello".into()))
            .unwrap();
    }
}

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

#[async_trait]
impl Handler<Communication> for Bob {
    async fn handle(&mut self, _ctx: &mut HandleContext, message: Communication) {
        println!("Received Communication(\"{}\")", message.0);
        {
            let mut visited = self.visited.lock().await;
            *visited = true;
        }
        self.complete();
    }
}

#[tokio::test]
async fn simple_pubsub() -> Result<(), Box<dyn ::std::error::Error>> {
    let mut system = System::new().await?;

    let alice = Alice;

    let (send, recv) = channel();
    let visited = Arc::new(Mutex::new(false));
    let bob = Bob {
        done: Some(send),
        visited: visited.clone(),
    };

    system.add_actor(bob).await?;
    system.add_source(alice).await?;

    recv.await.unwrap();
    system.shutdown().await?;

    assert_eq!(true, *visited.lock().await);
    Ok(())
}
