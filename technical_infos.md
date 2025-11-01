## Phase 1
### Security


| var_name | stored on client | stored on server | description |
|--------------------------------|------------------|------------------|-------------|
| master_encryption_key | ✓ | | Random key used to encrypt all user data |
| recovery_key_auth | ✓ | | Random 25 words for account recovery |
| recovery_key_data | ✓ | | Random 25 different words for data recovery |
| salt_auth | ✓ | ✓ | Salt for deriving password_hash_auth |
| salt_data | ✓ | ✓ | Salt for deriving password_hash_data |
| salt_recovery_auth | ✓ | ✓ | Salt for deriving recovery_hash_auth |
| salt_recovery_data | ✓ | ✓ | Salt for deriving recovery_hash_data |
| salt_server_auth | ✓ | ✓ | Salt for hashing password_hash_auth before server storage |
| salt_server_recovery | ✓ | ✓ | Salt for hashing recovery_hash_auth before server storage |
| nonce_mek_password | ✓ | ✓ | Nonce for AES-GCM encryption of MEK with password |
| nonce_mek_recovery | ✓ | ✓ | Nonce for AES-GCM encryption of MEK with recovery key |
| password_hash_auth | ✓ | | argon2id(password, salt_auth) - used for authentication |
| password_hash_data | ✓ | | argon2id(password, salt_data) - used to encrypt MEK |
| recovery_hash_auth | ✓ | | argon2id(recovery_key_auth, salt_recovery_auth) - for account recovery |
| recovery_hash_data | ✓ | | argon2id(recovery_key_data, salt_recovery_data) - for data recovery |
| stored_password_hash | | ✓ | argon2id(password_hash_auth, salt_server_auth) - stored on server |
| stored_recovery_hash | | ✓ | argon2id(recovery_hash_auth, salt_server_recovery) - stored on server |
| login_hash | temporary | | argon2id(password_hash_auth, salt_server_auth) - sent during login |
| recovery_login_hash | temporary | | argon2id(recovery_hash_auth, salt_server_recovery) - sent during account recovery |
| encrypted_mek_password | ✓ | ✓ | AES-256-GCM(MEK, key: password_hash_data, nonce: nonce_mek_password) |
| encrypted_mek_recovery | ✓ | ✓ | AES-256-GCM(MEK, key: recovery_hash_data, nonce: nonce_mek_recovery) |

- Data (notes) is encrypted with `master_encryption_key` on client side, then sent to the server

- For the login, the server give the `salt_auth` and `salt_server_auth`. The client send `login_hash`. The server compare it with `stored_password_hash` and send back `salt_data`, `encrypted_mek_password` and `encrypted_data`

- For account recovery: the server send `salt_recovery_auth`, `salt_server_recovery`. The client end `recovery_login_hash`. The server compare it with `stored_recovery_hash`.

- For data recovery, the server give `encrypted_mek_recovery` and `salt_recovery_data`. The user can now decrypt data and derive new `stored_password_hash` and new `encrypted_mek_password` and send back to server. (no data recovery without account logged in)


- Data (notes) are encrypted using AES-256-GCM
- Hashes are generated with argon2id.
- Server store only blobs of encrypted files. There will be the less possible cleartext metadata.
- Server handle login on new devices for sync. 
- Cloudflare will handle PQC for online data transmission phase 1.
- The things decrypted on client side should not be kept in memory

More info on the process:
https://claude.ai/public/artifacts/ef1b8b89-2651-47b6-a119-be4242cd76a2