// use std::time::Duration;

// use crate::signals::{BigBool, SmallNumber, SmallText};
// use async_trait::async_trait;
// use messages::prelude::{Actor, Address, Context, Handler, Notifiable};
// use rinf::{DartSignal, RustSignal, debug_print};
// use tokio::task::JoinSet;
// use tokio::time::interval;

// // Uncomment below to target the web.
// // use tokio_with_wasm::alias as tokio;

// /// The first actor.
// pub struct FirstActor {
//     /// Owned tasks that are canceled when the actor is dropped.
//     _owned_tasks: JoinSet<()>,
// }

// // Implementing the `Actor` trait for `CountingActor`.
// // This defines `FirstActor` as an actor in the async system.
// impl Actor for FirstActor {}

// impl FirstActor {
//     /// Creates the actor and initializes its fields.
//     pub fn new(self_addr: Address<Self>) -> Self {
//         let mut _owned_tasks = JoinSet::new();
//         _owned_tasks.spawn(Self::listen_to_dart(self_addr.clone()));
//         _owned_tasks.spawn(Self::listen_to_timer(self_addr));
//         FirstActor { _owned_tasks }
//     }
// }

// // Implementing the `Notifiable` trait
// // allows an actor's loop to listen for a specific message type.
// #[async_trait]
// impl Notifiable<SmallText> for FirstActor {
//     async fn notify(&mut self, msg: SmallText, _: &Context<Self>) {
//         debug_print!("{}", msg.text);
//         SmallNumber { number: 7 }.send_signal_to_dart();
//     }
// }

// // Implementing the `Handler` trait
// // allows an actor's loop to respond to a specific message type.
// #[async_trait]
// impl Handler<BigBool> for FirstActor {
//     type Result = bool;
//     async fn handle(&mut self, msg: BigBool, _: &Context<Self>) -> bool {
//         msg.send_signal_to_dart();
//         false
//     }
// }

// impl FirstActor {
//     /// Listen to an external source, which in this case is Dart.
//     async fn listen_to_dart(mut self_addr: Address<Self>) {
//         let receiver = SmallText::get_dart_signal_receiver();
//         while let Some(signal_pack) = receiver.recv().await {
//             let _ = self_addr.notify(signal_pack.message).await;
//         }
//     }

//     /// Listen to an external source, which in this case is a timer.
//     async fn listen_to_timer(mut self_addr: Address<Self>) {
//         let mut time_interval = interval(Duration::from_secs(3));
//         let text = "From an owned task".to_owned();
//         loop {
//             time_interval.tick().await;
//             let _ = self_addr.notify(SmallText { text: text.clone() }).await;
//         }
//     }
// }

use crate::signals::{SampleNumberInput, SampleNumberOutput};
use async_trait::async_trait;
use messages::{
    actor::Actor,
    prelude::{Address, Context, Notifiable},
};
use rinf::{DartSignal, RustSignal, debug_print};
use tokio::task::JoinSet;

pub struct CountingActor {
    count: i32,
    _owned_tasks: JoinSet<()>,
}

impl Actor for CountingActor {}

impl CountingActor {
    pub fn new(self_addr: Address<Self>) -> Self {
        let mut owned_tasks = JoinSet::new();
        owned_tasks.spawn(Self::listen_to_button_click(self_addr));
        CountingActor {
            count: 0,
            _owned_tasks: owned_tasks,
        }
    }

    async fn listen_to_button_click(mut self_addr: Address<Self>) {
        let receiver = SampleNumberInput::get_dart_signal_receiver();
        while let Some(signal_pack) = receiver.recv().await {
            let message = signal_pack.message;
            let _ = self_addr.notify(message).await;
        }
    }
}

#[async_trait]
impl Notifiable<SampleNumberInput> for CountingActor {
    async fn notify(&mut self, msg: SampleNumberInput, _: &Context<Self>) {
        debug_print!("{}", msg.letter);
        self.count += 7;

        SampleNumberOutput {
            current_number: self.count,
            dummy_one: 11,
            dummy_two: None,
            dummy_three: vec![22, 33, 44, 55],
        }
        .send_signal_to_dart();
    }
}
