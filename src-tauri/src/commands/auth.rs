use crate::AppState;
use crate::auth;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub user_id: String,
    pub recovery_phrase: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct SetupTotpRequest {
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct SetupTotpResponse {
    pub secret: String,
    pub backup_codes: Vec<String>,
    pub qr_code_uri: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyTotpRequest {
    pub token: String,
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct LogoutResponse {
    pub success: bool,
}

/// Register a new user account
#[tauri::command]
pub async fn register(
    state: tauri::State<'_, AppState>,
    req: RegisterRequest,
) -> Result<RegisterResponse, String> {
    let (user_id, recovery_phrase) = auth::register(&state.db, &req.username, &req.password)
        .map_err(|e| e.to_string())?;

    Ok(RegisterResponse {
        user_id,
        recovery_phrase,
    })
}

/// Login user with username and password
#[tauri::command]
pub async fn login(
    state: tauri::State<'_, AppState>,
    req: LoginRequest,
) -> Result<LoginResponse, String> {
    let token = auth::login(&state.db, &req.username, &req.password)
        .map_err(|e| e.to_string())?;

    Ok(LoginResponse { token })
}

/// Setup TOTP 2FA for user
#[tauri::command]
pub async fn setup_totp(
    state: tauri::State<'_, AppState>,
    req: SetupTotpRequest,
) -> Result<SetupTotpResponse, String> {
    // Verify session first
    let user_id = auth::verify_session(&state.db, &req.token)
        .map_err(|e| e.to_string())?;

    // Generate TOTP secret and backup codes
    let secret = auth::totp::generate_totp_secret();
    let backup_codes = auth::totp::generate_backup_codes(10);

    // Generate QR code URI for authenticator app
    let qr_code_uri = auth::totp::generate_totp_qr_code(&user_id, &secret, "Notto")
        .map_err(|e| format!("Failed to generate QR code: {}", e))?;

    Ok(SetupTotpResponse {
        secret,
        backup_codes,
        qr_code_uri,
    })
}

/// Verify TOTP code and complete 2FA setup
#[tauri::command]
pub async fn verify_totp_setup(
    state: tauri::State<'_, AppState>,
    token: String,
    secret: String,
    code: String,
) -> Result<bool, String> {
    // Verify session
    auth::verify_session(&state.db, &token)
        .map_err(|e| e.to_string())?;

    // Verify TOTP code
    auth::totp::verify_totp_code(&secret, &code)
        .map_err(|e| format!("Failed to verify TOTP: {}", e))
}

/// Verify session token
#[tauri::command]
pub async fn verify_session_token(
    state: tauri::State<'_, AppState>,
    token: String,
) -> Result<String, String> {
    auth::verify_session(&state.db, &token)
        .map_err(|e| e.to_string())
}

/// Logout user
#[tauri::command]
pub async fn logout(
    state: tauri::State<'_, AppState>,
    user_id: String,
) -> Result<LogoutResponse, String> {
    auth::logout(&state.db, &user_id)
        .map_err(|e| e.to_string())?;

    Ok(LogoutResponse { success: true })
}

