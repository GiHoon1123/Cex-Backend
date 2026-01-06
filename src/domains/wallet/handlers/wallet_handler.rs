use crate::domains::wallet::models::{
    CreateWalletResponse, WalletResponse, WalletsResponse,
    WalletBalanceResponse, TransferSolRequest, TransferSolResponse, TransactionStatusResponse,
};
use crate::shared::services::AppState;
use crate::shared::middleware::auth::AuthenticatedUser;
use crate::shared::errors::WalletError;
use crate::domains::cex::engine::Engine;
use axum::{extract::{Path, State}, http::StatusCode, Json};
use rust_decimal::Decimal;
use std::convert::Into;

/// 지갑 생성 핸들러
/// Create wallet handler
/// Note: user_id는 JWT 토큰에서 자동 추출됨
#[utoipa::path(
    post,
    path = "/api/wallets",
    responses(
        (status = 201, description = "Wallet created successfully", body = CreateWalletResponse),
        (status = 400, description = "Bad request (wallet already exists)"),
        (status = 401, description = "Unauthorized (missing or invalid token)"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Wallets",
    security(("BearerAuth" = []))
)]
pub async fn create_wallet(
    State(app_state): State<AppState>,
    authenticated_user: AuthenticatedUser,
) -> Result<Json<CreateWalletResponse>, (StatusCode, Json<serde_json::Value>)> {
    // JWT 토큰에서 추출한 user_id 사용
    let user_id = authenticated_user.user_id;

    // 1. 지갑 생성
    let wallet = app_state
        .wallet_state
        .wallet_service
        .create_wallet(user_id)
        .await
        .map_err(|e: WalletError| -> (StatusCode, Json<serde_json::Value>) { e.into() })?;

    // 2. 모의거래소 초기 잔액 설정
    // USDT: 10,000 (SOL은 지급하지 않음)
    let engine = app_state.engine.lock().await;
    
    // USDT 초기 잔액 설정
    engine.update_balance(
        user_id,
        "USDT",
        Decimal::new(10_000, 0),
    ).await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": format!("Failed to set initial USDT balance: {}", e)
            })),
        )
    })?;

    Ok(Json(CreateWalletResponse {
        wallet,
        message: "Wallet created successfully".to_string(),
    }))
}

/// 지갑 조회 핸들러 (ID로)
/// Get wallet by ID handler
#[utoipa::path(
    get,
    path = "/api/wallets/{id}",
    params(
        ("id" = u64, Path, description = "Wallet ID")
    ),
    responses(
        (status = 200, description = "Wallet retrieved successfully", body = WalletResponse),
        (status = 404, description = "Wallet not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Wallets"
)]
pub async fn get_wallet(
    State(app_state): State<AppState>,
    Path(wallet_id): Path<u64>,
) -> Result<Json<WalletResponse>, (StatusCode, Json<serde_json::Value>)> {
    let wallet = app_state
        .wallet_state
        .wallet_service
        .get_wallet(wallet_id)
        .await
        .map_err(|e: WalletError| -> (StatusCode, Json<serde_json::Value>) { e.into() })?;

    Ok(Json(WalletResponse { wallet }))
}

/// 사용자의 모든 지갑 조회 핸들러
/// Get all wallets for user handler
/// Note: 자신의 지갑만 조회 가능 (JWT 토큰에서 user_id 추출)
#[utoipa::path(
    get,
    path = "/api/wallets/my",
    responses(
        (status = 200, description = "Wallets retrieved successfully", body = WalletsResponse),
        (status = 401, description = "Unauthorized (missing or invalid token)"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Wallets",
    security(("BearerAuth" = []))
)]
pub async fn get_user_wallets(
    State(app_state): State<AppState>,
    authenticated_user: AuthenticatedUser,
) -> Result<Json<WalletsResponse>, (StatusCode, Json<serde_json::Value>)> {
    // JWT 토큰에서 추출한 user_id 사용
    let user_id = authenticated_user.user_id;

    let wallets = app_state
        .wallet_state
        .wallet_service
        .get_user_wallets(user_id)
        .await
        .map_err(|e: WalletError| -> (StatusCode, Json<serde_json::Value>) { e.into() })?;

    Ok(Json(WalletsResponse { wallets }))
}

/// 잔액 조회 핸들러
/// Get balance handler
#[utoipa::path(
    get,
    path = "/api/wallets/{id}/balance",
    params(
        ("id" = u64, Path, description = "Wallet ID")
    ),
    responses(
        (status = 200, description = "Balance retrieved successfully", body = WalletBalanceResponse),
        (status = 404, description = "Wallet not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Wallets"
)]
pub async fn get_balance(
    State(app_state): State<AppState>,
    Path(wallet_id): Path<u64>,
) -> Result<Json<WalletBalanceResponse>, (StatusCode, Json<serde_json::Value>)> {
    // 지갑 조회 (Public Key 가져오기)
    let wallet = app_state
        .wallet_state
        .wallet_service
        .get_wallet(wallet_id)
        .await
        .map_err(|e: WalletError| -> (StatusCode, Json<serde_json::Value>) { e.into() })?;

    // 잔액 조회
    let balance_lamports = app_state
        .wallet_state
        .wallet_service
        .get_balance(wallet_id)
        .await
        .map_err(|e: WalletError| -> (StatusCode, Json<serde_json::Value>) { e.into() })?;

    let balance_sol = app_state
        .wallet_state
        .wallet_service
        .get_balance_sol(wallet_id)
        .await
        .map_err(|e: WalletError| -> (StatusCode, Json<serde_json::Value>) { e.into() })?;

    Ok(Json(WalletBalanceResponse {
        balance_lamports,
        balance_sol,
        public_key: wallet.public_key,
    }))
}

/// SOL 전송 핸들러
/// Transfer SOL handler
/// Note: 자신의 지갑에서만 전송 가능 (JWT 토큰으로 소유권 검증)
#[utoipa::path(
    post,
    path = "/api/wallets/{id}/transfer",
    params(
        ("id" = u64, Path, description = "Wallet ID (sender)")
    ),
    request_body = TransferSolRequest,
    responses(
        (status = 200, description = "Transfer successful", body = TransferSolResponse),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized (missing or invalid token)"),
        (status = 403, description = "Forbidden (not your wallet)"),
        (status = 404, description = "Wallet not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Wallets",
    security(("BearerAuth" = []))
)]
pub async fn transfer_sol(
    State(app_state): State<AppState>,
    authenticated_user: AuthenticatedUser,
    Path(wallet_id): Path<u64>,
    Json(request): Json<TransferSolRequest>,
) -> Result<Json<TransferSolResponse>, (StatusCode, Json<serde_json::Value>)> {
    // 1. 지갑 조회
    let wallet = app_state
        .wallet_state
        .wallet_service
        .get_wallet(wallet_id)
        .await
        .map_err(|e: WalletError| -> (StatusCode, Json<serde_json::Value>) { e.into() })?;

    // 2. 소유권 검증 (토큰의 user_id와 지갑의 user_id가 일치하는지 확인)
    if wallet.user_id != authenticated_user.user_id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "You don't have permission to transfer from this wallet"
            })),
        ));
    }

    // 3. SOL 전송
    let signature = app_state
        .wallet_state
        .wallet_service
        .transfer_sol(wallet_id, &request.to_public_key, request.amount_lamports)
        .await
        .map_err(|e: WalletError| -> (StatusCode, Json<serde_json::Value>) { e.into() })?;

    Ok(Json(TransferSolResponse {
        signature,
        message: "Transfer successful".to_string(),
    }))
}

/// 트랜잭션 상태 조회 핸들러
/// Get transaction status handler
#[utoipa::path(
    get,
    path = "/api/wallets/transaction/{signature}",
    params(
        ("signature" = String, Path, description = "Transaction signature")
    ),
    responses(
        (status = 200, description = "Transaction status retrieved successfully", body = TransactionStatusResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "Wallets"
)]
pub async fn get_transaction_status(
    State(app_state): State<AppState>,
    Path(signature): Path<String>,
) -> Result<Json<TransactionStatusResponse>, (StatusCode, Json<serde_json::Value>)> {
    let status = app_state
        .wallet_state
        .wallet_service
        .get_transaction_status(&signature)
        .await
        .map_err(|e: WalletError| -> (StatusCode, Json<serde_json::Value>) { e.into() })?;

    Ok(Json(TransactionStatusResponse {
        signature,
        status,
    }))
}

