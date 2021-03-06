use ::yaaf::prelude::*;

#[derive(Clone, Debug)]
struct MyMessage;

#[derive(Actor)]
struct MyActor;

#[async_trait]
impl Handler<MyMessage> for MyActor {
    async fn handle(&mut self, _ctx: &mut Context<Self>, _message: MyMessage) {
    }
}

fn main() {}
