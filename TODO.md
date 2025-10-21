# Notto Phase 1 Implementation Progress

## Part 1: Foundation - ✅ COMPLETE

- [x] Initialize Tauri v2 project with React + TypeScript
- [x] Set up project directory structure
- [x] Configure Cargo.toml with all dependencies
- [x] **Compilation Status: ✅ PASSING**

## Part 2: Core Backend Logic - 🟡 IN PROGRESS

### Database Layer
- [x] Create SQLite schema with tables for: users, sessions, notes, folders, sync_metadata, encryption_params, recovery_phrases, totp_secrets
- [x] Implement FTS5 full-text search on notes
- [x] Create database CRUD operations
- [x] Create indexes for query optimization

### Encryption & Security
- [x] Implement Argon2id password-based key derivation
- [x] Implement AES-256-GCM encryption/decryption
- [x] Implement BIP39-style recovery phrase generation
- [x] Create encryption utilities with proper nonce/IV handling

### Tauri Commands
- [x] Create command handlers for note operations (create, read, update, delete, list)
- [ ] Create command handlers for folder operations
- [ ] Create command handlers for folder operations
- [ ] Add session/authentication to commands

## Part 3: Authentication & Encryption - 🟡 IN PROGRESS

- [x] Implement registration (username/password) with Argon2id hashing
- [x] Implement login with Argon2id verification
- [x] Implement session token management (UUID-based, 24-hour expiry)
- [ ] Implement OS keychain integration for token storage
- [x] Implement TOTP 2FA setup and verification (with backup codes)
- [x] Implement backup codes generation (10 codes)
- [ ] Implement recovery phrase restoration flow
- [ ] Implement password change with key re-derivation
- [x] Create Tauri command handlers for all auth operations
- [x] **Auth System Compilation Status: ✅ PASSING**

## Part 4: Sync & CouchDB - ⏳ PENDING

- [ ] Implement CouchDB HTTP client
- [ ] Implement bidirectional sync
- [ ] Implement conflict detection (timestamp-based)
- [ ] Implement conflict resolution (create conflict copies)
- [ ] Implement background sync thread
- [ ] Implement change tracking and versioning
- [ ] Implement sync status notifications

## Part 5: Frontend UI (React) - ⏳ PENDING

- [ ] Create basic React component structure
- [ ] Implement authentication UI (login, registration, 2FA)
- [ ] Implement markdown editor with live preview
- [ ] Implement note list view
- [ ] Implement folder/subfolder organization UI
- [ ] Implement search interface
- [ ] Implement sync status indicator
- [ ] Implement conflict resolution display UI

## Part 6: Testing - ⏳ PENDING

- [ ] Write unit tests for encryption module
- [ ] Write unit tests for database operations
- [ ] Write unit tests for auth logic
- [ ] Write integration tests for full workflows
- [ ] Write E2E tests for user flows

## Part 7: Polish & Platform Features - ⏳ PENDING

- [ ] Configure platform-specific builds (Windows, macOS, Linux)
- [ ] Implement system tray integration
- [ ] Implement global keyboard shortcuts
- [ ] Optimize performance (startup time, search, sync)
- [ ] Error handling and edge cases

---

## Code Structure

```
/home/clement/Documents/Rust/Notto/
├── Cargo.toml                      # Project manifest with dependencies
├── README.md
├── LICENSE
├── technical_infos.md              # Technical specifications
├── project_charter.md              # Project overview
├── CLAUDE.md                        # Development guidelines
└── src-tauri/
    └── src/
        ├── main.rs                 # Tauri app entry point
        ├── models.rs               # Data models
        ├── db/
        │   ├── mod.rs             # Database initialization
        │   ├── schema.rs           # SQLite schema creation
        │   └── operations.rs       # CRUD operations
        ├── crypto/
        │   ├── mod.rs
        │   ├── encryption.rs       # AES-256-GCM encryption
        │   ├── key_derivation.rs   # Argon2id derivation
        │   └── recovery_phrase.rs  # BIP39 recovery phrases
        ├── sync/
        │   └── mod.rs             # CouchDB sync (placeholder)
        ├── auth/
        │   └── mod.rs             # Authentication (placeholder)
        └── commands/
            ├── mod.rs
            ├── notes.rs            # Note IPC commands
            ├── auth.rs             # Auth IPC commands (placeholder)
            └── folders.rs          # Folder IPC commands (placeholder)
└── src-frontend/                   # React frontend (to be created)
```

## Key Decisions Made

1. **Database:** SQLite with Rust backend (not PouchDB)
2. **Encryption:** Password-derived keys (Argon2id) for Phase 1
3. **Recovery:** BIP39-style recovery phrases from password
4. **Sync:** Included in Phase 1 MVP
5. **ML-KEM-768:** Deferred to Phase 2
6. **2FA:** TOTP included in Phase 1
7. **Frontend:** React with TypeScript (confirmed)

## Next Steps

1. Implement remaining Tauri command handlers
2. Create authentication system (register/login)
3. Implement TOTP 2FA
4. Create React frontend structure
5. Implement CouchDB sync client
6. Build React UI components

## Implementation Details

### Part 3 - Authentication Module Structure

**Files Created:**
- `src-tauri/src/auth/mod.rs` - Main auth logic (register, login, verify, logout)
- `src-tauri/src/auth/password.rs` - Argon2id password hashing and verification
- `src-tauri/src/auth/session.rs` - UUID-based session token generation
- `src-tauri/src/auth/totp.rs` - TOTP 2FA setup, verification, and backup codes
- `src-tauri/src/commands/auth.rs` - Tauri IPC command handlers

**Key Features:**
- Password validation (minimum 8 characters)
- Argon2id hashing with per-user salt
- Session tokens with 24-hour expiry
- TOTP secret generation (base32-encoded)
- 10 backup codes per user
- QR code URI generation for authenticator apps
- Comprehensive error handling with custom error types

**Tauri Commands Exposed:**
- `register` - Register new user (returns user_id and recovery_phrase)
- `login` - Login with credentials (returns session token)
- `setup_totp` - Generate TOTP secret and backup codes
- `verify_totp_setup` - Verify TOTP code during setup
- `verify_session_token` - Verify a session token
- `logout` - Logout user

---

**Last Updated:** Phase 1 Part 3 - Authentication Complete (Part 2 & 3)
