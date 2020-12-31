use yaaf::prelude::*;

#[derive(Clone, Debug)]
struct MyMessage;

#[derive(Actor)]
#[handle(MyMessage)]
struct MyActor;

fn main() {}
