# 호가창 전략 (Orderbook Strategy)

## 📊 개요

CEX 거래소의 초기 유동성 확보와 호가창 구성 전략을 정리한 문서입니다.

---

## 🎯 핵심 전략

### **바이낸스 데이터 기반 + 우리 엔진 처리**

```
시장 데이터 (가격, 호가) → 바이낸스 API 활용
실제 거래 (주문, 체결)   → 우리 엔진 처리

= 데이터는 바이낸스, 로직은 우리 것!
```

---

## 🔥 문제 인식: 왜 이 전략이 필요한가?

### **초기 거래소의 딜레마**

```
신규 거래소 오픈
  ↓
호가창이 텅 비어있음
  ↓
유저: "거래할 수 없네? 이상한 거래소"
  ↓
유저 이탈 💀
```

### **해결책: 봇으로 초기 유동성 제공**

- 바이낸스 호가를 봇이 복사
- 실제로 주문을 생성
- 유저가 거래 가능한 환경 조성

---

## 📋 데이터 레이어 구분

### **1. 시장 데이터 (Market Data)**

바이낸스에서 가져오는 정보:

```
- 현재가 (Current Price)
- 호가창 (Orderbook / Depth)
- 차트 데이터 (Klines / Candles)
- 24시간 거래량 (Volume)
- 최고/최저가 (High/Low)
```

**API 예시:**
```
GET /api/cex/market-data/price?pair=SOL/USDT
GET /api/cex/market-data/orderbook?pair=SOL/USDT&depth=20
GET /api/cex/market-data/klines?pair=SOL/USDT&interval=1m
GET /api/cex/market-data/24h-stats?pair=SOL/USDT
```

**특징:**
- 바이낸스 데이터를 그대로 또는 캐싱해서 리턴
- 우리 서버는 **Proxy 역할**
- 가격 일관성 보장

---

### **2. 거래 데이터 (Trading Data)**

우리 엔진이 처리하는 정보:

```
- 주문 생성/취소 (Orders)
- 체결 내역 (Trades)
- 잔고 관리 (Balances)
- 내 주문 조회 (My Orders)
```

**API 예시:**
```
POST /api/cex/orders              # 주문 생성
DELETE /api/cex/orders/:id        # 주문 취소
GET /api/cex/orders/my            # 내 주문
GET /api/cex/trades               # 체결 내역
```

**특징:**
- 우리 엔진이 100% 처리
- WAL + DB 저장
- 실제 비즈니스 로직

---

## 🤖 봇 전략

### **옵션 1: 바이낸스 호가 그대로 복사 (기본)**

```rust
async fn bot_sync_orderbook() {
    loop {
        // 1. 바이낸스 호가 가져오기
        let binance = fetch_binance_orderbook("SOLUSDT", 10).await?;
        
        // 2. 우리 기존 봇 주문 전부 취소
        cancel_all_bot_orders().await?;
        
        // 3. 바이낸스 호가 그대로 복사
        for bid in binance.bids.iter().take(10) {
            bot_account.create_order(
                OrderType::Buy,
                bid.price,
                bid.quantity * 0.1  // 수량은 10%만
            ).await?;
        }
        
        for ask in binance.asks.iter().take(10) {
            bot_account.create_order(
                OrderType::Sell,
                ask.price,
                ask.quantity * 0.1
            ).await?;
        }
        
        // 4. 10초마다 갱신
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}
```

**결과:**
```
우리 호가창 ≈ 바이낸스 호가창 (거의 동일)
+ 유저의 실제 주문도 함께 표시됨!
```

---

### **옵션 2: Market Maker (스프레드 전략)**

```rust
async fn bot_market_maker() {
    loop {
        // 1. 바이낸스 현재가
        let binance_price = fetch_binance_price("SOLUSDT").await?;
        
        // 2. 스프레드 추가
        let buy_price = binance_price * 0.995;   // -0.5%
        let sell_price = binance_price * 1.005;  // +0.5%
        
        // 3. 우리 거래소에 주문
        bot.create_order(OrderType::Buy, buy_price, 10.0).await?;
        bot.create_order(OrderType::Sell, sell_price, 10.0).await?;
        
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}
```

**결과:**
```
바이낸스: 100.5
우리:
  매수: 99.995 (-0.5%)  ← 봇
  매도: 100.995 (+0.5%) ← 봇
  
중간 갭: 1% (봇 수익)
```

---

## 🎯 프론트엔드 연동

### **핵심: 프론트는 "우리 서버"만 봐야 함**

```typescript
// ✅ 올바른 방법
const orderbook = await fetch('/api/cex/orderbook?pair=SOL/USDT');

// 우리 호가창:
// - 봇 주문 (바이낸스 기반)
// - 유저 주문 (실제)
// 둘 다 포함!


// ❌ 잘못된 방법
const orderbook = await fetch('https://api.binance.com/api/v3/depth?symbol=SOLUSDT');

// 문제:
// - 유저 주문이 안 보임
// - 우리 거래소와 별개
```

---

## 🔄 데이터 동기화

### **방법 1: Polling (간단, 추천)**

```typescript
// 프론트엔드
useEffect(() => {
  const fetchOrderbook = async () => {
    const res = await fetch('/api/cex/orderbook?pair=SOL/USDT');
    const data = await res.json();
    setOrderbook(data);
  };
  
  // 1초마다 갱신
  const interval = setInterval(fetchOrderbook, 1000);
  
  return () => clearInterval(interval);
}, []);
```

**장점:**
- 구현 10분 컷
- 디버깅 쉬움
- 1초면 충분히 빠름

---

### **방법 2: WebSocket (나중에, 선택)**

```typescript
// 프론트엔드
const ws = new WebSocket('ws://localhost:3002/ws/orderbook/SOL-USDT');

ws.onmessage = (event) => {
  const orderbook = JSON.parse(event.data);
  setOrderbook(orderbook);  // 실시간 업데이트!
};
```

```rust
// 백엔드
use axum::extract::ws::{WebSocket, WebSocketUpgrade};

async fn ws_orderbook_handler(
    ws: WebSocketUpgrade,
    State(engine): State<Arc<dyn Engine>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_orderbook_ws(socket, engine))
}

async fn handle_orderbook_ws(
    mut socket: WebSocket,
    engine: Arc<dyn Engine>,
) {
    loop {
        // 엔진에서 호가창 가져오기
        let orderbook = engine.get_orderbook(&pair, Some(20)).await?;
        
        // JSON으로 전송
        let msg = serde_json::to_string(&orderbook)?;
        socket.send(Message::Text(msg)).await?;
        
        // 0.5초마다
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}
```

**장점:**
- 진짜 실시간
- 포트폴리오 과시

---

## 📊 차트는?

### **TradingView + 바이낸스 (추천)**

```html
<!-- 프론트엔드 -->
<script src="https://s3.tradingview.com/tv.js"></script>
<script>
  new TradingView.widget({
    symbol: "BINANCE:SOLUSDT",  // 바이낸스 데이터 사용
    interval: "1",
    container_id: "chart",
    // ...
  });
</script>
```

**이유:**
- TradingView가 바이낸스 연동 제공
- 우리가 구현 안 해도 됨
- 전문적으로 보임

**우리 데이터로 차트 만들기는 부담!** ✅

---

## 🎯 최종 아키텍처

```
┌───────────────────────────────────────────────────────┐
│                   프론트엔드                           │
│                                                        │
│  ┌─────────────┐  ┌─────────────┐  ┌──────────────┐ │
│  │   호가창    │  │   차트      │  │   내 주문    │ │
│  └─────────────┘  └─────────────┘  └──────────────┘ │
│        ↓               ↓                  ↓          │
└────────┼───────────────┼──────────────────┼──────────┘
         ↓               ↓                  ↓
    우리 API        TradingView         우리 API
         ↓          (바이낸스)             ↓
    우리 엔진                          우리 DB
         ↓
    ┌─────────┐
    │ 봇 주문 │ (바이낸스 기반)
    │ 유저주문│ (실제)
    └─────────┘
```

---

## 💡 구현 우선순위

### **1단계: 기본 API (지금)**

```
✅ POST /api/cex/orders        # 주문 생성
✅ GET /api/cex/orders/my      # 내 주문
✅ GET /api/cex/orderbook      # 호가창 (엔진에서)
✅ GET /api/cex/trades         # 체결 내역
```

---

### **2단계: 시장 데이터 API (다음)**

```
[ ] GET /api/cex/market-data/price
[ ] GET /api/cex/market-data/orderbook
[ ] GET /api/cex/market-data/klines
[ ] GET /api/cex/market-data/24h-stats

→ 바이낸스 API 클라이언트 구현
→ 캐싱 추가 (Redis 또는 메모리)
```

---

### **3단계: 봇 구현 (그 다음)**

```
[ ] 봇 계정 생성
[ ] 바이낸스 호가 → 우리 주문 변환
[ ] 주기적 동기화 (10초마다)
[ ] 리스크 관리 (최대 금액 제한)
```

---

### **4단계: WebSocket (선택, 나중에)**

```
[ ] ws://localhost:3002/ws/orderbook/:pair
[ ] ws://localhost:3002/ws/my-orders
[ ] 실시간 푸시 구현
```

---

## 🔧 바이낸스 API 엔드포인트

### **호가창 (Depth)**

```
GET https://api.binance.com/api/v3/depth
Parameters:
  - symbol: SOLUSDT
  - limit: 5, 10, 20, 50, 100, 500, 1000

Response:
{
  "bids": [
    ["100.50", "10.5"],  // [가격, 수량]
    ["100.45", "5.2"],
    ...
  ],
  "asks": [
    ["100.55", "8.3"],
    ["100.60", "12.1"],
    ...
  ]
}
```

---

### **현재가 (Ticker Price)**

```
GET https://api.binance.com/api/v3/ticker/price
Parameters:
  - symbol: SOLUSDT

Response:
{
  "symbol": "SOLUSDT",
  "price": "100.52"
}
```

---

### **24시간 통계**

```
GET https://api.binance.com/api/v3/ticker/24hr
Parameters:
  - symbol: SOLUSDT

Response:
{
  "symbol": "SOLUSDT",
  "priceChange": "-2.50",
  "priceChangePercent": "-2.43",
  "lastPrice": "100.52",
  "volume": "123456.78",  // 거래량 (SOL)
  "quoteVolume": "12345678.90",  // 거래량 (USDT)
  "highPrice": "105.00",
  "lowPrice": "98.00"
}
```

---

### **캔들 데이터 (Klines)**

```
GET https://api.binance.com/api/v3/klines
Parameters:
  - symbol: SOLUSDT
  - interval: 1m, 5m, 15m, 1h, 4h, 1d
  - limit: 100

Response: [
  [
    1499040000000,      // 시작 시간
    "100.00",           // 시가
    "101.00",           // 고가
    "99.50",            // 저가
    "100.50",           // 종가
    "1000.00",          // 거래량
    ...
  ],
  ...
]
```

---

## 🚀 구현 예시

### **바이낸스 클라이언트**

```rust
// src/shared/clients/binance.rs

use reqwest::Client;
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use anyhow::{Context, Result};

pub struct BinanceClient {
    http_client: Client,
    base_url: String,
}

impl BinanceClient {
    pub fn new() -> Self {
        Self {
            http_client: Client::new(),
            base_url: "https://api.binance.com".to_string(),
        }
    }
    
    /// 호가창 조회
    pub async fn get_orderbook(
        &self,
        symbol: &str,
        limit: Option<u32>,
    ) -> Result<BinanceOrderbook> {
        let url = format!(
            "{}/api/v3/depth?symbol={}&limit={}",
            self.base_url,
            symbol,
            limit.unwrap_or(20)
        );
        
        let response = self.http_client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch Binance orderbook")?;
        
        let orderbook: BinanceOrderbook = response
            .json()
            .await
            .context("Failed to parse Binance orderbook")?;
        
        Ok(orderbook)
    }
    
    /// 현재가 조회
    pub async fn get_price(&self, symbol: &str) -> Result<Decimal> {
        let url = format!(
            "{}/api/v3/ticker/price?symbol={}",
            self.base_url,
            symbol
        );
        
        let response = self.http_client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch Binance price")?;
        
        let price_data: BinancePrice = response
            .json()
            .await
            .context("Failed to parse Binance price")?;
        
        Ok(price_data.price)
    }
}

#[derive(Debug, Deserialize)]
pub struct BinanceOrderbook {
    pub bids: Vec<[String; 2]>,  // [가격, 수량]
    pub asks: Vec<[String; 2]>,
}

#[derive(Debug, Deserialize)]
pub struct BinancePrice {
    pub symbol: String,
    #[serde(with = "rust_decimal::serde::str")]
    pub price: Decimal,
}
```

---

### **Market Data API Handler**

```rust
// src/domains/cex/handlers/market_data_handler.rs

use axum::{extract::State, Json, extract::Query};
use serde::Deserialize;
use anyhow::Result;

#[derive(Deserialize)]
pub struct OrderbookQuery {
    pair: String,
    depth: Option<u32>,
}

/// 시장 호가창 조회 (바이낸스 기반)
pub async fn get_market_orderbook(
    State(binance): State<Arc<BinanceClient>>,
    Query(params): Query<OrderbookQuery>,
) -> Result<Json<BinanceOrderbook>> {
    // SOL/USDT → SOLUSDT 변환
    let symbol = params.pair.replace("/", "");
    
    let orderbook = binance
        .get_orderbook(&symbol, params.depth)
        .await?;
    
    Ok(Json(orderbook))
}
```

---

### **봇 주문 생성**

```rust
// src/domains/cex/bots/market_maker.rs

pub struct MarketMakerBot {
    engine: Arc<dyn Engine>,
    binance: Arc<BinanceClient>,
    bot_user_id: u64,
}

impl MarketMakerBot {
    pub async fn run(&self) {
        loop {
            if let Err(e) = self.sync_orderbook("SOL", "USDT").await {
                eprintln!("[BOT ERROR] {}", e);
            }
            
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    }
    
    async fn sync_orderbook(
        &self,
        base: &str,
        quote: &str,
    ) -> Result<()> {
        // 1. 바이낸스 호가 가져오기
        let symbol = format!("{}{}", base, quote);
        let binance_book = self.binance.get_orderbook(&symbol, Some(10)).await?;
        
        // 2. 기존 봇 주문 취소 (TODO)
        
        // 3. 새 봇 주문 생성
        for bid in binance_book.bids.iter().take(5) {
            let price = Decimal::from_str(&bid[0])?;
            let amount = Decimal::from_str(&bid[1])? * Decimal::new(1, 1); // 10%
            
            let order_entry = OrderEntry {
                id: generate_id(),
                user_id: self.bot_user_id,
                order_type: "buy".to_string(),
                order_side: "limit".to_string(),
                base_mint: base.to_string(),
                quote_mint: quote.to_string(),
                price: Some(price),
                amount,
                filled_amount: Decimal::ZERO,
                remaining_amount: amount,
                created_at: Utc::now(),
            };
            
            self.engine.submit_order(order_entry).await?;
        }
        
        // 매도 주문도 동일하게
        // ...
        
        Ok(())
    }
}
```

---

## ⚠️ 주의사항

### **1. 법적 리스크**

```
포트폴리오 / 테스트넷: ✅ 괜찮음
  - 실제 돈 없음
  - "Demo" 표시

실제 서비스: ⚠️ 주의
  - 허수 호가 = 시세 조종 (불법)
  - 실제 자금 + 실제 체결 필요
  - Market Maker로 해야 합법
```

---

### **2. 봇 자금 관리**

```
봇이 주문을 넣으려면:
  매수 봇: USDT 잔고 필요
  매도 봇: SOL 잔고 필요

초기 자금:
  - 테스트: DB에 임의로 넣기
  - 실제: 실제 입금 필요
```

---

### **3. 리스크 관리**

```rust
// 봇 주문 제한
const MAX_BOT_ORDER_AMOUNT: Decimal = Decimal::new(100, 0);  // 100 SOL
const MAX_BOT_ORDER_VALUE: Decimal = Decimal::new(10000, 0); // 10,000 USDT

// 체결 시 손실 제한
if bot_loss > MAX_LOSS {
    disable_bot();
}
```

---

## 🎓 최종 정리

### **당신의 전략: 100% 올바름!** ✅

```
바이낸스 호가 → 봇이 그대로 주문

장점:
✅ 가격 일관성 (호가창 = 차트 = 현재가)
✅ 구현 간단
✅ 유동성 확보
✅ 포트폴리오용으로 적합

주의:
⚠️ 차트는 바이낸스 (TradingView 사용)
⚠️ 봇 자금 필요 (테스트는 DB에 넣기)
⚠️ 프론트는 "우리 서버" 호가창 봐야 함
```

---

## 🚀 구현 순서

```
1. 바이낸스 클라이언트 구현
2. Market Data API 추가
3. 봇 주문 생성 로직
4. 주기적 동기화 (10초)
5. 프론트 연동 (Polling)
6. (선택) WebSocket 추가
```

---

## 💼 포트폴리오 설명

**면접관에게:**

```
"초기 유동성 확보를 위해 Market Data Bootstrapping 전략 사용

- 바이낸스 API 연동하여 실시간 시장 데이터 수집
- 봇 계정이 바이낸스 호가를 기반으로 초기 유동성 제공
- 실제 유저 주문과 함께 표시되어 자연스러운 거래 환경 조성

핵심 기술:
- Binance REST API Integration
- Orderbook Synchronization
- Automated Market Making
- Real-time Data Aggregation"
```

**멋있게 들립니다!** 🔥

---

**이제 바이낸스 클라이언트 구현할까요?** 🚀

