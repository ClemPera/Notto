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

* **Local Database:** SQLite or sled

  * Managed fully by Rust backend
  * Stores encrypted data
  * Supports offline-first behavior
* **Sync Server:** CouchDB (latest)

  * Self-hostable, used for sync only
  * All communication done via Rust
  * TypeScript never interacts directly

### Security & Encryption

* **Handled entirely in Rust**
* **Encryption Libraries:** `ring`, `RustCrypto`, `pqcrypto-kyber`, `argon2`
* **Key Derivation:** Argon2id
* **Encryption:** AES-256-GCM
* **Key Exchange:** ML-KEM-768 (post-quantum)
- **2FA**: TOTP (Time-based One-Time Password) using standard authenticator apps

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
- Encryption key derived from user password (never sent to server)
- Zero-knowledge architecture: server cannot decrypt notes
- All files encrypted at rest on device
- ML-KEM-768 used for secure key exchange between devices

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
* Derive keys using Argon2id
* Handle key exchange via ML-KEM-768
* AES-256-GCM for symmetric encryption or files
* Encrypt data before writing to disk or database
* Never expose keys or decrypted content to frontend

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

## Implementation Priorities (MVP)

**Part 1: Foundation**

* Set up Tauri + React + Rust + Tailwind
* Implement database layer (SQLite/sled)
* Create core Tauri command structure

**Part 2: Core Backend Logic**

* Implement encryption (AES-256-GCM, Argon2id)
* Build CRUD operations in Rust
* Implement search and indexing

**Part 3: Security & Crypto**

* Implement ML-KEM-768, recovery phrase (BIP39)
* At-rest encryption, session management, OS keychain

**Part 4: Sync & Auth**

* Implement CouchDB sync client and TOTP auth
* Manage sessions, tokens, and login flow

**Part 5: Conflict Handling**

* Implement conflict detection/resolution in Rust
* Expose minimal info to frontend for display

**Part 6: Frontend Polish**

* Markdown editor, folder/subfolders UI, search
* Settings and platform integration

**Part 7: Testing & Release**

* Rust unit and integration tests
* E2E tests (user flows, sync, auth)
* Optimization and bug fixes

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

- AES-256-GCM for markdown (and others) encryption
- Use ML-KEM-768 (CRYSTALS-Kyber) for post-quantum secure key exchange
- Generate cryptographically secure random IVs for each encryption operation
- Use authenticated encryption (GCM mode)
- Implement proper key derivation 
- Store salt securely
- Encrypt all files at rest on device
- Version encryption format for future upgrades
- Consider hybrid encryption: ML-KEM-768 for key exchange, AES-256-GCM for content


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