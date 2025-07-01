mod auth_messages;
mod user_messages;
mod data_messages;

pub use auth_messages::{Login, Logout, VerifyToken, ProcessLogin, AuthResult};
pub use user_messages::{GetProfile, UpdateProfile, UserEvent};
pub use data_messages::{FetchData, StoreData, CacheData, FetchRecentData};

// 공통 타입 정의
pub type UserId = String;
pub type UserError = Box<dyn std::error::Error + Send + Sync>;
pub type AuthError = Box<dyn std::error::Error + Send + Sync>;
pub type StorageError = Box<dyn std::error::Error + Send + Sync>;
