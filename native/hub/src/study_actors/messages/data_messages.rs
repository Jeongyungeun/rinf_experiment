use super::UserId;
use messages::prelude::Address;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchData {
    pub key: String,
    pub user_id: Option<UserId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreData {
    pub key: String,
    pub data: Vec<u8>,
    pub user_id: Option<UserId>,
    pub ttl: Option<u64>, // 초 단위 TTL
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheData {
    pub key: String,
    pub data: Vec<u8>,
    pub ttl: Option<u64>, // 초 단위 TTL
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchRecentData {
    pub user_id: UserId,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserData {
    pub user_id: UserId,
    pub items: Vec<DataItem>,
    pub last_updated: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataItem {
    pub id: String,
    pub title: String,
    pub content: String,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone)]
pub struct UpdateNetworkDependency(pub Address<super::super::actors::NetworkManagerActor>);
