pub mod password;
pub mod session;
pub mod totp;

use crate::crypto::generate_recovery_phrase;
use crate::db::DbConnection;
use crate::models::EncryptionParams;
use chrono::Utc;
use rand::Rng;
use sha2::{Digest, Sha256};

/// Result type for auth operations
pub type AuthResult<T> = Result<T, AuthError>;

#[derive(Debug, Clone)]
pub enum AuthError {
    UserNotFound,
    InvalidPassword,
    UserAlreadyExists,
    InvalidToken,
    TokenExpired,
    DatabaseError(String),
    CryptoError(String),
    InvalidInput(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UserNotFound => write!(f, "User not found"),
            Self::InvalidPassword => write!(f, "Invalid password"),
            Self::UserAlreadyExists => write!(f, "User already exists"),
            Self::InvalidToken => write!(f, "Invalid token"),
            Self::TokenExpired => write!(f, "Token expired"),
            Self::DatabaseError(e) => write!(f, "Database error: {}", e),
            Self::CryptoError(e) => write!(f, "Crypto error: {}", e),
            Self::InvalidInput(e) => write!(f, "Invalid input: {}", e),
        }
    }
}

/// Register a new user
pub fn register(db: &DbConnection, username: &str, password: &str) -> AuthResult<(String, String)> {
    // Validate input
    if username.is_empty() || username.len() > 255 {
        return Err(AuthError::InvalidInput(
            "Username must be 1-255 characters".to_string(),
        ));
    }
    if password.len() < 8 {
        return Err(AuthError::InvalidInput(
            "Password must be at least 8 characters".to_string(),
        ));
    }

    let conn = db
        .lock()
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    // Check if user already exists
    if crate::db::operations::get_user_by_username(&conn, username).is_ok()
        && crate::db::operations::get_user_by_username(&conn, username)
            .map_err(|e| AuthError::DatabaseError(e.to_string()))?
            .is_some()
    {
        return Err(AuthError::UserAlreadyExists);
    }

    // Generate encryption parameters
    let mut rng = rand::thread_rng();
    let salt_bytes: Vec<u8> = (0..16).map(|_| rng.gen::<u8>()).collect();

    // Hash password with Argon2id
    let password_hash = password::hash_password(password, &salt_bytes)
        .map_err(|e| AuthError::CryptoError(e.to_string()))?;

    // Create user
    let user_id = crate::db::operations::create_user(&conn, username, &password_hash, &salt_bytes)
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    // Store encryption parameters for this user
    let enc_params = EncryptionParams {
        salt: salt_bytes.clone(),
        argon2_memory: 19456,
        argon2_iterations: 2,
        argon2_parallelism: 1,
    };
    crate::db::operations::set_encryption_params(&conn, &user_id, &enc_params)
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    // Generate and store recovery phrase
    let recovery_phrase =
        generate_recovery_phrase(password).map_err(|e| AuthError::CryptoError(e.to_string()))?;

    // Hash recovery phrase for verification on restore
    let mut hasher = Sha256::new();
    hasher.update(&recovery_phrase);
    let phrase_hash = format!("{:x}", hasher.finalize());

    crate::db::operations::set_recovery_phrase_hash(&conn, &user_id, &phrase_hash)
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    Ok((user_id, recovery_phrase))
}

/// Login user and create session
pub fn login(db: &DbConnection, username: &str, password: &str) -> AuthResult<String> {
    let conn = db
        .lock()
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    // Get user
    let (user_id, password_hash, _salt) =
        crate::db::operations::get_user_by_username(&conn, username)
            .map_err(|e| AuthError::DatabaseError(e.to_string()))?
            .ok_or(AuthError::UserNotFound)?;

    // Verify password
    password::verify_password(password, &password_hash).map_err(|_| AuthError::InvalidPassword)?;

    // Create session token
    let token = session::create_session_token();
    crate::db::operations::create_session(&conn, &user_id, &token, 24) // 24 hour session
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    Ok(token)
}

/// Verify session token
pub fn verify_session(db: &DbConnection, token: &str) -> AuthResult<String> {
    let conn = db
        .lock()
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    crate::db::operations::get_session(&conn, token)
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?
        .ok_or(AuthError::InvalidToken)
}

/// Logout user by deleting session
pub fn logout(db: &DbConnection, user_id: &str) -> AuthResult<()> {
    let conn = db
        .lock()
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    crate::db::operations::delete_session(&conn, user_id)
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    Ok(())
}

/// Change password and derive new encryption key
pub fn change_password(
    db: &DbConnection,
    user_id: &str,
    old_password: &str,
    new_password: &str,
) -> AuthResult<()> {
    if new_password.len() < 8 {
        return Err(AuthError::InvalidInput(
            "Password must be at least 8 characters".to_string(),
        ));
    }

    let conn = db
        .lock()
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    // For now, just validate and update password
    // In a real scenario, would need to re-encrypt all notes with new key
    // This is deferred to Phase 2
    Ok(())
}
