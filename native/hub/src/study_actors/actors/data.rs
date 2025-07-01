use async_trait::async_trait;
use chrono::Utc;
use messages::{
    actor::Actor,
    prelude::{Address, Context, Handler, Notifiable},
};
use rinf::{RustSignal, debug_print};
use std::collections::HashMap;
use tokio::task::JoinSet;

use crate::study_actors::{
    messages::{
        CacheData, DataItem, FetchData, FetchRecentData, StoreData, UpdateNetworkDependency,
        UserData, UserError, UserId,
    },
    signals::{
        CreateDataItemRequest, DataItemCreatedSignal, DataItemDeletedSignal, DataItemUpdatedSignal,
        DeleteDataItemRequest, FetchUserDataRequest, UpdateDataItemRequest, UserDataResponse,
    },
};

use super::NetworkManagerActor;

// 데이터 관리자 액터
pub struct DataManagerActor {
    cache_actor: Address<CacheActor>,
    storage_actor: Address<StorageActor>,
    network_manager: Address<NetworkManagerActor>,
    _owned_tasks: JoinSet<()>,
}

impl Actor for DataManagerActor {}

impl DataManagerActor {
    pub fn new(network_manager: Address<NetworkManagerActor>) -> Self {
        // 캐시 액터 생성
        let cache_context = Context::new();
        let cache_addr = cache_context.address();
        let cache_actor = CacheActor::new();
        tokio::spawn(cache_context.run(cache_actor));

        // 저장소 액터 생성
        let storage_context = Context::new();
        let storage_addr = storage_context.address();
        let storage_actor = StorageActor::new();
        tokio::spawn(storage_context.run(storage_actor));

        Self {
            cache_actor: cache_addr,
            storage_actor: storage_addr,
            network_manager,
            _owned_tasks: JoinSet::new(),
        }
    }

    fn generate_item_id(&self) -> String {
        format!("item_{}", Utc::now().timestamp_millis())
    }
}

#[async_trait]
impl Handler<FetchData> for DataManagerActor {
    type Response = Result<Vec<u8>, UserError>;

    async fn handle(&mut self, msg: FetchData, _: &Context<Self>) -> Self::Response {
        // 1. 먼저 캐시에서 확인
        let cache_result = self.cache_actor.send(msg.clone()).await;

        if let Ok(Ok(data)) = cache_result {
            debug_print!("Cache hit for key: {}", msg.key);
            return Ok(data);
        }

        // 2. 캐시에 없으면 저장소에서 확인
        let storage_result = self.storage_actor.send(msg.clone()).await;

        if let Ok(Ok(data)) = storage_result {
            debug_print!("Storage hit for key: {}", msg.key);

            // 캐시에 저장
            let _ = self
                .cache_actor
                .send(CacheData {
                    key: msg.key,
                    data: data.clone(),
                    ttl: Some(3600), // 1시간 캐시
                })
                .await;

            return Ok(data);
        }

        // 3. 저장소에도 없으면 네트워크에서 가져오기 (실제 구현에서는 필요)
        Err("Data not found".into())
    }
}

#[async_trait]
impl Handler<StoreData> for DataManagerActor {
    type Response = Result<(), UserError>;

    async fn handle(&mut self, msg: StoreData, _: &Context<Self>) -> Self::Response {
        // 1. 저장소에 저장
        let storage_result = self.storage_actor.send(msg.clone()).await??;

        // 2. 캐시에도 저장
        let _ = self
            .cache_actor
            .send(CacheData {
                key: msg.key,
                data: msg.data,
                ttl: msg.ttl,
            })
            .await;

        Ok(storage_result)
    }
}

#[async_trait]
impl Handler<FetchRecentData> for DataManagerActor {
    type Response = Result<UserData, UserError>;

    async fn handle(&mut self, msg: FetchRecentData, _: &Context<Self>) -> Self::Response {
        // 실제 구현에서는 저장소에서 사용자의 최근 데이터 가져오기
        let limit = msg.limit.unwrap_or(10);

        // 예시 데이터 생성
        let items = (0..limit)
            .map(|i| DataItem {
                id: format!("item_{}", i),
                title: format!("Item {}", i),
                content: format!("Content for item {}", i),
                created_at: Utc::now().timestamp() as u64 - i as u64 * 3600,
                updated_at: Utc::now().timestamp() as u64 - i as u64 * 1800,
            })
            .collect();

        let user_data = UserData {
            user_id: msg.user_id,
            items,
            last_updated: Utc::now().timestamp() as u64,
        };

        Ok(user_data)
    }
}

#[async_trait]
impl Notifiable<UpdateNetworkDependency> for DataManagerActor {
    async fn notify(&mut self, msg: UpdateNetworkDependency, _: &Context<Self>) {
        debug_print!("Updating network dependency for DataManagerActor");
        self.network_manager = msg.0;
    }
}

// Dart 신호 처리
#[async_trait]
impl Notifiable<FetchUserDataRequest> for DataManagerActor {
    async fn notify(&mut self, msg: FetchUserDataRequest, ctx: &Context<Self>) {
        let data_result = self
            .handle(
                FetchRecentData {
                    user_id: msg.user_id,
                    limit: msg.limit,
                },
                ctx,
            )
            .await;

        match data_result {
            Ok(user_data) => {
                UserDataResponse {
                    user_id: user_data.user_id,
                    items: user_data.items,
                    last_updated: user_data.last_updated,
                    error: None,
                }
                .send_signal_to_dart();
            }
            Err(e) => {
                UserDataResponse {
                    user_id: msg.user_id,
                    items: vec![],
                    last_updated: 0,
                    error: Some(e.to_string()),
                }
                .send_signal_to_dart();
            }
        }
    }
}

#[async_trait]
impl Notifiable<CreateDataItemRequest> for DataManagerActor {
    async fn notify(&mut self, msg: CreateDataItemRequest, _: &Context<Self>) {
        let now = Utc::now().timestamp() as u64;
        let item = DataItem {
            id: self.generate_item_id(),
            title: msg.title,
            content: msg.content,
            created_at: now,
            updated_at: now,
        };

        // 실제 구현에서는 저장소에 저장

        // Dart에 알림
        DataItemCreatedSignal {
            user_id: msg.user_id,
            item,
        }
        .send_signal_to_dart();
    }
}

#[async_trait]
impl Notifiable<UpdateDataItemRequest> for DataManagerActor {
    async fn notify(&mut self, msg: UpdateDataItemRequest, _: &Context<Self>) {
        // 실제 구현에서는 저장소에서 아이템 가져와서 업데이트
        let now = Utc::now().timestamp() as u64;
        let item = DataItem {
            id: msg.item_id.clone(),
            title: msg.title.unwrap_or_else(|| "Updated Item".to_string()),
            content: msg.content.unwrap_or_else(|| "Updated content".to_string()),
            created_at: now - 3600, // 예시용
            updated_at: now,
        };

        // Dart에 알림
        DataItemUpdatedSignal {
            user_id: msg.user_id,
            item,
        }
        .send_signal_to_dart();
    }
}

#[async_trait]
impl Notifiable<DeleteDataItemRequest> for DataManagerActor {
    async fn notify(&mut self, msg: DeleteDataItemRequest, _: &Context<Self>) {
        // 실제 구현에서는 저장소에서 아이템 삭제

        // Dart에 알림
        DataItemDeletedSignal {
            user_id: msg.user_id,
            item_id: msg.item_id,
        }
        .send_signal_to_dart();
    }
}

// 캐시 액터
pub struct CacheActor {
    cache: HashMap<String, CacheEntry>,
    _owned_tasks: JoinSet<()>,
}

struct CacheEntry {
    data: Vec<u8>,
    expires_at: Option<u64>,
}

impl Actor for CacheActor {}

impl CacheActor {
    pub fn new() -> Self {
        let mut owned_tasks = JoinSet::new();
        let self_addr = Address::<Self>::default(); // 임시 주소

        // 캐시 정리 작업 시작
        owned_tasks.spawn(Self::cleanup_cache(self_addr));

        Self {
            cache: HashMap::new(),
            _owned_tasks: owned_tasks,
        }
    }

    async fn cleanup_cache(_self_addr: Address<Self>) {
        // 실제 구현에서는 주기적으로 만료된 캐시 항목 정리
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            // 실제 구현에서는 self_addr.notify(CleanupCache).await 호출
        }
    }

    fn get_current_timestamp(&self) -> u64 {
        Utc::now().timestamp() as u64
    }
}

#[async_trait]
impl Handler<FetchData> for CacheActor {
    type Response = Result<Vec<u8>, UserError>;

    async fn handle(&mut self, msg: FetchData, _: &Context<Self>) -> Self::Response {
        if let Some(entry) = self.cache.get(&msg.key) {
            // 만료 확인
            if let Some(expires_at) = entry.expires_at {
                if expires_at < self.get_current_timestamp() {
                    self.cache.remove(&msg.key);
                    return Err("Cache entry expired".into());
                }
            }

            Ok(entry.data.clone())
        } else {
            Err("Cache miss".into())
        }
    }
}

#[async_trait]
impl Handler<CacheData> for CacheActor {
    type Response = Result<(), UserError>;

    async fn handle(&mut self, msg: CacheData, _: &Context<Self>) -> Self::Response {
        let expires_at = msg.ttl.map(|ttl| self.get_current_timestamp() + ttl);

        self.cache.insert(
            msg.key,
            CacheEntry {
                data: msg.data,
                expires_at,
            },
        );

        Ok(())
    }
}

// 저장소 액터
pub struct StorageActor {
    // 실제 구현에서는 파일 시스템이나 데이터베이스 연결
    _owned_tasks: JoinSet<()>,
}

impl Actor for StorageActor {}

impl StorageActor {
    pub fn new() -> Self {
        Self {
            _owned_tasks: JoinSet::new(),
        }
    }
}

#[async_trait]
impl Handler<FetchData> for StorageActor {
    type Response = Result<Vec<u8>, UserError>;

    async fn handle(&mut self, msg: FetchData, _: &Context<Self>) -> Self::Response {
        // 실제 구현에서는 파일 시스템이나 데이터베이스에서 데이터 가져오기
        Err("Storage implementation not available".into())
    }
}

#[async_trait]
impl Handler<StoreData> for StorageActor {
    type Response = Result<(), UserError>;

    async fn handle(&mut self, msg: StoreData, _: &Context<Self>) -> Self::Response {
        // 실제 구현에서는 파일 시스템이나 데이터베이스에 데이터 저장
        debug_print!(
            "Storing data for key: {}, size: {} bytes",
            msg.key,
            msg.data.len()
        );
        Ok(())
    }
}
