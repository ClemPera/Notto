## Phase 1
### Security
salt_auth = RANDOM  
salt_data = RANDOM2  
auth_hash = Argon2id(password, salt_auth)  
data_key = Argon2id(password, salt_data) (client side only)

- Data is stored as AES-256 (GCM?) encrypted with data_key on client side, then sent to the server.
- For login, the user provide the auth_hash, then is compared on the server, then the server send back the salt_data with encrypted data blobs.
- Hashes are generated with argon2id.
- Server store only encrypted blobs of files. There will be the less possible cleartext metadata.
- Server handle login on new devices for sync. 
- Cloudflare will handle PQC for online data transmission phase 1.

More info on the login/register process:
- https://claude.ai/public/artifacts/cf860aef-7800-426d-bf4f-40e234e519a7
- https://claude.ai/public/artifacts/0e9a070f-72c5-4c12-a0f3-26bf119cef0a