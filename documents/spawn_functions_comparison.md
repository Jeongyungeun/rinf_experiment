# Rust에서의 세 가지 spawn 함수 비교

Rust의 비동기 프로그래밍과 Actor 모델에서 사용되는 세 가지 spawn 함수의 차이점과 사용 예제를 설명합니다.

## 세 가지 spawn 함수의 실제 예제와 비교

현재 `supervisor.rs` 파일에서 볼 수 있는 세 가지 spawn 함수의 실제 사용 예를 살펴보겠습니다:

## 1. `tokio::spawn` 예제

```rust
// AppSupervisor.new 메서드에서의 사용
let network_context = Context::new();
let network_addr = network_context.address();
let network_actor = NetworkManagerActor::new();
tokio::spawn(network_context.run(network_actor));
```

**특징:**
- 액터 컨텍스트를 독립적으로 실행
- 실행 결과를 기다리지 않음 (fire-and-forget)
- 액터의 전체 수명 주기 관리
- Tokio 런타임에서 직접 관리되는 독립적인 태스크

## 2. `JoinSet::spawn` 예제

```rust
// CacheActor.new 메서드에서의 사용
let mut owned_tasks = JoinSet::new();
let self_addr = Address::<Self>::default();

// 캐시 정리 작업 시작
owned_tasks.spawn(Self::cleanup_cache(self_addr));

Self {
    cache: HashMap::new(),
    _owned_tasks: owned_tasks,
}
```

**특징:**
- 액터 내부에서 관련 작업 관리
- 액터가 소유하므로 액터가 삭제되면 작업도 취소됨
- 필요시 결과 수집 가능
- 여러 관련 태스크를 그룹으로 관리

## 3. `Address::spawn` 예제

```rust
// 가상 예제 (실제 코드에는 없음)
impl AuthActor {
    fn start_token_check(&self, ctx: &Context<Self>) {
        let addr = ctx.address();
        addr.spawn(async move {
            // 이 함수는 액터의 컨텍스트에서 실행됨
            self.check_expired_tokens().await;
        });
    }
}
```

**특징:**
- 액터의 컨텍스트 내에서 실행
- 액터의 상태에 안전하게 접근 가능
- 메시지 큐를 통해 처리되므로 순서가 보장됨
- 액터의 메시지 처리 파이프라인에 통합됨

## 실행 순서 비교

다음 시나리오를 통해 실행 순서를 비교해 보겠습니다:

```rust
// 1. tokio::spawn - 즉시 스케줄링되지만 실행 순서는 보장되지 않음
tokio::spawn(async {
    println!("Task A");
    tokio::time::sleep(Duration::from_millis(100)).await;
    println!("Task A completed");
});

tokio::spawn(async {
    println!("Task B");
    println!("Task B completed");
});

// 출력 순서는 보장되지 않음:
// "Task B" -> "Task B completed" -> "Task A" -> "Task A completed"
// 또는
// "Task A" -> "Task B" -> "Task B completed" -> "Task A completed"

// 2. JoinSet::spawn - 여러 작업을 그룹으로 관리하고 결과 수집
let mut join_set = JoinSet::new();

join_set.spawn(async {
    println!("JoinSet Task 1");
    "Result 1"
});

join_set.spawn(async {
    println!("JoinSet Task 2");
    "Result 2"
});

// 결과 수집 (완료되는 순서대로)
while let Some(result) = join_set.join_next().await {
    println!("Got result: {}", result.unwrap());
}

// 3. Address::spawn - 액터의 메시지 큐를 통해 처리되므로 순서 보장
let addr = actor_context.address();

addr.spawn(async move {
    println!("Actor Task 1");
});

addr.spawn(async move {
    println!("Actor Task 2");
});

// 출력 순서는 보장됨:
// "Actor Task 1" -> "Actor Task 2"
```

## 실제 사용 시나리오

### 시나리오 1: 주기적인 토큰 만료 확인

```rust
// AuthActor 내부
pub fn new(self_addr: Address<Self>) -> Self {
    let mut owned_tasks = JoinSet::new();
    
    // JoinSet::spawn - 액터 소유의 백그라운드 작업
    owned_tasks.spawn(Self::check_token_expiry(self_addr));
    
    Self {
        active_sessions: HashMap::new(),
        _owned_tasks: owned_tasks,
    }
}

async fn check_token_expiry(mut self_addr: Address<Self>) {
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        // 액터에 메시지 전송
        let _ = self_addr.notify(CheckExpiredTokens).await;
    }
}
```

### 시나리오 2: 액터 시스템 초기화

```rust
// AppSupervisor.new에서
// tokio::spawn - 독립적인 액터 실행
tokio::spawn(network_context.run(network_actor));
tokio::spawn(data_context.run(data_actor));
tokio::spawn(auth_context.run(auth_actor));
```

### 시나리오 3: 복잡한 워크플로우 조정

```rust
// 가상 예제
impl WorkflowActor {
    async fn process_workflow(&mut self, workflow_id: String) {
        // 1. 독립적인 작업은 tokio::spawn
        let data_future = tokio::spawn(async move {
            // 외부 API에서 데이터 가져오기
            fetch_external_data(workflow_id).await
        });
        
        // 2. 관련 작업은 JoinSet::spawn
        self._owned_tasks.spawn(async move {
            // 워크플로우 진행 상황 모니터링
            monitor_workflow_progress(workflow_id).await
        });
        
        // 3. 액터 상태 접근이 필요한 작업은 Address::spawn
        let self_addr = self.context.address();
        self_addr.spawn(async move {
            // 액터 상태 업데이트
            self.update_workflow_state(workflow_id).await;
        });
    }
}
```

## 사용 시 고려사항

### 1. `tokio::spawn`
- **언제 사용하는가**: 독립적인 비동기 작업, 액터 컨텍스트 실행
- **장점**: 간단한 API, 독립적인 실행
- **단점**: 작업 취소 관리가 어려움, 결과 수집에 추가 코드 필요
- **적합한 상황**: 액터 시스템 초기화, 독립적인 백그라운드 작업

### 2. `JoinSet::spawn`
- **언제 사용하는가**: 관련된 여러 비동기 작업 관리, 결과 수집 필요 시
- **장점**: 그룹 관리, 자동 취소, 결과 수집 용이
- **단점**: 소유자 필요, `tokio::spawn`보다 약간 복잡한 API
- **적합한 상황**: 액터 내부 백그라운드 작업, 주기적 작업, 리소스 정리

### 3. `Address::spawn`
- **언제 사용하는가**: 액터 상태 접근 필요, 순서 보장 필요 시
- **장점**: 액터 상태 안전 접근, 메시지 순서 보장
- **단점**: 액터 컨텍스트 필요, 다른 방법보다 오버헤드 있음
- **적합한 상황**: 액터 상태 변경 작업, 순차적 처리 필요한 작업

## 결론

세 가지 spawn 함수는 각각 다른 목적과 실행 컨텍스트를 가지고 있으며, 적절한 상황에 맞게 선택하여 사용하는 것이 중요합니다:

1. **`tokio::spawn`**: 독립적인 태스크를 간단히 실행할 때
2. **`JoinSet::spawn`**: 관련된 여러 태스크를 그룹으로 관리하고 결과를 수집할 때
3. **`Address::spawn`**: 액터의 상태에 안전하게 접근하고 순서를 보장해야 할 때

이러한 차이점을 이해하고 적절히 활용하면 효율적이고 안전한 비동기 프로그래밍이 가능합니다.
