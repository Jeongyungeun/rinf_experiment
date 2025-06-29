use rinf::{DartSignal, RustSignal, SignalPiece};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, DartSignal)]
pub struct MyPreciousData {
    pub input_numbers: Vec<i32>,
    pub input_string: String,
}

/// rust->dart
#[derive(Serialize, RustSignal)]
pub struct MyAmazingNumber {
    pub current_number: i32,
}

#[derive(Deserialize, DartSignal)]
pub struct MyTreasureInput {}

#[derive(Serialize, RustSignal)]
pub struct MyTreasureOutput {
    pub current_value: i32,
}
