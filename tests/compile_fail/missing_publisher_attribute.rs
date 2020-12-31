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
    async fn handle(&mut self, ctx: &mut HandleContext, _message: ValidMessage) {
        ctx.publish(self, InvalidMessage);
    }
}

fn main() {}
