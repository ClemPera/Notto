## Phase 1
### Security
master_encryption_key = RANDOM1 (client side only)  
recovery_key_raw_auth = Random 25 words (client side only)  
recovery_key_raw_data = Random 25 different words (client side only)  

salt_auth = RANDOM2  
salt_data = RANDOM3   
salt_recovery_auth = RANDOM4  
salt_recovery_data = RANDOM5  

password_hash_auth = argon2id(password, salt_auth)  
recovery_hash_auth = argon2id(recovery_key_raw_auth, salt_recovery_auth)  

password_hash_data = argon2id(password, salt_data)  
recovery_hash_data = argon2id(recovery_key_raw_data, salt_recovery_data) (client side only)  

encrypted_mek_password_data = AES-256-GCM(master_encryption_key, password_hash_data)  
encrypted_mek_recovery_data = AES-256-GCM(master_encryption_key, recovery_hash_data)  

- Data (notes) is encrypted with `master_encryption_key` on client side, then sent to the server

- For login, the server give the `salt_auth`, then the user provide the `auth_hash`, then the server compare it and send back the `salt_data` and `encrypted_mek_password` and encrypted data blobs.

- For data recovery, the server give `encrypted_mek_recovery_data` and `salt_recovery_data`. The user can now decrypt data and derive new key and send back to server. (no data recovery without account logged in)
- For account recovery, the server give `salt_recovery_auth` and the user send derived: `recovery_hash_auth`

- Data (notes) are encrypted using AES-256-GCM
- Hashes are generated with argon2id.
- Server store only blobs of encrypted files. There will be the less possible cleartext metadata.
- Server handle login on new devices for sync. 
- Cloudflare will handle PQC for online data transmission phase 1.
- The things decrypted on client side should not be kept in memory

More info on the process (not totally complete but more visual):
- https://claude.ai/public/artifacts/d4ae536d-6e9c-4ebb-8e73-28fdbe2022dd