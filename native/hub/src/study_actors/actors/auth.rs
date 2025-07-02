use async_trait::async_trait;
use messages::{
    actor::Actor,
    prelude::{Address, Context, Handler, Notifiable},
};
use rinf::{debug_print, DartSignal, RustSignal};
use std::collections::HashMap;
use tokio::task::JoinSet;

use crate::study_actors::{
    messages::{AuthError, AuthResult, Login, Logout, UserId, VerifyToken},
    signals::{AuthStateChanged, LoginRequest, LoginResponse, LogoutRequest, LogoutResponse},
};

pub struct AuthActor {
    active_sessions: HashMap<String, AuthSession>,
    _owned_tasks: JoinSet<()>,
}

struct AuthSession {
    user_id: UserId,
    token: String,
    expires_at: u64,
}

impl Actor for AuthActor {}

impl AuthActor {
    pub fn new(self_addr: Address<Self>) -> Self {
        let mut owned_tasks = JoinSet::new();
        
        // 토큰 만료 체크 작업 시작
        owned_tasks.spawn(Self::check_token_expiry(self_addr));
        
        Self {
            active_sessions: HashMap::new(),
            _owned_tasks: owned_tasks,
        }
    }
    
    async fn check_token_expiry(mut self_addr: Address<Self>) {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            let _ = self_addr.notify(CheckExpiredTokens).await;
        }
    }
    
    fn generate_token(&self, user_id: &str) -> String {
        // 실제 구현에서는 보안 토큰 생성 로직 필요
        format!("token_{}_{}", user_id, chrono::Utc::now().timestamp())
    }
    
    fn get_current_timestamp(&self) -> u64 {
        chrono::Utc::now().timestamp() as u64
    }
}

// 내부 메시지 정의
struct CheckExpiredTokens;

#[async_trait]
impl Notifiable<CheckExpiredTokens> for AuthActor {
    async fn notify(&mut self, _: CheckExpiredTokens, _: &Context<Self>) {
        let current_time = self.get_current_timestamp();
        let expired_tokens: Vec<String> = self
            .active_sessions
            .iter()
            .filter(|(_, session)| session.expires_at < current_time)
            .map(|(token, _)| token.clone())
            .collect();
        
        for token in expired_tokens {
            if let Some(session) = self.active_sessions.remove(&token) {
                debug_print!("Token expired for user: {}", session.user_id);
                
                // 인증 상태 변경 알림
                AuthStateChanged {
                    is_authenticated: false,
                    user_id: Some(session.user_id),
                }
                .send_signal_to_dart();
            }
        }
    }
}

#[async_trait]
impl Handler<Login> for AuthActor {
    type Result = Result<AuthResult, AuthError>;
    
    async fn handle(&mut self, msg: Login, _: &Context<Self>) -> Self::Result {
        // 실제 구현에서는 데이터베이스 확인 등의 인증 로직 필요
        if msg.username == "demo" && msg.password == "password" {
            let user_id = "user_1".to_string();
            let token = self.generate_token(&user_id);
            let expires_at = self.get_current_timestamp() + 3600; // 1시간 후 만료
            
            let auth_result = AuthResult {
                user_id: user_id.clone(),
                token: token.clone(),
                expires_at,
            };
            
            // 세션 저장
            self.active_sessions.insert(
                token.clone(),
                AuthSession {
                    user_id: user_id.clone(),
                    token: token.clone(),
                    expires_at,
                },
            );
            
            // 인증 상태 변경 알림
            AuthStateChanged {
                is_authenticated: true,
                user_id: Some(user_id),
            }
            .send_signal_to_dart();
            
            Ok(auth_result)
        } else {
            Err("Invalid username or password".into())
        }
    }
}

#[async_trait]
impl Handler<Logout> for AuthActor {
    type Result = Result<(), AuthError>;
    
    async fn handle(&mut self, msg: Logout, _: &Context<Self>) -> Self::Result {
        if let Some(session) = self.active_sessions.remove(&msg.token) {
            // 인증 상태 변경 알림
            AuthStateChanged {
                is_authenticated: false,
                user_id: Some(session.user_id),
            }
            .send_signal_to_dart();
            
            Ok(())
        } else {
            Err("Invalid or expired token".into())
        }
    }
}

#[async_trait]
impl Handler<VerifyToken> for AuthActor {
    type Result = Result<UserId, AuthError>;
    
    async fn handle(&mut self, msg: VerifyToken, _: &Context<Self>) -> Self::Result {
        if let Some(session) = self.active_sessions.get(&msg.token) {
            if session.expires_at > self.get_current_timestamp() {
                Ok(session.user_id.clone())
            } else {
                Err("Token expired".into())
            }
        } else {
            Err("Invalid token".into())
        }
    }
}

// Dart 신호 처리
#[async_trait]
impl Notifiable<LoginRequest> for AuthActor {
    async fn notify(&mut self, msg: LoginRequest, ctx: &Context<Self>) {
        let login_result = self
            .handle(
                Login {
                    username: msg.username,
                    password: msg.password,
                },
                ctx,
            )
            .await;
        
        match login_result {
            Ok(result) => {
                LoginResponse {
                    success: true,
                    user_id: Some(result.user_id),
                    token: Some(result.token),
                    error: None,
                }
                .send_signal_to_dart();
            }
            Err(e) => {
                LoginResponse {
                    success: false,
                    user_id: None,
                    token: None,
                    error: Some(e.to_string()),
                }
                .send_signal_to_dart();
            }
        }
    }
}

#[async_trait]
impl Notifiable<LogoutRequest> for AuthActor {
    async fn notify(&mut self, msg: LogoutRequest, ctx: &Context<Self>) {
        // 사용자 ID로 토큰 찾기 (실제 구현에서는 더 효율적인 방법 필요)
        let token = self
            .active_sessions
            .iter()
            .find(|(_, session)| session.user_id == msg.user_id)
            .map(|(token, _)| token.clone());
        
        if let Some(token) = token {
            let logout_result = self
                .handle(
                    Logout {
                        user_id: msg.user_id,
                        token,
                    },
                    ctx,
                )
                .await;
            
            LogoutResponse {
                success: logout_result.is_ok(),
            }
            .send_signal_to_dart();
        } else {
            LogoutResponse { success: false }.send_signal_to_dart();
        }
    }
}
