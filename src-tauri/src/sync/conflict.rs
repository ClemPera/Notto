use super::SyncError;
use super::SyncResult;
use crate::models::Note;
use chrono::Utc;
use sha2::{Digest, Sha256};

/// Conflict detection strategy
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConflictStrategy {
    /// Last write wins - remote version overwrites local
    LastWriteWins,
    /// Keep local version (local wins)
    KeepLocal,
    /// Create both versions side-by-side with timestamps
    CreateBoth,
}

/// Detect if there's a conflict between local and remote versions
pub fn detect_conflict(local: &Note, remote: &Note) -> SyncResult<bool> {
    // Conflict exists if:
    // 1. Both have different sync_versions (different modification histories)
    // 2. Both have different updated_at times
    // 3. Content hash differs

    if local.sync_version != remote.sync_version {
        return Ok(true);
    }

    if local.updated_at != remote.updated_at {
        return Ok(true);
    }

    // Check content hash
    let local_hash = hash_content(&local.content);
    let remote_hash = hash_content(&remote.content);

    Ok(local_hash != remote_hash)
}

/// Resolve a conflict between two versions
pub fn resolve_conflict(
    local: Note,
    remote: Note,
    strategy: ConflictStrategy,
) -> SyncResult<(Note, Option<Note>)> {
    match strategy {
        ConflictStrategy::LastWriteWins => {
            // Use whichever is newer
            if remote.updated_at > local.updated_at {
                Ok((remote, None))
            } else {
                Ok((local, None))
            }
        }
        ConflictStrategy::KeepLocal => {
            // Always keep local version
            Ok((local, None))
        }
        ConflictStrategy::CreateBoth => {
            // Create a conflict copy of the remote version
            let mut conflict_remote = remote.clone();
            conflict_remote.id = format!(
                "{}_conflict_{}_{}",
                remote.id,
                remote.sync_version,
                Utc::now().timestamp()
            );
            conflict_remote.title = format!("{} (conflict - remote)", remote.title);

            Ok((local, Some(conflict_remote)))
        }
    }
}

/// Compute hash of note content for comparison
pub fn hash_content(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Three-way merge strategy for text content
pub fn three_way_merge(base: &str, local: &str, remote: &str) -> SyncResult<String> {
    // Simple merge: if both made the same change, apply it
    // If one side modified and the other didn't, apply the modification
    // If both modified differently, keep local (conservative approach)

    if local == remote {
        return Ok(local.to_string());
    }

    if local == base {
        // Only remote changed
        return Ok(remote.to_string());
    }

    if remote == base {
        // Only local changed
        return Ok(local.to_string());
    }

    // Both changed differently - keep local to avoid data loss
    Ok(local.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn create_test_note(id: &str, content: &str, version: u32) -> Note {
        Note {
            id: id.to_string(),
            title: "Test".to_string(),
            content: content.to_string(),
            folder_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            is_encrypted: true,
            sync_version: version,
        }
    }

    #[test]
    fn test_detect_conflict_different_versions() {
        let local = create_test_note("note1", "content1", 1);
        let remote = create_test_note("note1", "content1", 2);

        let has_conflict = detect_conflict(&local, &remote).unwrap();
        assert!(has_conflict);
    }

    #[test]
    fn test_detect_conflict_different_content() {
        let mut local = create_test_note("note1", "content1", 1);
        let mut remote = create_test_note("note1", "content2", 1);

        // Manually set same timestamp to isolate content difference
        local.updated_at = Utc::now();
        remote.updated_at = local.updated_at;

        let has_conflict = detect_conflict(&local, &remote).unwrap();
        assert!(has_conflict);
    }

    #[test]
    fn test_no_conflict_identical_notes() {
        let local = create_test_note("note1", "content1", 1);
        let remote = create_test_note("note1", "content1", 1);

        let has_conflict = detect_conflict(&local, &remote).unwrap();
        assert!(!has_conflict);
    }

    #[test]
    fn test_resolve_conflict_last_write_wins() {
        let mut local = create_test_note("note1", "local", 1);
        let mut remote = create_test_note("note1", "remote", 1);

        let now = Utc::now();
        local.updated_at = now;
        remote.updated_at = now + Duration::seconds(1); // Remote is newer

        let (winner, loser) =
            resolve_conflict(local, remote.clone(), ConflictStrategy::LastWriteWins).unwrap();

        assert_eq!(winner.content, "remote");
        assert_eq!(winner.id, remote.id);
        assert!(loser.is_none());
    }

    #[test]
    fn test_resolve_conflict_create_both() {
        let local = create_test_note("note1", "local", 1);
        let remote = create_test_note("note1", "remote", 1);

        let (winner, loser) =
            resolve_conflict(local.clone(), remote, ConflictStrategy::CreateBoth).unwrap();

        assert_eq!(winner.content, "local");
        assert!(loser.is_some());

        let conflict_copy = loser.unwrap();
        assert!(conflict_copy.id.contains("conflict"));
        assert_eq!(conflict_copy.content, "remote");
    }

    #[test]
    fn test_three_way_merge_both_changed() {
        let base = "line1\nline2\nline3";
        let local = "line1_modified\nline2\nline3";
        let remote = "line1\nline2_modified\nline3";

        let merged = three_way_merge(base, local, remote).unwrap();
        // Both changed differently, keep local
        assert_eq!(merged, local);
    }

    #[test]
    fn test_three_way_merge_only_local_changed() {
        let base = "original";
        let local = "local_change";
        let remote = "original";

        let merged = three_way_merge(base, local, remote).unwrap();
        assert_eq!(merged, "local_change");
    }

    #[test]
    fn test_hash_content_consistent() {
        let content = "test content";
        let hash1 = hash_content(content);
        let hash2 = hash_content(content);
        assert_eq!(hash1, hash2);
    }
}
