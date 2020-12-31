# Yet Another Actor Framework

A simple, local actor framework.

## Example

```rust
use ::std::time::Duration;
use ::tokio::time::sleep;
use ::yaaf::prelude::*;

#[derive(Clone)]
struct MyMessage;

#[derive(Actor)]
struct MyActor;

#[async_trait]
impl Handler<Communication> for Bob {
    async fn handle(&mut self, _ctx: &mut HandleContext, message: MyMessage) {
        println!("Received message");
    }
}

#[tokio::main]
async fn main() {
    let mut system = System::new().await;

    let actor = MyActor;

    let addr = system.add_actor(actor).await;
    addr.tell(MyMessage);

    sleep(Duration::from_millis(100));

    system.shutdown().await;
}
```