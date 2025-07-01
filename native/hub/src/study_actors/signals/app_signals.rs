use rinf::{DartSignal, RustSignal};
use serde::{Deserialize, Serialize};

#[derive(DartSignal, Serialize, Deserialize, Debug)]
pub struct InitializeAppRequest {
    pub reset_state: bool,
}

#[derive(RustSignal, Serialize, Deserialize, Debug)]
pub struct AppInitializedSignal {
    pub success: bool,
    pub version: String,
    pub initialized_at: u64,
}

#[derive(DartSignal, Serialize, Deserialize, Debug)]
pub struct StateChangedSignal {
    pub state_type: String,
    pub state_json: String,
}

#[derive(DartSignal, Serialize, Deserialize, Debug)]
pub struct CreateActorsRequest {
    pub initialize_all: bool,
}

#[derive(RustSignal, Serialize, Deserialize, Debug)]
pub struct ActorsCreatedSignal {
    pub actor_count: usize,
    pub initialized_actors: Vec<String>,
}
