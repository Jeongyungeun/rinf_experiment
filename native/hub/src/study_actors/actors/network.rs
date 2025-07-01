use async_trait::async_trait;
use messages::{
    actor::Actor,
    prelude::{Address, Context, Handler, Notifiable},
};
use rinf::{debug_print, RustSignal};
use std::collections::HashMap;
use tokio::task::JoinSet;

use crate::study_actors::messages::UserError;

// 네트워크 요청 타입
#[derive(Debug, Clone)]
pub struct NetworkRequest {
    pub url: String,
    pub method: HttpMethod,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct NetworkResponse {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

// 네트워크 관리자 액터
pub struct NetworkManagerActor {
    connection_pool: HashMap<String, u32>, // 도메인별 연결 수 추적
    max_connections: usize,
    _owned_tasks: JoinSet<()>,
}

impl Actor for NetworkManagerActor {}

impl NetworkManagerActor {
    pub fn new() -> Self {
        let mut owned_tasks = JoinSet::new();
        let self_addr = Address::<Self>::default(); // 임시 주소
        
        // 네트워크 상태 모니터링 작업 시작
        owned_tasks.spawn(Self::monitor_network_status(self_addr));
        
        Self {
            connection_pool: HashMap::new(),
            max_connections: 10,
            _owned_tasks: owned_tasks,
        }
    }
    
    async fn monitor_network_status(_self_addr: Address<Self>) {
        // 실제 구현에서는 주기적으로 네트워크 상태 확인
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            // 실제 구현에서는 self_addr.notify(CheckNetworkStatus).await 호출
        }
    }
    
    fn extract_domain(&self, url: &str) -> String {
        // 간단한 도메인 추출 (실제 구현에서는 더 정교한 방법 필요)
        url.split("://")
            .nth(1)
            .unwrap_or(url)
            .split('/')
            .next()
            .unwrap_or(url)
            .to_string()
    }
}

#[async_trait]
impl Handler<NetworkRequest> for NetworkManagerActor {
    type Response = Result<NetworkResponse, UserError>;
    
    async fn handle(&mut self, msg: NetworkRequest, _: &Context<Self>) -> Self::Response {
        let domain = self.extract_domain(&msg.url);
        
        // 연결 수 증가
        let connection_count = self.connection_pool.entry(domain.clone()).or_insert(0);
        *connection_count += 1;
        
        // 최대 연결 수 초과 확인
        if *connection_count > self.max_connections as u32 {
            *connection_count -= 1;
            return Err(format!("Too many connections to domain: {}", domain).into());
        }
        
        // 실제 HTTP 요청 수행 (여기서는 시뮬레이션)
        debug_print!(
            "Sending {} request to {}",
            format!("{:?}", msg.method),
            msg.url
        );
        
        // 요청 시뮬레이션 (실제 구현에서는 reqwest 등의 HTTP 클라이언트 사용)
        let response = match msg.method {
            HttpMethod::Get => {
                // GET 요청 시뮬레이션
                NetworkResponse {
                    status_code: 200,
                    headers: HashMap::new(),
                    body: b"Sample response data".to_vec(),
                    error: None,
                }
            }
            HttpMethod::Post => {
                // POST 요청 시뮬레이션
                NetworkResponse {
                    status_code: 201,
                    headers: HashMap::new(),
                    body: b"{\"id\": \"123\", \"status\": \"created\"}".to_vec(),
                    error: None,
                }
            }
            _ => {
                // 기타 요청 시뮬레이션
                NetworkResponse {
                    status_code: 200,
                    headers: HashMap::new(),
                    body: b"OK".to_vec(),
                    error: None,
                }
            }
        };
        
        // 연결 수 감소
        if let Some(count) = self.connection_pool.get_mut(&domain) {
            *count = count.saturating_sub(1);
        }
        
        Ok(response)
    }
}

// 네트워크 상태 확인 메시지
struct CheckNetworkStatus;

#[async_trait]
impl Notifiable<CheckNetworkStatus> for NetworkManagerActor {
    async fn notify(&mut self, _: CheckNetworkStatus, _: &Context<Self>) {
        debug_print!("Checking network status...");
        // 실제 구현에서는 네트워크 상태 확인 및 문제 해결
    }
}
