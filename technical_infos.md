## Phase 1
### Security
master_encryption_key = RANDOM1 (client side only)  
recovery_key_raw_auth = Random 25 words (client side only)  
recovery_key_raw_data = Random 25 different words (client side only)  

salt_auth = RANDOM2  
salt_data = RANDOM3   
salt_recovery_auth = RANDOM4  
salt_recovery_data = RANDOM5  
salt_server_auth = RANDOM6
salt_server_recovery = RANDOM7 

password_hash_auth = argon2id(password, salt_auth)  
recovery_hash_auth = argon2id(recovery_key_raw_auth, salt_recovery_auth)  
stored_password_hash||login_hash = argon2id(password_hash_auth, salt_server_auth)
stored_recovery_hash = argon2id(password_hash_auth, salt_server_recovery)

password_hash_data = argon2id(password, salt_data)  
recovery_hash_data = argon2id(recovery_key_raw_data, salt_recovery_data) (client side only)  

encrypted_mek_password = AES-256-GCM(master_encryption_key, password_hash_data)  
encrypted_mek_recovery = AES-256-GCM(master_encryption_key, recovery_hash_data)  

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