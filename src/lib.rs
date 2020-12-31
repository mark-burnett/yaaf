#![forbid(unsafe_code)]

//! Yet Another Actor Framework
//!
//! A simple, local actor framework.
//!
//! # Key Features
//!
//! - Compile time checks on message publishing.
//! - Simple UX.
//!
//! ## Example
//!
//! ```rust
//! # use ::std::time::Duration;
//! # use ::tokio::time::sleep;
//! # use ::yaaf::prelude::*;
//! #
//! #[derive(Clone, Debug)]
//! struct Ping;
//! #[derive(Clone, Debug)]
//! struct Pong;
//!
//! // A long-lived Source that can publish events.
//! #[derive(Source)]
//! #[publish(Ping)]
//! struct Server;
//!
//! #[async_trait]
//! impl Source for Server {
//!     async fn run(mut self, mut ctx: Context<Self>) {
//!         // Broadcast a message to all handlers
//!         ctx.publish(Ping);
//!     }
//! }
//!
//! // An Actor that receives and sends events.
//! #[derive(Actor)]
//! #[handle(Ping)]
//! #[publish(Pong)]
//! struct Paddle;
//!
//! #[async_trait]
//! impl Handler<Ping> for Paddle {
//!     async fn handle(&mut self, ctx: &mut Context<Self>, _message: Ping) {
//!         // Broadcast a message to all handlers
//!         ctx.publish(Pong);
//!     }
//! }
//!
//! // An Actor that only receives events
//! #[derive(Actor)]
//! #[handle(Pong)]
//! struct Floor;
//!
//! #[async_trait]
//! impl Handler<Pong> for Floor {
//!     async fn handle(&mut self, _ctx: &mut Context<Self>, _message: Pong) {
//!         println!("Received message");
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn ::std::error::Error>> {
//!     let mut system = System::new().await?;
//!
//!     let server = Server;
//!     let paddle1 = Paddle;
//!     let paddle2 = Paddle;
//!     let floor = Floor;
//!
//!     system.add_source(server).await?;
//!
//!     let paddle_addr = system.add_actor(paddle1).await?;
//!     system.add_actor(paddle2).await?;
//!     let floor_addr = system.add_actor(floor).await?;
//!
//!     // Send messages directly to actors
//!     paddle_addr.tell(Ping);
//!     floor_addr.tell(Pong);
//!
//!     sleep(Duration::from_millis(50));
//!
//!     system.shutdown().await?;
//!     Ok(())
//! }
//! ```

mod actor;
mod context;
mod handler;
mod mailbox;
mod message;
mod publisher;
mod router;
mod source;
mod system;

pub mod error;
pub mod prelude;

pub use crate::actor::ActorAddress;
pub use crate::handler::HandlerRegistered;
pub use crate::message::Message;
#[doc(inline)]
pub use crate::prelude::*;
pub use crate::publisher::Publisher;
pub use crate::source::SourceMeta;
