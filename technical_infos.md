# Developer Implementation Guide

## Overview

This document defines the technical implementation for a cross-platform note-taking application. It serves as a roadmap for developers to understand what needs to be built, the technologies to use, and how components should interact.

---

## Technology Stack

### Frontend Layer (Presentation Only)

* **Framework:** React (latest)
* **Language:** TypeScript (latest)
* **Styling:** Tailwind CSS (latest)
* **Build Tool:** Vite (latest)
* **State Management:** React hooks (useState, useContext)
* **Responsibilities:**

  * Render UI and manage user interactions
  * Call Rust backend via Tauri IPC
  * Display returned data and handle UI state only
* **Restrictions:**

  * No business logic, encryption, or data manipulation
  * No direct access to CouchDB or filesystem

### Backend Layer (Core Logic)

* **Framework:** Tauri v2 (latest stable)
* **Language:** Rust (latest stable)
* **Responsibilities:**

  * All business logic and computation
  * Encryption/decryption
  * Database operations (SQLite/sled)
  * File and sync management
  * Authentication, session, and key management
  * Conflict detection and resolution
  * CouchDB communication (sync, offline management)

### Data Persistence & Sync

* **Local Database:** SQLite (Rust backend)

  * Managed fully by Rust backend
  * Stores encrypted note content and metadata
  * Supports offline-first behavior
  * Notes stored as encrypted markdown in database with encryption applied at rest
* **Storage Format:** Notes are markdown files (.md) that are encrypted before storage

  * Markdown content stored encrypted in SQLite
  * Can be exported as encrypted markdown files
  * All markdown editable; encryption/decryption handled transparently in Rust
* **Sync Server:** CouchDB (latest)

  * Self-hostable, used for sync only
  * All communication done via Rust
  * TypeScript never interacts directly
  * Sync implemented in Phase 1 (included in MVP)

### Security & Encryption

* **Handled entirely in Rust**
* **Encryption Libraries:** `ring`, `RustCrypto`, `argon2`
* **Key Derivation:** Argon2id with password-based key derivation
* **Encryption:** AES-256-GCM for all note content
* **Key Exchange:** ML-KEM-768 (post-quantum) - deferred to Phase 2
- **2FA**: TOTP (Time-based One-Time Password) using standard authenticator apps - included in Phase 1

### Environment

* **Notes:** Encrypted Markdown files (.md)


**Infrastructure:**
- Docker & Docker Compose (for running CouchDB locally)


---

## Architecture Overview

```
React (TS UI)
  ↓
Tauri IPC Calls
  ↓
Rust Backend
  ├─ Encryption / Key Management
  ├─ Database (SQLite/sled)
  ├─ File System / CouchDB Sync
  ├─ Authentication & 2FA
  └─ Conflict Handling
```

### Architectural Principles


**Local-First Architecture:**
- All operations work offline by default
- Data stored locally (encrypted)
- Sync is optional and runs in background
- UI never blocks waiting for network

**End-to-End Encryption:**
- Encryption happens client-side before sync and for local storage
- Server (CouchDB) only stores encrypted blobs
- Encryption key derived from user password via Argon2id (never sent to server)
- Zero-knowledge architecture: server cannot decrypt notes
- All notes encrypted at rest on device in SQLite
- Recovery phrase derived from password (BIP39-style) for account recovery
- ML-KEM-768 for post-quantum key exchange deferred to Phase 2

**Authentication & Security:**
- The users should be able to login with username/password
- TOTP 2FA

**Cross-Platform Strategy:**
- Single React codebase for all platforms
- Tauri handles platform-specific compilation
- Conditional logic for platform-specific features using Tauri's platform detection
- Shared Rust backend logic across desktop and mobile

* **Rust is the single source of truth.**
* **TypeScript handles presentation only.**
* **No logic duplication.**
* **Frontend cannot access sensitive or unencrypted data.**
* **All core features (sync, crypto, validation) live in Rust.**

---

## Core Components to Implement

### 1. Rust Backend – Database Layer

* Initialize and manage SQLite/sled database
* CRUD operations exposed via Tauri commands
* Manage metadata, relationships, and indexing
* Implement search and filtering in Rust
* Handle migrations and integrity checks

### 2. Rust Backend – Encryption Layer

* Perform all encryption/decryption
* Derive keys using Argon2id with user password
* Generate recovery phrases (BIP39-style) derived from password
* AES-256-GCM for symmetric encryption of note content
* Encrypt markdown before writing to database at rest
* Never expose keys or decrypted content to frontend
* ML-KEM-768 key exchange deferred to Phase 2

### 3. Rust Backend – Sync Layer

* Implement CouchDB hostable using docker and sync logic in Rust
* Handle bidirectional encrypted sync
* Manage sync state and conflict resolution
* Perform retries and background sync threads
* Frontend displays sync status only

### 4. Rust Backend – Authentication & 2FA

* Handle login, registration, and recovery
* Manage TOTP, session tokens, and keychain storage
* Generate recovery and backup codes
* TypeScript only displays UI; Rust performs all checks

### 5. Rust Backend – Conflict Resolution

* Detect and resolve sync conflicts
* Maintain conflict versions and metadata
* Notify frontend of conflict states for display only

### 6. Rust Backend – File Operations

* Manage encrypted markdown files and attachments
* Import/export features (always through Rust)
* Support format conversion and validation

### 7. Frontend – React UI Layer (Presentation Only)

* Display UI and interact with Rust commands
* Render editor, note lists, folders/subfolders, and search results
* Show sync, authentication, and conflict status

---

## Implementation Priorities (MVP - Phase 1)

**Part 1: Foundation**

* Set up Tauri + React + Rust + Tailwind
* Implement SQLite database layer in Rust
* Create core Tauri command structure and IPC

**Part 2: Core Backend Logic**

* Implement encryption (AES-256-GCM, Argon2id with password-based derivation)
* Build CRUD operations for notes in Rust
* Implement search and indexing in Rust

**Part 3: Authentication & Encryption**

* Implement password-based encryption key derivation (Argon2id)
* Generate and store recovery phrases (BIP39-style)
* Session management and OS keychain integration
* Login, registration, and recovery flows in Rust

**Part 4: Sync & CouchDB**

* Implement CouchDB sync client in Rust (bidirectional)
* Implement conflict detection and resolution
* TOTP 2FA support
* Background sync thread management

**Part 5: Frontend UI**

* Markdown editor with live preview (React)
* Note list and folder/subfolder organization
* Authentication UI (login, registration, 2FA)
* Search interface
* Sync status display
* Conflict resolution UI (display only)

**Part 6: File Operations & Polish**

* Encrypted file export/import
* Settings and preferences
* Platform integration (tray, shortcuts)

**Part 7: Testing & Release**

* Rust unit and integration tests
* E2E tests (user flows, sync, auth, conflicts)
* Performance optimization and bug fixes

## Phase 2 (Deferred)

* ML-KEM-768 post-quantum key exchange implementation
* Multiple vault support
* Advanced conflict resolution UI
* Mobile optimization

---

## Testing Strategy

### Rust Unit Tests

* Core functions (encryption, DB, auth, conflict)

### Rust Integration Tests

* Full data flow: create/edit/sync notes
* Authentication and 2FA

### Frontend Tests

* UI rendering, Tauri command invocation

### E2E Tests

* User registration → sync → recovery flow

---

## Security Considerations

1. **Never store plaintext encryption keys or unencrypted content**
   - Always derive keys from password on-demand
   - Clear keys from memory when app locks
   - All files encrypted at rest on device
   - No plaintext metadata in local storage

2. **Never send unencrypted content to server**
   - All encryption happens client-side
   - CouchDB only receives encrypted blobs

3. **Validate all user input**
   - Validate all Tauri inputs
   - Implement sanitization 
   - Validate folder names and paths
   - Rate-limit authentication attempts

4. **Secure credential storage**
   - Use OS keychain/keystore for auth tokens
   - Never log sensitive data
   - Clear sensitive data on logout

5. **Session management**
   - Optional auto-lock after inactivity or on app restart
   - Invalidate tokens on password change
   
* All sensitive operations in Rust

### Encryption Best Practices

- AES-256-GCM for all note content encryption
- Password-based key derivation using Argon2id (Phase 1)
- Generate cryptographically secure random IVs/nonces for each encryption operation
- Use authenticated encryption (GCM mode with authentication tags)
- Store Argon2id parameters (salt, cost, iterations) securely alongside encrypted data
- Encrypt all notes at rest in SQLite database
- Version encryption format for future upgrades (support multiple versions)
- BIP39-style recovery phrase generation for account recovery
- ML-KEM-768 hybrid encryption deferred to Phase 2 (for device-specific keys)


## Platform-Specific Considerations

**Features to Implement:**
- System tray integration (show sync status, quick actions)
- Global keyboard shortcuts (quick note creation)
- Native menus (File, Edit, View, etc.)
- Auto-updater (Tauri's built-in updater)


---

## Performance Targets

* App startup: < 2s
* Note opening: < 50ms
* Search: < 200ms (10k notes)
* Sync: < 5s (100 notes)
* Encryption overhead: < 10ms per note

---

## Development Best Practices

1. **Keep frontend dumb:** No logic beyond UI.
2. **Implement and test all logic in Rust.**
3. **Use Result<T, Error> and proper error handling.**
4. **Add structured logs with `tracing`.**
5. **Add enough comments (but not too much)**
6. **Implements enough tests**

7. **Handle edge cases**
   - Empty states (no notes, no folders)
   - Very long note titles or content
   - Special characters in filenames
   - Concurrent edits (conflict scenarios)
8. Keep the code maintainable for humans

---

## Success Criteria

✅ Users can create, edit, delete notes offline  
✅ Markdown rendering works correctly  
✅ Organization of notes works correctly  
✅ Search returns accurate results quickly  
✅ E2E encryption is implemented and tested  
✅ Sync works bidirectionally between devices  
✅ Conflicts are detected and handled  
✅ 2FA authentication is functional  
✅ App works on all target platforms (Windows, macOS, Linux, iOS, Android)  
✅ Recovery phrase generation and restoration works  
✅ No critical bugs or data loss scenarios  