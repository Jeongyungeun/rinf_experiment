use rinf::{DartSignal, RustSignal};
use serde::{Deserialize, Serialize};
use super::super::messages::{UserId, DataItem, UserData};

#[derive(DartSignal, Serialize, Deserialize, Debug)]
pub struct FetchUserDataRequest {
    pub user_id: UserId,
    pub limit: Option<usize>,
}

#[derive(RustSignal, Serialize, Deserialize, Debug)]
pub struct UserDataResponse {
    pub user_id: UserId,
    pub items: Vec<DataItem>,
    pub last_updated: u64,
    pub error: Option<String>,
}

#[derive(DartSignal, Serialize, Deserialize, Debug)]
pub struct CreateDataItemRequest {
    pub user_id: UserId,
    pub title: String,
    pub content: String,
}

#[derive(RustSignal, Serialize, Deserialize, Debug)]
pub struct DataItemCreatedSignal {
    pub user_id: UserId,
    pub item: DataItem,
}

#[derive(DartSignal, Serialize, Deserialize, Debug)]
pub struct UpdateDataItemRequest {
    pub user_id: UserId,
    pub item_id: String,
    pub title: Option<String>,
    pub content: Option<String>,
}

#[derive(RustSignal, Serialize, Deserialize, Debug)]
pub struct DataItemUpdatedSignal {
    pub user_id: UserId,
    pub item: DataItem,
}

#[derive(DartSignal, Serialize, Deserialize, Debug)]
pub struct DeleteDataItemRequest {
    pub user_id: UserId,
    pub item_id: String,
}

#[derive(RustSignal, Serialize, Deserialize, Debug)]
pub struct DataItemDeletedSignal {
    pub user_id: UserId,
    pub item_id: String,
}
