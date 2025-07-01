use rinf::{DartSignal, RustSignal};
use serde::{Deserialize, Serialize};
use super::super::messages::{UserId, UserProfile, UserPreferences};

#[derive(DartSignal, Serialize, Deserialize, Debug)]
pub struct GetUserProfileRequest {
    pub user_id: UserId,
}

#[derive(RustSignal, Serialize, Deserialize, Debug)]
pub struct UserProfileResponse {
    pub profile: Option<UserProfile>,
    pub error: Option<String>,
}

#[derive(DartSignal, Serialize, Deserialize, Debug)]
pub struct UpdateUserProfileRequest {
    pub user_id: UserId,
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(RustSignal, Serialize, Deserialize, Debug)]
pub struct ProfileUpdatedSignal {
    pub user_id: UserId,
    pub profile: UserProfile,
}

#[derive(DartSignal, Serialize, Deserialize, Debug)]
pub struct UpdatePreferencesRequest {
    pub user_id: UserId,
    pub theme: Option<String>,
    pub notifications_enabled: Option<bool>,
    pub language: Option<String>,
}

#[derive(RustSignal, Serialize, Deserialize, Debug)]
pub struct PreferencesUpdatedSignal {
    pub user_id: UserId,
    pub preferences: UserPreferences,
}
