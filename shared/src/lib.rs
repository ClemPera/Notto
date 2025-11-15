use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct User {
    pub id: Option<u32>,
    pub username: String,
    pub stored_password_hash: String,
    pub stored_recovery_hash: String,
    pub encrypted_mek_password: Vec<u8>,
    pub mek_password_nonce: Vec<u8>,
    pub encrypted_mek_recovery: Vec<u8>,
    pub mek_recovery_nonce: Vec<u8>,
    pub salt_auth: String,
    pub salt_data: String,
    pub salt_recovery_auth: String,
    pub salt_recovery_data: String,
    pub salt_server_auth: String,
    pub salt_server_recovery: String,
}