mod auth;
mod user;
mod data;
mod network;
mod supervisor;

pub use auth::AuthActor;
pub use user::{UserManagerActor, UserProfileActor};
pub use data::{DataManagerActor, CacheActor, StorageActor};
pub use network::NetworkManagerActor;
pub use supervisor::AppSupervisor;

use messages::prelude::{Address, Context};
use rinf::debug_print;
use tokio::spawn;

use crate::study_actors::signals::{ActorsCreatedSignal, CreateActorsRequest};

pub async fn create_actors() {
    // Dart 신호를 기다려 Actor 생성 시작
    let receiver = CreateActorsRequest::get_dart_signal_receiver();
    debug_print!("Waiting for CreateActorsRequest signal from Dart...");
    
    if let Some(signal_pack) = receiver.recv().await {
        let initialize_all = signal_pack.message.initialize_all;
        debug_print!("Received CreateActorsRequest: initialize_all={}", initialize_all);
        
        // 계층적으로 Actor 생성
        let supervisor_context = Context::new();
        let supervisor_addr = supervisor_context.address();
        
        // 감독자 Actor 생성 및 실행
        let supervisor = AppSupervisor::new(supervisor_addr.clone(), initialize_all);
        spawn(supervisor_context.run(supervisor));
        
        // Dart에 Actor 생성 완료 신호 전송
        ActorsCreatedSignal {
            actor_count: 5, // 실제 생성된 Actor 수
            initialized_actors: vec![
                "AppSupervisor".to_string(),
                "UserManagerActor".to_string(),
                "DataManagerActor".to_string(),
                "NetworkManagerActor".to_string(),
                "AuthActor".to_string(),
            ],
        }.send_signal_to_dart();
        
        debug_print!("Actors created and initialized successfully");
    }
}
