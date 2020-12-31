# Yet Another Actor Framework

A simple, local actor framework.

## Example

```rust
use ::std::time::Duration;
use ::tokio::time::sleep;
use ::yaaf::prelude::*;

#[derive(Clone, Debug)]
struct MyMessage;

#[derive(Actor)]
struct MyActor;

#[async_trait]
impl Handler<Communication> for Bob {
    async fn handle(&mut self, _ctx: &mut Context, message: MyMessage) {
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

## License

This software is licensed under the Apache License, Version 2.0.

<hr>

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be licensed as above, without any additional terms or conditions.

<hr>
