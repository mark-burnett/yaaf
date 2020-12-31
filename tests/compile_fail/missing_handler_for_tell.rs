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

#[tokio::main]
async fn main() -> Result<(), Box<dyn ::std::error::Error>> {
    let mut system = System::new().await?;

    let actor = MyActor;

    let address = system.add_actor(actor).await?;

    address.tell(InvalidMessage)?;

    Ok(())
}
