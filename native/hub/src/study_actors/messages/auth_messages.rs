use serde::{Deserialize, Serialize};
use super::UserId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Login {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Logout {
    pub user_id: UserId,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyToken {
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessLogin {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResult {
    pub user_id: UserId,
    pub token: String,
    pub expires_at: u64,
}
