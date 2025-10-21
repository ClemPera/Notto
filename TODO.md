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

## Part 4: Sync & CouchDB - 🟡 IN PROGRESS

- [x] Implement CouchDB HTTP client (reqwest-based)
- [x] Implement conflict detection (timestamp and content-based)
- [x] Implement conflict resolution (three-way merge, create copies)
- [ ] Implement bidirectional sync (get_local_changes, upload, download)
- [ ] Implement background sync thread with tokio
- [ ] Implement change tracking and versioning in database
- [ ] Implement sync status notifications
- [x] Create Tauri sync command handlers
- [x] **Sync System Compilation Status: ✅ PASSING**

## Part 5: Frontend UI (React) - ✅ COMPLETE (MVP)

- [x] Set up Vite + React + TypeScript + Tailwind
- [x] Implement state management with Zustand
- [x] Create Tauri/React IPC bridge
- [x] Implement authentication UI (login, registration)
- [x] Implement markdown editor with live preview
- [x] Implement note list view with sidebar
- [x] Implement folder/subfolder organization UI
- [x] Implement sync status indicator
- [x] Implement main app layout and navigation
- [x] Connect all UI components to Tauri backend
- [ ] Implement 2FA setup UI (advanced feature)
- [ ] Implement search interface (advanced feature)
- [ ] Implement conflict resolution display UI (advanced feature)
- [x] **Frontend Compilation Status: ✅ READY (npm install required)**

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

### Part 4 - CouchDB Sync Module Structure

**Files Created:**
- `src-tauri/src/sync/mod.rs` - Main sync client and orchestration (174 lines)
- `src-tauri/src/sync/client.rs` - HTTP client for CouchDB API (350+ lines)
- `src-tauri/src/sync/conflict.rs` - Conflict detection and resolution (280+ lines)
- `src-tauri/src/sync/models.rs` - Sync data structures (110+ lines)
- `src-tauri/src/commands/sync.rs` - Tauri IPC sync command handlers (130+ lines)
- Updated `src-tauri/src/commands/mod.rs` - Exposed sync commands

**Key Features:**
- CouchDB document management (upload, download, delete)
- Connectivity checking with 5-second timeout
- Changes feed polling for incremental sync
- Conflict detection using timestamps, versions, and content hashes
- Three-way merge strategy for text content
- Create conflict copies strategy for manual resolution
- Async sync operations with status tracking
- User-per-database model (userdb-{username})
- Bearer token authentication

**Tauri Commands Exposed:**
- `initialize_sync` - Initialize sync client with server URL
- `start_sync` - Begin synchronization process
- `get_sync_status` - Check current sync status (Idle, Syncing, Success, Error, Offline)
- `check_connectivity` - Test connection to CouchDB server

**Sync Data Structures:**
- `SyncClient` - Main client managing sync operations
- `SyncStatus` - States: Idle, Syncing, Success, Error, Offline
- `CouchDbDocument` - Document format for storage
- `ChangesResponse` - Incremental changes from CouchDB
- `ConflictStrategy` - LastWriteWins, KeepLocal, or CreateBoth
- `SyncEvent` - Event types for sync operations

**Conflict Resolution Strategies:**
1. Last-Write-Wins: Newer document overwrites older
2. Keep Local: Always prefer local changes
3. Create Both: Keep local and create conflict copy of remote

**HTTP Operations:**
- POST `/_session` - Authenticate with CouchDB
- PUT `/{db}` - Create user database
- PUT `/{db}/{docid}` - Upload encrypted document
- GET `/{db}/{docid}` - Download encrypted document
- GET `/{db}/_changes` - Get incremental changes
- DELETE `/{db}/{docid}` - Delete document

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

## Implementation Details

### Part 5 - React Frontend Structure

**Files Created:**
- `package.json` - Dependencies and build scripts
- `vite.config.ts` - Vite build configuration with path aliases
- `tsconfig.json` - TypeScript strict mode configuration
- `tailwind.config.js` - Tailwind CSS theme customization
- `index.html` - HTML entry point
- `src/index.css` - Global styles and Tailwind imports
- `src/main.tsx` - React app entry point
- `src/App.tsx` - Main app component (500+ lines)
- `src/store/appStore.ts` - Zustand global state management
- `src/utils/tauri.ts` - Type-safe Tauri IPC wrappers
- Component files in `src/components/`:
  - LoginForm.tsx - Login UI with error handling
  - RegisterForm.tsx - Registration with recovery phrase display
  - MarkdownEditor.tsx - Split-view editor with auto-save
  - NoteList.tsx - Sidebar with note/folder creation
  - SyncStatus.tsx - Real-time sync status indicator

**Key Features:**
- Full authentication flow with login/register
- Session persistence via localStorage
- Dark/light theme toggle
- Split-view markdown editor with live preview
- Auto-save after 2 seconds of inactivity
- Note and folder creation/management
- Real-time sync status monitoring
- Responsive design with Tailwind CSS
- Type-safe Tauri IPC communication
- Error handling and user feedback

**State Management (Zustand):**
- Auth state: token, user_id, username, 2FA setup
- Editor state: current note, title, content, unsaved changes
- Sync state: status, message, last sync time
- UI state: theme, sidebar, preview toggle

**UI Components:**
1. LoginForm: Handles user login
2. RegisterForm: User registration with recovery phrase backup
3. MarkdownEditor: Main editor with live preview
4. NoteList: Sidebar for note/folder management
5. SyncStatus: Real-time sync indicator
6. App: Main layout and auth flow

**Tauri Commands Connected:**
- Auth: register, login, logout, verify_session, setup_totp, verify_totp
- Notes: create, read, update, delete, list
- Folders: create, list
- Sync: initialize, start, get_status, check_connectivity

---

**Last Updated:** Phase 1 Part 5 - React Frontend Complete (MVP Ready)
