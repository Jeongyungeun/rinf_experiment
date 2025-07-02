use messages::prelude::Address;
use reqwest::Method;
use crate::study_actors::actors::network::{NetworkManagerActor, NetworkRequest};

// 네트워크 요청 예제
async fn example_network_requests(network_actor: Address<NetworkManagerActor>) {
    // 1. 기본 GET 요청
    let get_request = NetworkRequest::new("https://api.example.com/data")
        .method(Method::GET)
        .header("Accept", "application/json")
        .timeout(5000); // 5초 타임아웃
    
    let get_response = network_actor.send(get_request).await.unwrap();
    if let Ok(response) = get_response {
        if response.is_success() {
            println!("GET 요청 성공: {:?}", response.status);
            
            // JSON 응답 파싱
            if let Ok(json_data) = response.json::<serde_json::Value>() {
                println!("응답 데이터: {:?}", json_data);
            }
            
            // 텍스트 응답 파싱
            if let Ok(text) = response.text() {
                println!("응답 텍스트: {}", text);
            }
        } else {
            println!("GET 요청 실패: {:?}, 오류: {:?}", response.status, response.error);
        }
    }
    
    // 2. JSON 데이터와 함께 POST 요청
    let json_data = serde_json::json!({
        "name": "사용자",
        "email": "user@example.com",
        "age": 30
    });
    
    let post_request = NetworkRequest::new("https://api.example.com/users")
        .method(Method::POST)
        .header("Content-Type", "application/json")
        .json(&json_data);
    
    let post_response = network_actor.send(post_request).await.unwrap();
    if let Ok(response) = post_response {
        if response.is_success() {
            println!("POST 요청 성공: {:?}", response.status);
        } else {
            println!("POST 요청 실패: {:?}", response.status);
        }
    }
    
    // 3. 바이너리 데이터와 함께 PUT 요청
    let binary_data = vec![0, 1, 2, 3, 4, 5];
    
    let put_request = NetworkRequest::new("https://api.example.com/files")
        .method(Method::PUT)
        .header("Content-Type", "application/octet-stream")
        .body(binary_data);
    
    let put_response = network_actor.send(put_request).await.unwrap();
    if let Ok(response) = put_response {
        println!("PUT 요청 상태: {:?}", response.status);
    }
}
