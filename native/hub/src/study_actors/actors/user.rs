use async_trait::async_trait;
use messages::{
    actor::Actor,
    prelude::{Address, Context, Handler, Notifiable},
};
use rinf::{debug_print, RustSignal};
use std::collections::HashMap;
use tokio::task::JoinSet;

use crate::study_actors::{
    messages::{
        AuthResult, GetProfile, Login, UpdateProfile, UserError, UserId, UserEvent, UserProfile,
        UserPreferences, UpdateProfileCache,
    },
    signals::{
        GetUserProfileRequest, ProfileUpdatedSignal, UpdatePreferencesRequest,
        PreferencesUpdatedSignal, UserProfileResponse,
    },
};

use super::AuthActor;

pub struct UserManagerActor {
    auth_actor: Address<AuthActor>,
    profile_actors: HashMap<UserId, Address<UserProfileActor>>,
    _owned_tasks: JoinSet<()>,
}

impl Actor for UserManagerActor {}

impl UserManagerActor {
    pub fn new(auth_actor: Address<AuthActor>) -> Self {
        Self {
            auth_actor,
            profile_actors: HashMap::new(),
            _owned_tasks: JoinSet::new(),
        }
    }
    
    async fn get_or_create_profile_actor(&mut self, user_id: &UserId) -> Address<UserProfileActor> {
        if let Some(addr) = self.profile_actors.get(user_id) {
            return addr.clone();
        }
        
        // 새 프로필 액터 생성
        let context = Context::new();
        let addr = context.address();
        let actor = UserProfileActor::new(user_id.clone());
        
        // 액터 실행 및 저장
        tokio::spawn(context.run(actor));
        self.profile_actors.insert(user_id.clone(), addr.clone());
        
        addr
    }
}

#[async_trait]
impl Handler<Login> for UserManagerActor {
    type Response = Result<AuthResult, UserError>;
    
    async fn handle(&mut self, msg: Login, _: &Context<Self>) -> Self::Response {
        // 인증 액터에 로그인 요청 전달
        let auth_result = self.auth_actor.send(msg).await??;
        
        // 사용자 프로필 액터 생성 (없는 경우)
        self.get_or_create_profile_actor(&auth_result.user_id).await;
        
        Ok(auth_result)
    }
}

#[async_trait]
impl Handler<GetProfile> for UserManagerActor {
    type Response = Result<UserProfile, UserError>;
    
    async fn handle(&mut self, msg: GetProfile, _: &Context<Self>) -> Self::Response {
        let profile_actor = self.get_or_create_profile_actor(&msg.user_id).await;
        profile_actor.send(msg).await?
    }
}

#[async_trait]
impl Handler<UpdateProfile> for UserManagerActor {
    type Response = Result<(), UserError>;
    
    async fn handle(&mut self, msg: UpdateProfile, _: &Context<Self>) -> Self::Response {
        let profile_actor = self.get_or_create_profile_actor(&msg.user_id).await;
        let result = profile_actor.send(msg.clone()).await?;
        
        if result.is_ok() {
            // 프로필 업데이트 이벤트 발행
            let _ = self.notify(UserEvent::ProfileUpdated(msg.user_id, msg.profile)).await;
        }
        
        result
    }
}

#[async_trait]
impl Notifiable<UserEvent> for UserManagerActor {
    async fn notify(&mut self, event: UserEvent, _: &Context<Self>) {
        match event {
            UserEvent::ProfileUpdated(user_id, profile) => {
                debug_print!("Profile updated for user: {}", user_id);
                
                // 프로필 캐시 업데이트
                if let Some(addr) = self.profile_actors.get(&user_id) {
                    let _ = addr.notify(UpdateProfileCache(profile.clone())).await;
                }
                
                // Dart에 알림
                ProfileUpdatedSignal {
                    user_id,
                    profile,
                }.send_signal_to_dart();
            },
            UserEvent::PreferencesChanged(user_id, preferences) => {
                debug_print!("Preferences changed for user: {}", user_id);
                
                // Dart에 알림
                PreferencesUpdatedSignal {
                    user_id,
                    preferences,
                }.send_signal_to_dart();
            },
            UserEvent::LoggedIn(user_id) => {
                debug_print!("User logged in: {}", user_id);
            },
            UserEvent::LoggedOut(user_id) => {
                debug_print!("User logged out: {}", user_id);
                
                // 프로필 액터 제거 (선택적)
                self.profile_actors.remove(&user_id);
            },
        }
    }
}

// Dart 신호 처리
#[async_trait]
impl Notifiable<GetUserProfileRequest> for UserManagerActor {
    async fn notify(&mut self, msg: GetUserProfileRequest, ctx: &Context<Self>) {
        let profile_result = self
            .handle(
                GetProfile {
                    user_id: msg.user_id,
                },
                ctx,
            )
            .await;
        
        match profile_result {
            Ok(profile) => {
                UserProfileResponse {
                    profile: Some(profile),
                    error: None,
                }
                .send_signal_to_dart();
            }
            Err(e) => {
                UserProfileResponse {
                    profile: None,
                    error: Some(e.to_string()),
                }
                .send_signal_to_dart();
            }
        }
    }
}

#[async_trait]
impl Notifiable<UpdatePreferencesRequest> for UserManagerActor {
    async fn notify(&mut self, msg: UpdatePreferencesRequest, ctx: &Context<Self>) {
        // 먼저 현재 프로필 가져오기
        let profile_result = self
            .handle(
                GetProfile {
                    user_id: msg.user_id.clone(),
                },
                ctx,
            )
            .await;
        
        if let Ok(mut profile) = profile_result {
            // 선택적 필드 업데이트
            if let Some(theme) = msg.theme {
                profile.preferences.theme = theme;
            }
            
            if let Some(notifications_enabled) = msg.notifications_enabled {
                profile.preferences.notifications_enabled = notifications_enabled;
            }
            
            if let Some(language) = msg.language {
                profile.preferences.language = language;
            }
            
            // 프로필 업데이트
            let _ = self
                .handle(
                    UpdateProfile {
                        user_id: msg.user_id,
                        profile,
                    },
                    ctx,
                )
                .await;
        }
    }
}

// 사용자 프로필 액터
pub struct UserProfileActor {
    user_id: UserId,
    profile: Option<UserProfile>,
}

impl Actor for UserProfileActor {}

impl UserProfileActor {
    pub fn new(user_id: UserId) -> Self {
        Self {
            user_id,
            profile: None,
        }
    }
    
    fn create_default_profile(&self) -> UserProfile {
        UserProfile {
            user_id: self.user_id.clone(),
            name: format!("User {}", self.user_id),
            email: format!("user{}@example.com", self.user_id),
            avatar_url: None,
            preferences: UserPreferences {
                theme: "light".to_string(),
                notifications_enabled: true,
                language: "en".to_string(),
            },
        }
    }
}

#[async_trait]
impl Handler<GetProfile> for UserProfileActor {
    type Response = Result<UserProfile, UserError>;
    
    async fn handle(&mut self, _: GetProfile, _: &Context<Self>) -> Self::Response {
        // 프로필이 없으면 기본값 생성
        if self.profile.is_none() {
            self.profile = Some(self.create_default_profile());
        }
        
        Ok(self.profile.clone().unwrap())
    }
}

#[async_trait]
impl Handler<UpdateProfile> for UserProfileActor {
    type Response = Result<(), UserError>;
    
    async fn handle(&mut self, msg: UpdateProfile, _: &Context<Self>) -> Self::Response {
        // 프로필 업데이트
        self.profile = Some(msg.profile);
        Ok(())
    }
}

#[async_trait]
impl Notifiable<UpdateProfileCache> for UserProfileActor {
    async fn notify(&mut self, msg: UpdateProfileCache, _: &Context<Self>) {
        // 프로필 캐시 업데이트
        self.profile = Some(msg.0);
    }
}
