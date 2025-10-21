use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Represents a note in the system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub content: String, // Encrypted markdown content
    pub folder_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_encrypted: bool,
    pub sync_version: u32, // For conflict resolution
}

/// Represents a folder for organizing notes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// User account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password_hash: String, // Argon2id hash
    pub salt: Vec<u8>,
    pub created_at: DateTime<Utc>,
}

/// Session token for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub user_id: String,
    pub token: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// TOTP 2FA configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpSecret {
    pub user_id: String,
    pub secret: String, // Base32 encoded
    pub verified: bool,
    pub backup_codes: Vec<String>,
}

/// Sync metadata for CouchDB replication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMetadata {
    pub note_id: String,
    pub last_sync: DateTime<Utc>,
    pub remote_revision: Option<String>, // CouchDB revision ID
    pub is_conflicted: bool,
    pub conflict_versions: Vec<String>, // IDs of conflicting versions
}

/// Encryption parameters for key derivation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionParams {
    pub salt: Vec<u8>,
    pub argon2_memory: u32,
    pub argon2_iterations: u32,
    pub argon2_parallelism: u32,
}

impl Default for EncryptionParams {
    fn default() -> Self {
        Self {
            salt: Vec::new(),
            argon2_memory: 19456, // 19 MB
            argon2_iterations: 2,
            argon2_parallelism: 1,
        }
    }
}

/// Error types for the application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
}

impl ApiError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}
