# Address, Context, Actor의 관계와 역할

Address, Context, Actor는 Rinf의 액터 모델에서 핵심적인 세 가지 구성 요소입니다. 이들의 관계와 역할을 예제와 함께 자세히 살펴보겠습니다.

## 1. 기본 개념과 관계

### Actor
- **정의**: 상태와 동작을 캡슐화한 독립적인 실행 단위
- **역할**: 메시지를 받아 처리하고 상태를 관리
- **특징**: 자신의 상태에 대한 배타적 접근 권한을 가짐

### Context
- **정의**: Actor의 실행 환경을 제공하는 컨테이너
- **역할**: 메시지 큐 관리, Actor 인스턴스 실행, 생명주기 관리
- **특징**: Actor 인스턴스와 1:1 관계를 가짐

### Address
- **정의**: Actor에 메시지를 보내기 위한 핸들(참조)
- **역할**: Actor의 위치를 추상화하고 메시지 전달
- **특징**: 여러 개가 존재할 수 있으며 복제 가능

## 2. 관계 다이어그램

```
+----------------+       생성       +----------------+
|                | ---------------→ |                |
|    Context     |                  |     Actor      |
|                | ←--------------- |                |
+----------------+       실행       +----------------+
       ↓                                   ↑
       |                                   |
       | 생성                              | 메시지 전달
       |                                   |
       ↓                                   |
+----------------+                         |
|                |-------------------------+
|    Address     |
|                |
+----------------+
```

## 3. 코드로 보는 관계와 흐름

### 기본 생성 및 실행 흐름

```rust
// 1. Context 생성
let context = Context::new();

// 2. Context로부터 Address 얻기
let address = context.address();

// 3. Actor 인스턴스 생성
let actor = MyActor::new(address.clone());

// 4. Context에서 Actor 실행
tokio::spawn(context.run(actor));

// 5. Address를 통해 Actor에 메시지 전송
let result = address.send(MyMessage { data: "hello" }).await;
```

## 4. 상세 예제: 온도 센서 모니터링 시스템

더 구체적인 예제를 통해 세 구성 요소의 관계와 역할을 살펴보겠습니다:

```rust
use tokio::sync::JoinSet;
use std::collections::HashMap;
use std::time::Duration;

// 메시지 정의
struct ReadTemperature;
struct SetAlarmThreshold(f32);
struct GetStatus;

// 응답 정의
struct TemperatureReading {
    value: f32,
    timestamp: u64,
}

struct Status {
    is_active: bool,
    current_temp: f32,
    alarm_threshold: f32,
}

// Actor 정의
struct TemperatureSensorActor {
    sensor_id: String,
    current_temperature: f32,
    alarm_threshold: f32,
    is_active: bool,
    readings_history: Vec<TemperatureReading>,
    _owned_tasks: JoinSet<()>,
}

// Actor 트레이트 구현
impl Actor for TemperatureSensorActor {}

// Actor 구현
impl TemperatureSensorActor {
    fn new(sensor_id: String, self_addr: Address<Self>) -> Self {
        let mut owned_tasks = JoinSet::new();
        
        // 주기적인 온도 읽기 작업 시작
        owned_tasks.spawn(Self::periodic_temperature_read(self_addr.clone()));
        
        Self {
            sensor_id,
            current_temperature: 20.0,
            alarm_threshold: 30.0,
            is_active: true,
            readings_history: Vec::new(),
            _owned_tasks: owned_tasks,
        }
    }
    
    // 백그라운드 작업
    async fn periodic_temperature_read(addr: Address<Self>) {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            // 주기적으로 온도 읽기 메시지 전송
            let _ = addr.send(ReadTemperature).await;
        }
    }
}

// 메시지 핸들러 구현
#[async_trait]
impl Handler<ReadTemperature> for TemperatureSensorActor {
    type Result = TemperatureReading;
    
    async fn handle(&mut self, _msg: ReadTemperature, _ctx: &Context<Self>) -> Self::Result {
        // 실제로는 센서에서 온도를 읽어옴 (여기서는 시뮬레이션)
        let new_temp = self.current_temperature + (rand::random::<f32>() - 0.5) * 2.0;
        self.current_temperature = new_temp;
        
        let timestamp = chrono::Utc::now().timestamp() as u64;
        let reading = TemperatureReading { value: new_temp, timestamp };
        
        // 기록 저장
        self.readings_history.push(reading.clone());
        
        // 알람 임계값 확인
        if new_temp > self.alarm_threshold {
            println!("ALARM: Temperature {} exceeds threshold {}!", new_temp, self.alarm_threshold);
        }
        
        reading
    }
}

#[async_trait]
impl Handler<SetAlarmThreshold> for TemperatureSensorActor {
    type Result = ();
    
    async fn handle(&mut self, msg: SetAlarmThreshold, _ctx: &Context<Self>) -> Self::Result {
        println!("Setting alarm threshold from {} to {}", self.alarm_threshold, msg.0);
        self.alarm_threshold = msg.0;
    }
}

#[async_trait]
impl Handler<GetStatus> for TemperatureSensorActor {
    type Result = Status;
    
    async fn handle(&mut self, _msg: GetStatus, _ctx: &Context<Self>) -> Self::Result {
        Status {
            is_active: self.is_active,
            current_temp: self.current_temperature,
            alarm_threshold: self.alarm_threshold,
        }
    }
}

// 사용 예제
async fn run_temperature_monitoring() {
    // 1. Context 생성
    let context = Context::new();
    
    // 2. Address 얻기
    let address = context.address();
    
    // 3. Actor 생성 (Address 주입)
    let actor = TemperatureSensorActor::new("living_room".to_string(), address.clone());
    
    // 4. Context에서 Actor 실행
    let actor_handle = tokio::spawn(context.run(actor));
    
    // 5. Address를 통해 메시지 전송
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // 온도 읽기 요청
    let reading = address.send(ReadTemperature).await.unwrap();
    println!("Current temperature: {}", reading.value);
    
    // 알람 임계값 설정
    let _ = address.send(SetAlarmThreshold(25.0)).await;
    
    // 상태 확인
    let status = address.send(GetStatus).await.unwrap();
    println!("Sensor status: active={}, temp={}, threshold={}", 
             status.is_active, status.current_temp, status.alarm_threshold);
    
    // 일정 시간 실행 후 종료
    tokio::time::sleep(Duration::from_secs(30)).await;
    actor_handle.abort(); // Actor 종료
}
```

## 5. 각 구성 요소의 세부 역할 분석

### Actor의 역할
1. **상태 관리**: `TemperatureSensorActor`는 온도, 임계값, 활성 상태 등의 상태를 관리
2. **메시지 처리**: `ReadTemperature`, `SetAlarmThreshold`, `GetStatus` 메시지에 대한 핸들러 제공
3. **비즈니스 로직 실행**: 온도 읽기, 알람 체크 등의 로직 수행

### Context의 역할
1. **메시지 큐 관리**: Actor로 들어오는 메시지를 큐에 저장하고 순차적으로 처리
2. **Actor 실행 환경 제공**: `context.run(actor)`를 통해 Actor의 실행 환경 제공
3. **생명주기 관리**: Actor가 종료될 때까지 메시지 처리 루프 유지

### Address의 역할
1. **메시지 전달**: `address.send(ReadTemperature)`와 같이 Actor에 메시지 전송
2. **위치 추상화**: 실제 Actor의 위치나 상태와 무관하게 메시지 전달 가능
3. **비동기 통신 지원**: `send()` 메서드는 `Future`를 반환하여 비동기 통신 지원

## 6. 고급 패턴: 계층적 Actor 시스템

여러 Actor가 협력하는 계층적 시스템에서의 관계를 살펴보겠습니다:

```rust
// 상위 Actor: 여러 온도 센서 관리
struct TemperatureMonitorActor {
    sensors: HashMap<String, Address<TemperatureSensorActor>>,
    alert_subscribers: Vec<Address<AlertSubscriberActor>>,
}

impl Actor for TemperatureMonitorActor {}

impl TemperatureMonitorActor {
    fn new() -> Self {
        Self {
            sensors: HashMap::new(),
            alert_subscribers: Vec::new(),
        }
    }
    
    // 새 센서 추가
    fn add_sensor(&mut self, id: String, location: String) {
        // 1. 새 센서 Actor의 Context 생성
        let context = Context::new();
        let address = context.address();
        
        // 2. 센서 Actor 생성
        let actor = TemperatureSensorActor::new(id.clone(), address.clone());
        
        // 3. Context에서 Actor 실행
        tokio::spawn(context.run(actor));
        
        // 4. 센서 Address 저장
        self.sensors.insert(id, address);
    }
    
    // 모든 센서의 상태 확인
    async fn check_all_sensors(&self) -> HashMap<String, Status> {
        let mut results = HashMap::new();
        
        for (id, addr) in &self.sensors {
            if let Ok(status) = addr.send(GetStatus).await {
                results.insert(id.clone(), status);
            }
        }
        
        results
    }
}
```

## 7. 핵심 관계 요약

1. **Context는 Actor를 생성하고 실행**:
   - Context는 Actor의 실행 환경을 제공
   - Actor의 메시지 처리 루프를 관리

2. **Address는 Context로부터 생성**:
   - `context.address()`를 통해 Address 생성
   - Address는 Actor에 메시지를 보내는 핸들

3. **Actor는 Address를 통해 메시지 수신**:
   - Actor는 자신의 Address를 알 수 있음
   - 다른 Actor나 시스템 컴포넌트는 Address를 통해 메시지 전송

4. **생명주기 관계**:
   - Context가 종료되면 Actor도 종료
   - Address는 Context/Actor와 독립적으로 존재 가능
   - Address를 통한 메시지 전송은 Actor가 살아있을 때만 성공

이러한 관계를 통해 Rinf의 Actor 모델은 비동기적이고 안전한 상태 관리와 메시지 기반 통신을 가능하게 합니다.

## 8. JoinSet과 다른 구성 요소와의 비교

### JoinSet
- **정의**: 여러 비동기 태스크를 관리하고 결과를 수집하는 Tokio의 유틸리티
- **역할**: 여러 비동기 작업을 생성하고, 완료될 때 결과를 수집하며, 리소스 정리를 자동화
- **특징**: 태스크 그룹 관리와 결과 수집에 최적화

### JoinSet vs Context

| 특성 | JoinSet | Context |
|------|---------|--------|
| **주요 목적** | 여러 비동기 태스크 관리 | Actor 실행 환경 제공 |
| **생명주기 관리** | 태스크 완료 시 자동 정리 | Actor의 전체 생명주기 관리 |
| **메시지 처리** | 직접적인 메시지 큐 없음 | 메시지 큐 관리 및 순차적 처리 |
| **결과 수집** | `join_next()`로 완료된 태스크 결과 수집 | 메시지 응답을 통해 결과 반환 |
| **사용 사례** | 병렬 작업, 팬아웃 패턴 | Actor 기반 상태 관리 및 통신 |

### JoinSet vs Address

| 특성 | JoinSet | Address |
|------|---------|--------|
| **통신 방식** | 직접적인 통신 메커니즘 없음 | 메시지 기반 비동기 통신 |
| **참조 특성** | 태스크에 대한 소유권 | Actor에 대한 참조(핸들) |
| **복제 가능성** | 복제 불가 (단일 소유자) | 여러 개 복제 가능 |
| **결과 처리** | 완료된 태스크의 결과 직접 수집 | 메시지 응답으로 결과 수신 |

### 코드로 보는 JoinSet 활용

```rust
// Actor 내부에서 JoinSet 활용 예제
struct WorkerManagerActor {
    workers: JoinSet<Result<WorkerOutput, WorkerError>>,
    pending_tasks: usize,
}

impl Actor for WorkerManagerActor {}

impl WorkerManagerActor {
    fn new() -> Self {
        Self {
            workers: JoinSet::new(),
            pending_tasks: 0,
        }
    }
    
    // 새 작업 추가
    fn add_task(&mut self, task_data: TaskData) {
        self.workers.spawn(async move {
            // 비동기 작업 수행
            process_task(task_data).await
        });
        self.pending_tasks += 1;
    }
    
    // 완료된 작업 결과 수집
    async fn collect_results(&mut self) -> Vec<WorkerOutput> {
        let mut results = Vec::new();
        
        while let Some(result) = self.workers.join_next().await {
            match result {
                Ok(Ok(output)) => results.push(output),
                Ok(Err(e)) => println!("Worker error: {:?}", e),
                Err(e) => println!("Task join error: {:?}", e),
            }
            self.pending_tasks -= 1;
        }
        
        results
    }
}
```

### JoinSet의 주요 이점

1. **리소스 관리 자동화**: 태스크가 완료되면 자동으로 리소스 정리
2. **병렬 처리 간소화**: 여러 비동기 작업을 쉽게 생성하고 관리
3. **결과 수집 용이성**: `join_next()`를 통해 완료된 작업의 결과를 순차적으로 수집
4. **취소 처리**: `JoinSet`이 드롭되면 모든 태스크가 자동으로 취소됨

### Actor 모델에서 JoinSet의 활용

1. **백그라운드 작업 관리**: Actor 내부에서 여러 백그라운드 작업 실행 및 관리
2. **팬아웃 패턴**: 하나의 요청을 여러 병렬 작업으로 분할하여 처리
3. **자원 정리**: Actor가 종료될 때 모든 관련 태스크도 함께 정리
4. **비동기 작업 조율**: 여러 비동기 작업의 진행 상황과 결과를 조율

JoinSet은 Actor 모델의 핵심 구성 요소는 아니지만, Actor 내부에서 여러 비동기 작업을 효율적으로 관리하는 데 매우 유용한 도구입니다. Context와 Address가 Actor 모델의 통신과 생명주기를 담당한다면, JoinSet은 Actor 내부의 병렬 작업 관리를 위한 보조 도구로 볼 수 있습니다.
