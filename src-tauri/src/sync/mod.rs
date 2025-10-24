pub mod client;
pub mod conflict;
pub mod models;

use crate::db::DbConnection;
use crate::models::Note;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Result type for sync operations
pub type SyncResult<T> = Result<T, SyncError>;

#[derive(Debug, Clone)]
pub enum SyncError {
    HttpError(String),
    JsonError(String),
    DatabaseError(String),
    ConflictError(String),
    AuthenticationError,
    ServerError(String),
    OfflineError,
    InvalidResponse(String),
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HttpError(e) => write!(f, "HTTP error: {}", e),
            Self::JsonError(e) => write!(f, "JSON error: {}", e),
            Self::DatabaseError(e) => write!(f, "Database error: {}", e),
            Self::ConflictError(e) => write!(f, "Conflict error: {}", e),
            Self::AuthenticationError => write!(f, "Authentication failed"),
            Self::ServerError(e) => write!(f, "Server error: {}", e),
            Self::OfflineError => write!(f, "Offline - cannot sync"),
            Self::InvalidResponse(e) => write!(f, "Invalid response: {}", e),
        }
    }
}

impl std::error::Error for SyncError {}

/// Sync status for display in UI
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SyncStatus {
    Idle,
    Syncing,
    Success,
    Error(String),
    Offline,
}

/// Sync metadata for a note
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    pub note_id: String,
    pub user_id: String,
    pub last_sync: Option<DateTime<Utc>>,
    pub remote_revision: Option<String>,
    pub is_conflicted: bool,
    pub conflict_versions: Vec<String>,
}

/// CouchDB Sync Client
pub struct SyncClient {
    pub server_url: String,
    pub username: String,
    pub auth_token: String,
    pub db: DbConnection,
    pub status: Arc<Mutex<SyncStatus>>,
}

impl SyncClient {
    /// Create a new sync client
    pub fn new(server_url: String, username: String, auth_token: String, db: DbConnection) -> Self {
        Self {
            server_url,
            username,
            auth_token,
            db,
            status: Arc::new(Mutex::new(SyncStatus::Idle)),
        }
    }

    /// Get current sync status
    pub async fn get_status(&self) -> SyncStatus {
        self.status.lock().await.clone()
    }

    /// Set sync status
    async fn set_status(&self, status: SyncStatus) {
        *self.status.lock().await = status;
    }

    /// Check if connection to server is available
    pub async fn check_connectivity(&self) -> SyncResult<bool> {
        client::check_server_connectivity(&self.server_url).await
    }

    /// Sync all notes with server
    pub async fn sync_all(&self) -> SyncResult<()> {
        self.set_status(SyncStatus::Syncing).await;

        match self.perform_sync().await {
            Ok(_) => {
                self.set_status(SyncStatus::Success).await;
                Ok(())
            }
            Err(e) => {
                self.set_status(SyncStatus::Error(e.to_string())).await;
                Err(e)
            }
        }
    }

    /// Perform the actual sync operation
    async fn perform_sync(&self) -> SyncResult<()> {
        // 1. Check connectivity
        if !self.check_connectivity().await? {
            return Err(SyncError::OfflineError);
        }

        // 2. Get list of local notes to sync
        let local_changes = self.get_local_changes().await?;

        // 3. Upload local changes to server
        for note in local_changes {
            self.upload_note(&note).await?;
        }

        // 4. Download remote changes from server
        let remote_changes = self.download_remote_changes().await?;

        // 5. Merge remote changes (conflict detection)
        for remote_note in remote_changes {
            self.merge_remote_note(&remote_note).await?;
        }

        Ok(())
    }

    /// Get local changes since last sync
    async fn get_local_changes(&self) -> SyncResult<Vec<Note>> {
        let conn = self
            .db
            .lock()
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

        // TODO: Query database for notes with pending sync
        Ok(Vec::new())
    }

    /// Upload a single note to server
    async fn upload_note(&self, note: &Note) -> SyncResult<()> {
        // TODO: Encrypt and upload note to CouchDB
        Ok(())
    }

    /// Download remote changes from server
    async fn download_remote_changes(&self) -> SyncResult<Vec<Note>> {
        // TODO: Fetch changes from CouchDB using _changes feed
        Ok(Vec::new())
    }

    /// Merge a remote note with local version
    async fn merge_remote_note(&self, remote_note: &Note) -> SyncResult<()> {
        // TODO: Implement conflict detection and resolution
        Ok(())
    }
}
