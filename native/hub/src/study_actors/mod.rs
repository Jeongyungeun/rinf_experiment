pub mod actors;
pub mod messages;
pub mod signals;
pub mod storage;

use messages::prelude::Address;
use rinf::debug_print;

use self::actors::AppSupervisor;

pub async fn initialize() {
    debug_print!("Initializing study_actors module...");
    
    // 액터 생성 함수 호출
    actors::create_actors().await;
    
    debug_print!("study_actors module initialized");
}
