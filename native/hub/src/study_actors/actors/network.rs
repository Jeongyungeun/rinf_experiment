use async_trait::async_trait;
use messages::{
    actor::Actor,
    prelude::{Address, Context, Handler, Notifiable},
};
use reqwest::{
    self, Body, Error, Method, Response, StatusCode,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use rinf::{RustSignal, debug_print};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr, time::Duration};
use tokio::task::JoinSet;

use crate::study_actors::messages::UserError;

// 네트워크 요청 타입
#[derive(Debug)]
pub struct NetworkRequest {
    pub url: String,
    pub method: Method,
    pub headers: HeaderMap,
    pub body: Option<Body>,
    pub timeout_ms: Option<u64>,
    pub json: Option<serde_json::Value>,
}

impl NetworkRequest {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            method: Method::GET,
            headers: HeaderMap::new(),
            body: None,
            timeout_ms: None,
            json: None,
        }
    }

    pub fn method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }

    pub fn header(mut self, key: &str, value: &str) -> Self {
        if let (Ok(name), Ok(val)) = (HeaderName::from_str(key), HeaderValue::from_str(value)) {
            self.headers.insert(name, val);
        }
        self
    }

    pub fn timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = Some(ms);
        self
    }

    pub fn body(mut self, body: impl Into<Body>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn json<T: Serialize>(mut self, json: &T) -> Self {
        if let Ok(value) = serde_json::to_value(json) {
            self.json = Some(value);
        }
        self
    }
}

#[derive(Debug, Clone)]
pub struct NetworkResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
    pub error: Option<String>,
}

impl NetworkResponse {
    pub fn is_success(&self) -> bool {
        self.status.is_success() && self.error.is_none()
    }

    pub fn json<T: for<'de> Deserialize<'de>>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_slice(&self.body)
    }

    pub fn text(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.body.clone())
    }
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
        let owned_tasks = JoinSet::new();

        Self {
            connection_pool: HashMap::new(),
            max_connections: 10,
            _owned_tasks: owned_tasks,
        }
    }

    fn started(&mut self, ctx: &Context<Self>) {
        // actor가 인스턴스화 되고 context에서 주소를 얻는 방법이 일반적이다.
        let self_addr = ctx.address();

        // 네트워크 상태 모니터링 작업 시작
        self._owned_tasks
            .spawn(Self::monitor_network_status(self_addr));
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
    type Result = Result<NetworkResponse, UserError>;

    async fn handle(&mut self, msg: NetworkRequest, _: &Context<Self>) -> Self::Result {
        let domain = self.extract_domain(&msg.url);

        // 연결 수 증가
        let connection_count = self.connection_pool.entry(domain.clone()).or_insert(0);
        *connection_count += 1;

        // 최대 연결 수 초과 확인
        if *connection_count > self.max_connections as u32 {
            *connection_count -= 1;
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Too many connections to domain: {}", domain),
            )) as UserError);
        }

        debug_print!("Sending {} request to {}", msg.method.as_str(), msg.url);

        // reqwest 클라이언트 생성
        let client = reqwest::Client::builder();

        // 타임아웃 설정
        let client = if let Some(timeout) = msg.timeout_ms {
            client.timeout(Duration::from_millis(timeout))
        } else {
            client
        };

        let client = client.build().map_err(|e| {
            debug_print!("Failed to build HTTP client: {}", e);
            Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Network error: Failed to build HTTP client: {}", e),
            )) as UserError
        })?;

        // 요청 생성
        let mut request_builder = client.request(msg.method.clone(), &msg.url);

        // 헤더 설정
        request_builder = request_builder.headers(msg.headers.clone());

        // JSON 또는 바디 설정
        if let Some(json) = msg.json {
            request_builder = request_builder.json(&json);
        } else if let Some(body) = msg.body {
            request_builder = request_builder.body(body);
        }

        // 요청 실행
        let result = match request_builder.send().await {
            Ok(resp) => {
                let status = resp.status();
                let headers = resp.headers().clone();

                // 응답 바디 읽기
                match resp.bytes().await {
                    Ok(bytes) => NetworkResponse {
                        status,
                        headers,
                        body: bytes.to_vec(),
                        error: None,
                    },
                    Err(e) => NetworkResponse {
                        status,
                        headers,
                        body: Vec::new(),
                        error: Some(format!("Failed to read response body: {}", e)),
                    },
                }
            }
            Err(e) => NetworkResponse {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                headers: HeaderMap::new(),
                body: Vec::new(),
                error: Some(format!("Request failed: {}", e)),
            },
        };

        // 연결 수 감소
        if let Some(count) = self.connection_pool.get_mut(&domain) {
            *count = count.saturating_sub(1);
        }

        Ok(result)
    }
}

// 네트워크 상태 확인 메시지
struct CheckNetworkStatus;

#[async_trait]
/// NetworkManagerActor가 CheckNetworkStatus를 받았을때 어떻게 하는지를 나타낸다.
impl Notifiable<CheckNetworkStatus> for NetworkManagerActor {
    async fn notify(&mut self, _: CheckNetworkStatus, _: &Context<Self>) {
        debug_print!("Checking network status...");
        // 실제 구현에서는 네트워크 상태 확인 및 문제 해결
    }
}
