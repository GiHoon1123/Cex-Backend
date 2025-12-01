/// Bot 모듈
/// Bot Module
/// 
/// 역할:
/// - 바이낸스 오더북을 실시간으로 수신
/// - 봇 계정(bot1, bot2)을 통해 동일한 지정가 주문 생성
/// 
/// 구조:
/// - `services/binance_client.rs`: 바이낸스 WebSocket 클라이언트
/// - `services/bot_manager.rs`: 봇 계정 관리, 주문 생성/취소
/// - `services/orderbook_sync.rs`: 오더북 동기화 서비스
/// - `models/config.rs`: 봇 설정 (주문 수량 등)
pub mod models;
pub mod services;
pub mod handlers;
pub mod routes;

