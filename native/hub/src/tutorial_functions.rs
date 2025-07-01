// use std::time::Duration;

// use crate::signals::{MyAmazingNumber, MyPreciousData, MyTreasureInput, MyTreasureOutput};
// use rinf::{DartSignal, RustSignal, debug_print};
// use tokio::time::interval;

// pub async fn calculate_precious_data() {
//     let receiver = MyPreciousData::get_dart_signal_receiver();
//     while let Some(signal_pack) = receiver.recv().await {
//         let my_precious_data = signal_pack.message;
//         let new_numbers: Vec<i32> = my_precious_data
//             .input_numbers
//             .into_iter()
//             .map(|x| x + 1)
//             .collect();

//         let new_string = my_precious_data.input_string.to_uppercase();

//         debug_print!("{:?}", new_numbers);
//         debug_print!("{}", new_string);
//     }
// }

// pub async fn stream_amazing_number() {
//     let mut current_number: i32 = 1;
//     let mut time_interval = interval(Duration::from_secs(1));
//     loop {
//         time_interval.tick().await;
//         MyAmazingNumber { current_number }.send_signal_to_dart();
//         current_number += 1;
//     }
// }

// pub async fn tell_treasure() {
//     let mut current_value: i32 = 1;
//     let receiver = MyTreasureInput::get_dart_signal_receiver();
//     debug_print!("tell_treasure: waiting for signals...");
//     while let Some(_) = receiver.recv().await {
//         debug_print!("tell_treasure: received signal from Dart");
//         MyTreasureOutput { current_value }.send_signal_to_dart();
//         current_value += 1;
//     }
// }
