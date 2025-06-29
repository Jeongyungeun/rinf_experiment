# Rinf에서 복잡한 Actor 모델 구현 방법

## 개요

Rinf에서 복잡한 모델을 Actor 시스템으로 관리할 때는 체계적인 접근 방식이 필요합니다. 이 문서에서는 복잡한 상태 관리, 반응형 프로그래밍, 그리고 앱 생명주기 동안 상태를 유지하는 방법에 대해 설명합니다.

## 1. 계층적 Actor 구조 설계

복잡한 시스템은 계층적 구조로 설계하여 관리하는 것이 효율적입니다:

```rust
// 최상위 감독자(Supervisor) Actor
struct AppSupervisor {
    user_manager: Address<UserManagerActor>,
    data_manager: Address<DataManagerActor>,
    network_manager: Address<NetworkManagerActor>,
}

// 중간 관리자 Actor들
struct UserManagerActor {
    auth_actor: Address<AuthActor>,
    profile_actors: HashMap<UserId, Address<UserProfileActor>>,
}

struct DataManagerActor {
    cache_actor: Address<CacheActor>,
    storage_actor: Address<StorageActor>,
}
```

## 2. 도메인별 Actor 모듈화

각 도메인별로 Actor를 모듈화하여 코드 구조를 명확하게 유지합니다:

```rust
// actors/mod.rs
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

pub async fn create_actors() -> Address<AppSupervisor> {
    // 계층적으로 Actor 생성
    let supervisor_context = Context::new();
    let supervisor_addr = supervisor_context.address();
    
    // 감독자 Actor 생성 및 실행
    let supervisor = AppSupervisor::new(supervisor_addr.clone());
    spawn(supervisor_context.run(supervisor));
    
    supervisor_addr
}
```

## 3. 메시지 타입 체계화

메시지 타입을 체계적으로 정의하여 Actor 간 통신을 명확하게 합니다:

```rust
// messages/mod.rs
mod auth_messages;
mod user_messages;
mod data_messages;

pub use auth_messages::{Login, Logout, VerifyToken};
pub use user_messages::{GetProfile, UpdateProfile, UserEvent};
pub use data_messages::{FetchData, StoreData, CacheData};
```

## 4. Actor 간 통신 패턴 구현

### 요청-응답 패턴

```rust
#[async_trait]
impl Handler<GetProfile> for UserProfileActor {
    type Response = Result<UserProfile, UserError>;
    
    async fn handle(&mut self, msg: GetProfile, _: &Context<Self>) -> Self::Response {
        // 프로필 데이터 처리 로직
        Ok(self.profile.clone())
    }
}
```

### 이벤트 발행-구독 패턴

```rust
#[async_trait]
impl Notifiable<UserEvent> for UserManagerActor {
    async fn notify(&mut self, event: UserEvent, _: &Context<Self>) {
        match event {
            UserEvent::ProfileUpdated(user_id, profile) => {
                // 프로필 업데이트 이벤트 처리
                if let Some(addr) = self.profile_actors.get(&user_id) {
                    let _ = addr.notify(UpdateProfileCache(profile)).await;
                }
                
                // Dart에 알림
                ProfileUpdatedSignal { user_id, profile }.send_signal_to_dart();
            },
            // 다른 이벤트 처리...
        }
    }
}
```

## 5. 복잡한 상태 관리를 위한 Actor 초기화

```rust
impl AppSupervisor {
    pub fn new(self_addr: Address<Self>) -> Self {
        // 1. 네트워크 관리자 생성
        let network_context = Context::new();
        let network_addr = network_context.address();
        let network_actor = NetworkManagerActor::new();
        spawn(network_context.run(network_actor));
        
        // 2. 데이터 관리자 생성 (네트워크 의존성 주입)
        let data_context = Context::new();
        let data_addr = data_context.address();
        let data_actor = DataManagerActor::new(network_addr.clone());
        spawn(data_context.run(data_actor));
        
        // 3. 사용자 관리자 생성 (데이터 및 네트워크 의존성 주입)
        let user_context = Context::new();
        let user_addr = user_context.address();
        let user_actor = UserManagerActor::new(data_addr.clone(), network_addr);
        spawn(user_context.run(user_actor));
        
        // 4. 감독자 구성
        Self {
            user_manager: user_addr,
            data_manager: data_addr,
            network_manager: network_addr,
        }
    }
}
```

## 6. 복잡한 워크플로우 조정

여러 Actor 간의 협업이 필요한 복잡한 워크플로우를 조정합니다:

```rust
// 로그인 프로세스 조정
#[async_trait]
impl Handler<ProcessLogin> for AppSupervisor {
    type Response = Result<UserSession, AuthError>;
    
    async fn handle(&mut self, msg: ProcessLogin, _: &Context<Self>) -> Self::Response {
        // 1. 인증 처리
        let auth_result = self.user_manager.send(Login {
            username: msg.username,
            password: msg.password,
        }).await??;
        
        // 2. 사용자 프로필 로드
        let profile = self.user_manager.send(GetProfile {
            user_id: auth_result.user_id,
        }).await??;
        
        // 3. 최근 데이터 로드
        let recent_data = self.data_manager.send(FetchRecentData {
            user_id: auth_result.user_id,
        }).await??;
        
        // 4. 세션 생성 및 반환
        Ok(UserSession {
            token: auth_result.token,
            profile,
            recent_data,
        })
    }
}
```

## 7. 오류 처리 및 복구 전략

Actor 실패 시 복구 전략을 구현합니다:

```rust
impl AppSupervisor {
    async fn handle_actor_failure(&mut self, actor_type: ActorType) {
        match actor_type {
            ActorType::Network => {
                debug_print!("Network actor failed, restarting...");
                // 네트워크 액터 재시작 로직
                let network_context = Context::new();
                let network_addr = network_context.address();
                let network_actor = NetworkManagerActor::new();
                spawn(network_context.run(network_actor));
                
                // 의존성 업데이트
                self.network_manager = network_addr;
                let _ = self.data_manager.notify(UpdateNetworkDependency(network_addr.clone())).await;
                let _ = self.user_manager.notify(UpdateNetworkDependency(network_addr)).await;
            },
            // 다른 액터 유형 처리...
        }
    }
}
```

## 8. 반응형 상태 관리

하나의 상태를 다른 상태가 구독하면서 반응형으로 변경되는 로직을 구현합니다:

```rust
// 상태 변경 알림용 메시지
struct StateChanged(i32);

#[async_trait]
impl Notifiable<UpdateState> for StateActor {
    async fn notify(&mut self, msg: UpdateState, _: &Context<Self>) {
        // 상태 업데이트
        self.value = msg.0;
        
        // 모든 구독자에게 알림
        for subscriber in &self.subscribers {
            let _ = subscriber.notify(StateChanged(self.value)).await;
        }
    }
}

#[async_trait]
impl Notifiable<StateChanged> for SubscriberActor {
    async fn notify(&mut self, msg: StateChanged, _: &Context<Self>) {
        // 원본 상태에 기반하여 파생 상태 업데이트
        self.derived_state = format!("Derived from: {}", msg.0);
        
        // 필요하다면 Dart에 신호 전송
        DerivedStateSignal { value: self.derived_state.clone() }.send_signal_to_dart();
    }
}
```

## 9. 앱 생명주기 동안 상태 유지

앱이 종료되어도 상태를 유지하기 위해 로컬 저장소와 동기화합니다:

```rust
struct PersistentStateActor {
    state: AppState,
    storage: Box<dyn Storage>,
}

impl PersistentStateActor {
    async fn new() -> Self {
        let storage = SledStorage::new("app_data").await;
        let state = match storage.load("app_state").await {
            Ok(data) => serde_json::from_slice(&data).unwrap_or_default(),
            Err(_) => AppState::default(),
        };
        
        Self { state, storage }
    }
    
    async fn save_state(&self) -> Result<(), StorageError> {
        let data = serde_json::to_vec(&self.state)?;
        self.storage.save("app_state", &data).await
    }
}

#[async_trait]
impl Notifiable<UpdateState> for PersistentStateActor {
    async fn notify(&mut self, msg: UpdateState, _: &Context<Self>) {
        self.state.update(msg);
        
        // 상태 변경 시 자동 저장
        if let Err(e) = self.save_state().await {
            debug_print!("Failed to save state: {:?}", e);
        }
        
        // 상태 변경 알림 전송
        StateChangedSignal { state: self.state.clone() }.send_signal_to_dart();
    }
}
```

## 10. Riverpod의 family 기능과 유사한 구현

Riverpod의 family 기능처럼 매개변수화된 상태 관리를 구현합니다:

```rust
// 매개변수화된 액터 레지스트리
struct ParameterizedActorRegistry<K: Eq + Hash + Clone + Send + 'static, A: Actor> {
    actors: HashMap<K, Address<A>>,
}

impl<K: Eq + Hash + Clone + Send + 'static, A: Actor> ParameterizedActorRegistry<K, A> {
    fn new() -> Self {
        Self {
            actors: HashMap::new(),
        }
    }
    
    // 특정 키에 대한 액터 주소 가져오기 (없으면 생성)
    async fn get_or_create<F>(&mut self, key: K, create_fn: F) -> Address<A>
    where
        F: FnOnce(K) -> A,
    {
        if let Some(addr) = self.actors.get(&key) {
            return addr.clone();
        }
        
        // 새 액터 생성
        let context = Context::new();
        let addr = context.address();
        let actor = create_fn(key.clone());
        
        // 액터 실행 및 저장
        spawn(context.run(actor));
        self.actors.insert(key, addr.clone());
        
        addr
    }
}

// 사용 예시
struct UserDataManager {
    registry: ParameterizedActorRegistry<String, UserDataActor>,
}

impl UserDataManager {
    async fn request_user_data(&mut self, user_id: String) {
        let addr = self.registry.get_or_create(
            user_id.clone(),
            |id| UserDataActor::new(id)
        ).await;
        
        let _ = addr.notify(FetchUserData).await;
    }
}
```

## 결론

Rinf의 Actor 모델은 복잡한 상태 관리, 반응형 프로그래밍, 그리고 앱 생명주기 동안의 상태 유지에 매우 적합합니다. 계층적 구조 설계, 도메인별 모듈화, 체계적인 메시지 타입 정의, 그리고 다양한 통신 패턴을 통해 확장 가능하고 유지보수하기 쉬운 시스템을 구축할 수 있습니다.
