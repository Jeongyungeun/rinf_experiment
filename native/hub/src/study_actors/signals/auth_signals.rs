use rinf::{DartSignal, RustSignal};
use serde::{Deserialize, Serialize};
use super::super::messages::{UserId, AuthResult};

#[derive(DartSignal, Serialize, Deserialize, Debug)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(RustSignal, Serialize, Deserialize, Debug)]
pub struct LoginResponse {
    pub success: bool,
    pub user_id: Option<UserId>,
    pub token: Option<String>,
    pub error: Option<String>,
}

#[derive(DartSignal, Serialize, Deserialize, Debug)]
pub struct LogoutRequest {
    pub user_id: UserId,
}

#[derive(RustSignal, Serialize, Deserialize, Debug)]
pub struct LogoutResponse {
    pub success: bool,
}

#[derive(RustSignal, Serialize, Deserialize, Debug)]
pub struct AuthStateChanged {
    pub is_authenticated: bool,
    pub user_id: Option<UserId>,
}
