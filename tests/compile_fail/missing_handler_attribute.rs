use ::yaaf::prelude::*;

#[derive(Clone, Debug)]
struct ValidMessage;

#[derive(Clone, Debug)]
struct InvalidMessage;

#[derive(Actor)]
#[handle(ValidMessage)]
struct MyActor;

#[async_trait]
impl Handler<ValidMessage> for MyActor {
    async fn handle(&mut self, _ctx: &mut Context<Self>, _message: ValidMessage) {
    }
}

#[async_trait]
impl Handler<InvalidMessage> for MyActor {
    async fn handle(&mut self, _ctx: &mut Context<Self>, _message: InvalidMessage) {
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn ::std::error::Error>> {
    let mut system = System::new();

    let actor = MyActor;

    let address = system.add_actor(actor).await?;

    address.tell(InvalidMessage)?;

    Ok(())
}
