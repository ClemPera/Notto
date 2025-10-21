use super::SyncResult;
use super::SyncError;
use serde::{Serialize, Deserialize};
use reqwest::{Client, StatusCode};

/// Response from CouchDB API
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CouchDbResponse {
    pub ok: Option<bool>,
    pub id: Option<String>,
    pub rev: Option<String>,
    pub error: Option<String>,
    pub reason: Option<String>,
}

/// Document to store in CouchDB
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CouchDbDocument {
    pub _id: String,
    pub _rev: Option<String>,
    pub note_id: String,
    pub user_id: String,
    pub encrypted_content: String, // Base64 encoded encrypted note
    pub title: String,
    pub updated_at: String,
    pub sync_version: u32,
}

/// Check if CouchDB server is reachable
pub async fn check_server_connectivity(server_url: &str) -> SyncResult<bool> {
    let client = Client::new();
    match client.get(format!("{}/", server_url))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
    {
        Ok(response) => Ok(response.status().is_success()),
        Err(_) => Ok(false),
    }
}

/// Authenticate with CouchDB server
pub async fn authenticate(
    server_url: &str,
    username: &str,
    password: &str,
) -> SyncResult<String> {
    let client = Client::new();
    let login_url = format!("{}/_session", server_url);

    #[derive(Serialize)]
    struct LoginRequest {
        name: String,
        password: String,
    }

    let response = client
        .post(&login_url)
        .json(&LoginRequest {
            name: username.to_string(),
            password: password.to_string(),
        })
        .send()
        .await
        .map_err(|e| SyncError::HttpError(e.to_string()))?;

    match response.status() {
        StatusCode::OK => {
            let body = response
                .text()
                .await
                .map_err(|e| SyncError::HttpError(e.to_string()))?;

            // Extract session cookie or token from response
            // For now, return a simple auth token
            Ok(format!("session_{}", username))
        }
        StatusCode::UNAUTHORIZED => Err(SyncError::AuthenticationError),
        _ => Err(SyncError::ServerError(format!(
            "Auth failed with status {}",
            response.status()
        ))),
    }
}

/// Get or create user database in CouchDB
pub async fn get_user_database(
    server_url: &str,
    auth_token: &str,
    username: &str,
) -> SyncResult<String> {
    let db_name = format!("userdb-{}", username);
    let client = Client::new();
    let db_url = format!("{}/{}", server_url, db_name);

    // Try to get the database
    let response = client
        .head(&db_url)
        .header("Authorization", format!("Bearer {}", auth_token))
        .send()
        .await
        .map_err(|e| SyncError::HttpError(e.to_string()))?;

    if response.status() == StatusCode::OK {
        Ok(db_name)
    } else if response.status() == StatusCode::NOT_FOUND {
        // Create the database if it doesn't exist
        create_user_database(server_url, auth_token, &db_name).await?;
        Ok(db_name)
    } else {
        Err(SyncError::ServerError(format!(
            "Failed to access database with status {}",
            response.status()
        )))
    }
}

/// Create a new database for the user
async fn create_user_database(
    server_url: &str,
    auth_token: &str,
    db_name: &str,
) -> SyncResult<()> {
    let client = Client::new();
    let db_url = format!("{}/{}", server_url, db_name);

    let response = client
        .put(&db_url)
        .header("Authorization", format!("Bearer {}", auth_token))
        .send()
        .await
        .map_err(|e| SyncError::HttpError(e.to_string()))?;

    match response.status() {
        StatusCode::CREATED | StatusCode::OK => Ok(()),
        StatusCode::UNAUTHORIZED => Err(SyncError::AuthenticationError),
        _ => Err(SyncError::ServerError(format!(
            "Failed to create database with status {}",
            response.status()
        ))),
    }
}

/// Upload a document to CouchDB
pub async fn upload_document(
    server_url: &str,
    auth_token: &str,
    db_name: &str,
    doc: &CouchDbDocument,
) -> SyncResult<String> {
    let client = Client::new();
    let doc_url = format!("{}/{}/{}", server_url, db_name, &doc._id);

    let response = client
        .put(&doc_url)
        .header("Authorization", format!("Bearer {}", auth_token))
        .header("Content-Type", "application/json")
        .json(doc)
        .send()
        .await
        .map_err(|e| SyncError::HttpError(e.to_string()))?;

    match response.status() {
        StatusCode::CREATED | StatusCode::OK => {
            let body: CouchDbResponse = response
                .json()
                .await
                .map_err(|e| SyncError::JsonError(e.to_string()))?;

            body.rev.ok_or_else(|| SyncError::InvalidResponse("No revision in response".to_string()))
        }
        StatusCode::UNAUTHORIZED => Err(SyncError::AuthenticationError),
        StatusCode::CONFLICT => Err(SyncError::ConflictError("Document conflict".to_string())),
        _ => {
            let body: Result<CouchDbResponse, _> = response.json().await;
            if let Ok(err_response) = body {
                Err(SyncError::ServerError(format!(
                    "{}: {}",
                    err_response.error.unwrap_or_default(),
                    err_response.reason.unwrap_or_default()
                )))
            } else {
                Err(SyncError::ServerError("Unknown error".to_string()))
            }
        }
    }
}

/// Download a document from CouchDB
pub async fn download_document(
    server_url: &str,
    auth_token: &str,
    db_name: &str,
    doc_id: &str,
) -> SyncResult<CouchDbDocument> {
    let client = Client::new();
    let doc_url = format!("{}/{}/{}", server_url, db_name, doc_id);

    let response = client
        .get(&doc_url)
        .header("Authorization", format!("Bearer {}", auth_token))
        .send()
        .await
        .map_err(|e| SyncError::HttpError(e.to_string()))?;

    match response.status() {
        StatusCode::OK => {
            response
                .json::<CouchDbDocument>()
                .await
                .map_err(|e| SyncError::JsonError(e.to_string()))
        }
        StatusCode::NOT_FOUND => Err(SyncError::ServerError("Document not found".to_string())),
        StatusCode::UNAUTHORIZED => Err(SyncError::AuthenticationError),
        _ => Err(SyncError::ServerError(format!(
            "Failed to download document with status {}",
            response.status()
        ))),
    }
}

/// Get changes feed from CouchDB (_changes feed)
#[derive(Debug, Serialize, Deserialize)]
pub struct ChangesResponse {
    pub results: Vec<ChangeEntry>,
    pub last_seq: String,
    pub pending: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChangeEntry {
    pub seq: String,
    pub id: String,
    pub changes: Vec<ChangeInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChangeInfo {
    pub rev: String,
}

pub async fn get_changes(
    server_url: &str,
    auth_token: &str,
    db_name: &str,
    since: Option<String>,
) -> SyncResult<ChangesResponse> {
    let client = Client::new();
    let changes_url = format!("{}/_db/{}_changes", server_url, db_name);

    let mut request = client.get(&changes_url)
        .header("Authorization", format!("Bearer {}", auth_token))
        .query(&[("include_docs", "true")]);

    if let Some(seq) = since {
        request = request.query(&[("since", &seq)]);
    }

    let response = request
        .send()
        .await
        .map_err(|e| SyncError::HttpError(e.to_string()))?;

    match response.status() {
        StatusCode::OK => {
            response
                .json::<ChangesResponse>()
                .await
                .map_err(|e| SyncError::JsonError(e.to_string()))
        }
        StatusCode::UNAUTHORIZED => Err(SyncError::AuthenticationError),
        _ => Err(SyncError::ServerError(format!(
            "Failed to get changes with status {}",
            response.status()
        ))),
    }
}

/// Delete a document from CouchDB
pub async fn delete_document(
    server_url: &str,
    auth_token: &str,
    db_name: &str,
    doc_id: &str,
    rev: &str,
) -> SyncResult<()> {
    let client = Client::new();
    let doc_url = format!("{}/{}/{}?rev={}", server_url, db_name, doc_id, rev);

    let response = client
        .delete(&doc_url)
        .header("Authorization", format!("Bearer {}", auth_token))
        .send()
        .await
        .map_err(|e| SyncError::HttpError(e.to_string()))?;

    match response.status() {
        StatusCode::OK => Ok(()),
        StatusCode::UNAUTHORIZED => Err(SyncError::AuthenticationError),
        StatusCode::NOT_FOUND => Err(SyncError::ServerError("Document not found".to_string())),
        _ => Err(SyncError::ServerError(format!(
            "Failed to delete document with status {}",
            response.status()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_couch_db_document_serialization() {
        let doc = CouchDbDocument {
            _id: "note_123".to_string(),
            _rev: Some("1-abc".to_string()),
            note_id: "123".to_string(),
            user_id: "user_456".to_string(),
            encrypted_content: "base64_encrypted_data".to_string(),
            title: "My Note".to_string(),
            updated_at: "2025-10-21T00:00:00Z".to_string(),
            sync_version: 1,
        };

        let json = serde_json::to_string(&doc).unwrap();
        let decoded: CouchDbDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded._id, doc._id);
    }
}
