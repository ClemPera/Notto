use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Sync event that occurred
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncEvent {
    /// Local note was created
    LocalCreated { note_id: String },
    /// Local note was updated
    LocalUpdated { note_id: String },
    /// Local note was deleted
    LocalDeleted { note_id: String },
    /// Remote note was created
    RemoteCreated { note_id: String },
    /// Remote note was updated
    RemoteUpdated { note_id: String },
    /// Remote note was deleted
    RemoteDeleted { note_id: String },
    /// Conflict detected between local and remote
    Conflict {
        note_id: String,
        local_rev: u32,
        remote_rev: u32,
    },
}

/// Change tracker for local modifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalChange {
    pub note_id: String,
    pub change_type: ChangeType,
    pub timestamp: DateTime<Utc>,
    pub sync_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Created,
    Updated,
    Deleted,
}

/// Conflict entry when two versions exist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictEntry {
    pub note_id: String,
    pub local_version: ConflictVersion,
    pub remote_version: ConflictVersion,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictVersion {
    pub revision: u32,
    pub updated_at: DateTime<Utc>,
    pub content_hash: String,
}

/// Sync checkpoint for resuming interrupted syncs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncCheckpoint {
    pub user_id: String,
    pub last_sync_timestamp: DateTime<Utc>,
    pub last_remote_seq: String,
    pub last_local_version: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_event_serialization() {
        let event = SyncEvent::LocalCreated {
            note_id: "test_note".to_string(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let decoded: SyncEvent = serde_json::from_str(&json).unwrap();

        match decoded {
            SyncEvent::LocalCreated { note_id } => assert_eq!(note_id, "test_note"),
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_change_type_variants() {
        let created = ChangeType::Created;
        let updated = ChangeType::Updated;
        let deleted = ChangeType::Deleted;

        // Just verify they exist and serialize
        let _ = serde_json::to_string(&created).unwrap();
        let _ = serde_json::to_string(&updated).unwrap();
        let _ = serde_json::to_string(&deleted).unwrap();
    }
}
