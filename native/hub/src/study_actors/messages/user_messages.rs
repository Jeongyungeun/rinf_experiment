use serde::{Deserialize, Serialize};
use super::UserId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub user_id: UserId,
    pub name: String,
    pub email: String,
    pub avatar_url: Option<String>,
    pub preferences: UserPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub theme: String,
    pub notifications_enabled: bool,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetProfile {
    pub user_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProfile {
    pub user_id: UserId,
    pub profile: UserProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserEvent {
    ProfileUpdated(UserId, UserProfile),
    PreferencesChanged(UserId, UserPreferences),
    LoggedIn(UserId),
    LoggedOut(UserId),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProfileCache(pub UserProfile);
