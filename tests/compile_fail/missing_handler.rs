use yaaf::prelude::*;

#[derive(Clone, Debug)]
struct Communication(String);

#[derive(Actor)]
#[handle(Communication)]
struct Bob;

fn main() {}
