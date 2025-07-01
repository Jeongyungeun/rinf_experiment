use async_trait::async_trait;
use messages::{
    actor::Actor,
    prelude::{Address, Context, Handler, Notifiable},
};
use rinf::{debug_print, RustSignal};
use tokio::task::JoinSet;

use crate::study_actors::{
    messages::{
        AuthError, AuthResult, FetchRecentData, GetProfile, Login, ProcessLogin, UserId, UserError,
        UserProfile,
    },
    signals::{AppInitializedSignal, InitializeAppRequest},
};

use super::{AuthActor, DataManagerActor, NetworkManagerActor, UserManagerActor};

// 액터 타입 열거형
pub enum ActorType {
    Auth,
    User,
    Data,
    Network,
}

// 사용자 세션 구조체
pub struct UserSession {
    pub token: String,
    pub profile: UserProfile,
    pub recent_data: crate::study_actors::messages::UserData,
}

// 앱 감독자 액터
pub struct AppSupervisor {
    user_manager: Address<UserManagerActor>,
    data_manager: Address<DataManagerActor>,
    network_manager: Address<NetworkManagerActor>,
    _owned_tasks: JoinSet<()>,
}

impl Actor for AppSupervisor {}

impl AppSupervisor {
    pub fn new(self_addr: Address<Self>, initialize_all: bool) -> Self {
        // 1. 네트워크 관리자 생성
        let network_context = Context::new();
        let network_addr = network_context.address();
        let network_actor = NetworkManagerActor::new();
        tokio::spawn(network_context.run(network_actor));
        
        // 2. 데이터 관리자 생성 (네트워크 의존성 주입)
        let data_context = Context::new();
        let data_addr = data_context.address();
        let data_actor = DataManagerActor::new(network_addr.clone());
        tokio::spawn(data_context.run(data_actor));
        
        // 3. 인증 액터 생성
        let auth_context = Context::new();
        let auth_addr = auth_context.address();
        let auth_actor = AuthActor::new(auth_addr.clone());
        tokio::spawn(auth_context.run(auth_actor));
        
        // 4. 사용자 관리자 생성 (인증 의존성 주입)
        let user_context = Context::new();
        let user_addr = user_context.address();
        let user_actor = UserManagerActor::new(auth_addr);
        tokio::spawn(user_context.run(user_actor));
        
        // 5. 감독자 구성
        let mut owned_tasks = JoinSet::new();
        
        if initialize_all {
            // 초기화 작업 시작
            owned_tasks.spawn(Self::initialize_system(self_addr.clone()));
        }
        
        Self {
            user_manager: user_addr,
            data_manager: data_addr,
            network_manager: network_addr,
            _owned_tasks: owned_tasks,
        }
    }
    
    async fn initialize_system(_self_addr: Address<Self>) {
        // 시스템 초기화 작업 (실제 구현에서는 필요한 초기화 수행)
        debug_print!("Initializing system...");
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        debug_print!("System initialized");
    }
    
    async fn handle_actor_failure(&mut self, actor_type: ActorType) {
        match actor_type {
            ActorType::Network => {
                debug_print!("Network actor failed, restarting...");
                // 네트워크 액터 재시작 로직
                let network_context = Context::new();
                let network_addr = network_context.address();
                let network_actor = NetworkManagerActor::new();
                tokio::spawn(network_context.run(network_actor));
                
                // 의존성 업데이트
                self.network_manager = network_addr.clone();
                let _ = self
                    .data_manager
                    .notify(crate::study_actors::messages::UpdateNetworkDependency(
                        network_addr,
                    ))
                    .await;
            }
            ActorType::Data => {
                debug_print!("Data actor failed, restarting...");
                // 데이터 액터 재시작 로직
                let data_context = Context::new();
                let data_addr = data_context.address();
                let data_actor = DataManagerActor::new(self.network_manager.clone());
                tokio::spawn(data_context.run(data_actor));
                
                // 의존성 업데이트
                self.data_manager = data_addr;
            }
            ActorType::User => {
                debug_print!("User actor failed, restarting...");
                // 사용자 액터 재시작 로직 (실제 구현에서는 AuthActor 주소 필요)
                // 여기서는 간단히 처리
                let user_context = Context::new();
                let user_addr = user_context.address();
                let user_actor = UserManagerActor::new(Address::<AuthActor>::default());
                tokio::spawn(user_context.run(user_actor));
                
                // 의존성 업데이트
                self.user_manager = user_addr;
            }
            ActorType::Auth => {
                debug_print!("Auth actor failed, cannot recover automatically");
                // 인증 액터는 중요해서 자동 복구 안함 (실제 구현에서는 더 복잡한 복구 전략 필요)
            }
        }
    }
}

#[async_trait]
impl Handler<ProcessLogin> for AppSupervisor {
    type Response = Result<UserSession, AuthError>;
    
    async fn handle(&mut self, msg: ProcessLogin, _: &Context<Self>) -> Self::Response {
        // 1. 인증 처리
        let auth_result = self
            .user_manager
            .send(Login {
                username: msg.username,
                password: msg.password,
            })
            .await??;
        
        // 2. 사용자 프로필 로드
        let profile = self
            .user_manager
            .send(GetProfile {
                user_id: auth_result.user_id.clone(),
            })
            .await??;
        
        // 3. 최근 데이터 로드
        let recent_data = self
            .data_manager
            .send(FetchRecentData {
                user_id: auth_result.user_id.clone(),
                limit: Some(5),
            })
            .await??;
        
        // 4. 세션 생성 및 반환
        Ok(UserSession {
            token: auth_result.token,
            profile,
            recent_data,
        })
    }
}

// Dart 신호 처리
#[async_trait]
impl Notifiable<InitializeAppRequest> for AppSupervisor {
    async fn notify(&mut self, msg: InitializeAppRequest, _: &Context<Self>) {
        debug_print!("Initializing app with reset_state={}", msg.reset_state);
        
        // 앱 초기화 로직 (실제 구현에서는 필요한 초기화 수행)
        let version = env!("CARGO_PKG_VERSION").to_string();
        let initialized_at = chrono::Utc::now().timestamp() as u64;
        
        // Dart에 초기화 완료 신호 전송
        AppInitializedSignal {
            success: true,
            version,
            initialized_at,
        }
        .send_signal_to_dart();
    }
}
