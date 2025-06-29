//! This `hub` crate is the
//! entry point of the Rust logic.

mod actors;
mod signals;
mod tutorial_functions;

use async_trait::async_trait;
use messages::{
    actor::Actor,
    prelude::{Address, Context, Notifiable},
};
// use actors::create_actors;
use rinf::{dart_shutdown, debug_print, write_interface};
use tokio::spawn;

use crate::tutorial_functions::{calculate_precious_data, stream_amazing_number, tell_treasure};

// Uncomment below to target the web.
// use tokio_with_wasm::alias as tokio;

write_interface!();

struct Sum(usize, usize);

struct MyActor {
    count: i32,
}

impl Actor for MyActor {}

impl MyActor {
    pub fn new() -> Self {
        Self { count: 0 }
    }
}

#[async_trait]
impl Notifiable<Sum> for MyActor {
    async fn notify(&mut self, msg: Sum, _: &Context<Self>) {
        self.count += 1;
        debug_print!("{}:{}", msg.0 + msg.1, self.count);
    }
}

fn create_actors() -> Address<MyActor> {
    let context = Context::new();
    let addr = context.address();
    let actor = MyActor::new();
    spawn(context.run(actor));
    addr
}

// You can go with any async library, not just `tokio`.
#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Spawn concurrent tasks.
    // Always use non-blocking async functions like `tokio::fs::File::open`.
    // If you must use blocking code, use `tokio::task::spawn_blocking`
    // or the equivalent provided by your async library.
    // spawn(calculate_precious_data());
    // spawn(stream_amazing_number());
    // spawn(tell_treasure());
    let mut addr = create_actors();
    let _ = addr.notify(Sum(10, 5)).await;
    dart_shutdown();

    // Keep the main function running until Dart shutdown.
    dart_shutdown().await;
}
