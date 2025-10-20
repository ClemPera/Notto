# Developer Implementation Guide

## Overview

This document outlines the technical implementation requirements for building a cross-platform note-taking application. It serves as a roadmap for developers to understand what needs to be built, the technologies to use, and how components should interact.

---

## Technology Stack

### Frontend Layer
- **UI Framework**: React (latest)
- **Language**: TypeScript (latest)
- **Styling**: Tailwind CSS (latest)
- **Build Tool**: Vite (latest)
- **State Management**: React hooks (useState, useContext, useReducer)

### Application Shell
- **Framework**: Tauri v2 (latest stable)
- **Backend Language**: Rust (latest stable)
- **Purpose**: Provides native application wrapper, system API access, and Rust-based backend logic

### Data Persistence & Sync
- **Local Database**: PouchDB (latest)
  - Runs in the application's webview
  - Stores encrypted note data
  - Handles offline-first storage
- **Sync Server**: CouchDB (latest v3.x)
  - Self-hostable by users
  - Official instance hosted by project
  - Communicates via CouchDB replication protocol

### Security & Encryption
- **Encryption API**: Web Crypto API (browser native) + ML-KEM-768 library for key exchange
- **Key Exchange**: ML-KEM-768 (post-quantum secure lattice-based cryptography)
- **Content Encryption**: AES-256-GCM for symmetric encryption of note content
- **Key Derivation**: PBKDF2 or Argon2id for password-based key generation
- **At-Rest Encryption**: All files encrypted on local device storage
- **2FA**: TOTP (Time-based One-Time Password) using standard authenticator apps

### Storage Format
- **Note Files**: Markdown (.md) format
- **Metadata**: JSON (embedded as frontmatter or separate sidecar files)
- **Structure**: Hierarchical folder/subfolder organization on local filesystem

---

## Development Environment Setup

### Required Tools

**Core Development:**
- Node.js (latest LTS)
- npm or yarn (latest)
- Rust toolchain (rustc + cargo, latest stable)
- Tauri CLI (`cargo install tauri-cli`)

**Platform-Specific (for building mobile apps):**
- **iOS builds**: macOS with Xcode (latest), CocoaPods
- **Android builds**: Android Studio (latest), Android SDK, NDK

**Infrastructure:**
- Docker & Docker Compose (for running CouchDB locally)

### Initial Setup Commands

```bash
# Install dependencies
npm install

# Install Rust dependencies
cd src-tauri && cargo build

# Start local development environment
docker-compose up -d

# Run desktop app in development mode
npm run tauri dev
```

### Development Services (Docker)

Create a `docker-compose.yml` that provides:
- **CouchDB** on port 5984
  - Admin UI accessible at http://localhost:5984/_utils
  - Default admin credentials for development
  - Pre-configured with CORS enabled for local development
  - Database auto-creation enabled

---

## Architecture Overview

### Application Flow

```
User Interaction
    ↓
React UI (TypeScript + Tailwind)
    ↓
PouchDB (local storage + encryption)
    ↓
Tauri Bridge (Rust backend)
    ↓
Native OS APIs / CouchDB Sync (with 2FA session auth)
```

### Key Architectural Decisions

**Local-First Architecture:**
- All operations work offline by default
- Data stored locally using PouchDB
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
- CouchDB native authentication with username/password
- TOTP 2FA built into CouchDB (no external services needed)
- Session cookie-based authentication for sync
- Self-hostable with full 2FA support
- Admin creates users via CouchDB API with TOTP secrets

**Cross-Platform Strategy:**
- Single React codebase for all platforms
- Tauri handles platform-specific compilation
- Conditional logic for platform-specific features using Tauri's platform detection
- Shared Rust backend logic across desktop and mobile

---

## Core Components to Implement

### 1. Database Layer

**Goal**: Provide abstraction over PouchDB for all note operations.

**Requirements:**
- Initialize PouchDB database on app startup
- Implement CRUD operations (Create, Read, Update, Delete) for notes
- Handle note metadata (title, folder, tags, timestamps)
- Implement search functionality across title and content
- Support querying by folder/tag
- Handle database migrations for schema changes

**Data Schema:**
```
Note Document:
- _id: unique identifier (e.g., "note-{timestamp}")
- _rev: PouchDB revision (managed automatically)
- title: string
- content: string (encrypted)
- folder: string (path-like: "Work/Projects/ProjectA")
- tags: array of strings
- createdAt: timestamp
- modifiedAt: timestamp
- encrypted: boolean flag
```

### 2. Encryption Layer

**Goal**: Implement client-side end-to-end encryption for all note content with post-quantum secure key exchange.

**Requirements:**
- Use Web Crypto API for AES-256-GCM symmetric encryption
- Implement ML-KEM-768 (CRYSTALS-Kyber) for post-quantum secure key exchange between devices
- Use a JavaScript/WebAssembly library for ML-KEM-768 (e.g., liboqs-js or similar)
- Encrypt all note content with AES-256-GCM before storing locally or syncing
- Derive encryption key from user password using PBKDF2 (100,000+ iterations) or Argon2id
- Generate random initialization vectors (IV) for each encryption operation
- Store encrypted content, IV, and key exchange public keys in note/settings documents
- Decrypt content on-demand when note is accessed
- Never store plain-text encryption key (derive on-demand from password)
- All files must be encrypted at rest on device (no plaintext storage)

**Key Exchange Process:**
```
Device A Setup:
- Generate ML-KEM-768 keypair (public/private)
- Store public key in user profile (encrypted, uploaded to server)
- Store private key locally (encrypted with user's derived key)

Device B Syncing:
- Retrieve Device A's public key from server
- Generate shared secret using ML-KEM-768 encapsulation
- Derive session key from shared secret
- Use session key to encrypt/decrypt sync-specific data
```

**Key Derivation Process:**
```
User Password
    ↓
+ Random Salt (generated once, stored encrypted in settings)
    ↓
PBKDF2 / Argon2id (100k+ iterations)
    ↓
256-bit Master Key (held in memory, never persisted)
    ↓
Derives: Content Encryption Key, Local Storage Key, Key Exchange Private Key Encryption
```

**Local Storage Encryption:**
- All note files encrypted with AES-256-GCM before writing to disk
- PouchDB documents stored encrypted
- Metadata (titles, folders) should also be encrypted
- Decryption happens in memory only when accessing notes

**Recovery Phrase:**
- Generate 24-word BIP39 mnemonic phrase during account setup
- Derive master key from mnemonic
- User must write down phrase (displayed once)
- Implement recovery flow using phrase to regenerate encryption key and ML-KEM keypair

### 3. Sync Layer

**Goal**: Synchronize encrypted notes across devices using CouchDB replication protocol.

**Requirements:**
- Implement bidirectional sync between PouchDB (local) and CouchDB (remote)
- Use PouchDB's built-in sync API (`db.sync()`)
- Configure continuous sync (live: true, retry: true)
- Handle sync authentication using user credentials
- Implement sync status indicators (syncing, paused, error, up-to-date)
- Only sync when user is authenticated and has configured sync
- Encrypt all note content before sending to server
- Handle network interruptions gracefully (automatic retry)

**Sync Configuration:**
```
PouchDB Local ←→ CouchDB Remote
- Authentication: username + password
- Sync Mode: Continuous (live updates)
- Conflict Strategy: Detect and create conflict copies
- Encryption: Client-side before upload
```

### 4. Conflict Resolution

**Goal**: Handle situations where the same note is edited on multiple devices while offline.

**Requirements:**
- Detect conflicts using PouchDB's conflict detection (automatically provided)
- Retrieve all conflicting versions of a note using `db.get(id, { conflicts: true })`
- For MVP: Create conflict copy notes with timestamp suffix
  - Example: "Original Note" → "Original Note (conflict 2024-10-18)"
- Show notification to user when conflicts are detected
- Mark conflict copies with special metadata flag
- Allow user to manually merge or delete conflict copies

**Phase 2 Enhancement:**
- Build visual conflict resolution UI
- Show side-by-side diff of conflicting versions
- Allow user to choose version or manually merge
- Implement three-way merge when possible

### 5. Authentication & 2FA

**Goal**: Secure user accounts using CouchDB's native authentication with TOTP 2FA.

**Requirements:**

**User Registration Flow:**
- Generate TOTP secret (base32 encoded, 32 characters) client-side
- Generate 24-word BIP39 recovery phrase for encryption key recovery
- Generate ML-KEM-768 keypair for key exchange
- Derive encryption key from password (never sent to server)
- Encrypt private key with derived encryption key
- Admin creates user in CouchDB `_users` database with:
  - Username and password (CouchDB hashes with PBKDF2)
  - TOTP secret in user document: `{ "totp": { "key": "BASE32SECRET" } }`
  - Encrypted ML-KEM-768 public key in user profile
  - User's salt for key derivation
- Display TOTP QR code for authenticator app enrollment
- Generate and display 10 single-use backup codes for 2FA recovery
- User must confirm they've saved:
  - Recovery phrase (for encryption key recovery)
  - 2FA backup codes (for 2FA recovery)
  - TOTP secret (in authenticator app)

**User Login Flow:**
- User enters username, password, and TOTP code (6 digits from authenticator app)
- Client derives encryption key from password (client-side, never sent)
- Client sends authentication request to CouchDB `/_session` endpoint:
  ```json
  {
    "name": "username",
    "password": "password",
    "token": "123456"
  }
  ```
- CouchDB validates password and TOTP code
- CouchDB returns session cookie on success
- Client retrieves user profile from CouchDB (contains encrypted keys, salt)
- Client decrypts ML-KEM-768 private key using derived encryption key
- Initialize PouchDB with user-specific database using session cookie

**Session Management:**
- Session cookies automatically handled by PouchDB/browser
- Store session state in Tauri secure storage (OS keychain)
- Never store password or encryption key (only session cookie)
- Implement auto-lock after inactivity (configurable: 5/10/15/30 min)
- Require full re-authentication (password + 2FA) on app restart
- Clear encryption keys from memory on lock/logout

**2FA Backup Code Recovery:**
- If user loses access to authenticator app
- Login with username + password + backup code (instead of TOTP)
- Backup code is single-use and removed after successful login
- User must re-enroll 2FA (new TOTP secret) after using backup code

**CouchDB User Document Structure:**
```json
{
  "_id": "org.couchdb.user:username",
  "name": "username",
  "password": "hashed_by_couchdb",
  "type": "user",
  "roles": [],
  "totp": {
    "key": "BASE32_TOTP_SECRET"
  },
  "mlkem_public_key": "base64_encoded_public_key",
  "salt": "hex_encoded_salt",
  "backup_codes": ["code1", "code2", ...] // Optional: store hashed
}
```

**Tauri Backend Commands:**
- `secure_store(key, value)`: Store session cookie in OS keychain
- `secure_retrieve(key)`: Retrieve session cookie from keychain
- `secure_delete(key)`: Clear session data on logout
- `generate_totp_secret()`: Generate cryptographically secure base32 secret
- `generate_backup_codes()`: Generate 10 random backup codes

**Important Notes:**
- CouchDB handles password hashing (PBKDF2) automatically
- TOTP validation happens on CouchDB server
- Encryption key derivation is entirely client-side
- Server never sees or can derive encryption keys
- 2FA is required for login but not for sync (session cookie is used)
- Users can self-host with full 2FA support (no external services needed)

### 6. Markdown Editor

**Goal**: Provide rich markdown editing experience with live preview and checkbox support.

**Requirements:**
- Implement markdown editor component using a library like:
  - React-Markdown (for rendering)
  - CodeMirror 6 or Monaco Editor (for editing with syntax highlighting)
- Support CommonMark standard
- Implement checkbox/task list syntax:
  ```
  - [ ] Unchecked task
  - [x] Completed task
  ```
- Auto-save changes after 2-3 seconds of inactivity
- Track cursor position and restore on note reopen
- Implement undo/redo functionality
- Show character/word count

**MVP Features:**
- Headers (h1-h6)
- Bold, italic, strikethrough
- Lists (ordered, unordered, checkboxes)
- Code blocks with syntax highlighting
- Links
- Block quotes

**Phase 2 Enhancements:**
- Image embeds and attachments (store encrypted)
- Tables
- Mermaid diagrams
- LaTeX/math equations
- File attachments (PDFs, documents, audio)

### 7. File Organization

**Goal**: Organize notes in hierarchical folder structure.

**Requirements:**
- Implement folder tree UI in sidebar
- Allow creating/renaming/deleting folders
- Support nested folders (unlimited depth)
- Store folder path as string in note metadata (e.g., "Work/Projects/ProjectA")
- Implement drag-and-drop to move notes between folders
- Show note count per folder
- Implement folder collapse/expand state (persist in local settings)
- Support root-level notes (no folder)

**Folder Operations:**
- Create folder: Add to folder list in settings
- Rename folder: Update folder path in all affected notes
- Delete folder: Prompt for action (delete notes or move to root)
- Move note: Update folder field in note document

### 8. Search Functionality

**Goal**: Fast full-text search across all notes.

**Requirements:**
- Implement search across note titles and content
- Display results as-you-type (debounced)
- Highlight matching terms in results
- Show note preview with context around match
- Support case-insensitive search
- Search decrypted content (decrypt in memory for search)
- Filter results by folder or tag (optional)

**Phase 2 Enhancements:**
- Advanced search syntax (AND, OR, NOT)
- Search by date range
- Search by modification date
- Fuzzy search
- Search history

### 9. Settings & Configuration

**Goal**: Allow users to configure app behavior and sync settings.

**Requirements:**
- Store settings in PouchDB or local JSON file
- Settings UI with categories:
  - **Account**: View username, change password, 2FA settings
  - **Sync**: Configure sync server URL, enable/disable sync, sync status
  - **Appearance**: Theme (light/dark), font size, editor preferences
  - **Security**: Auto-lock timeout, require 2FA for sync changes
  - **About**: App version, license, credits

**Critical Settings:**
- Sync server URL (default: official instance, allow custom)
- Auto-lock timer (5/10/15/30 minutes, or never)
- Theme preference
- Default note folder
- Editor preferences (show line numbers, word wrap, etc.)

### 10. Tauri Backend (Rust)

**Goal**: Implement native functionalities.

**Requirements:**
- Implement Tauri commands for:
  - **Filesystem operations**: Read/write encrypted markdown files, handle encrypted image/file attachments
  - **Secure storage**: Store auth tokens and encrypted keys in OS keychain
  - **System integration**: File dialogs (import/export), system notifications
  - **Crypto helpers**: Optional Rust-based crypto for performance-critical operations, ML-KEM-768 operations if needed
  
**Platform Detection:**
- Use Tauri's platform API to detect OS
- Conditionally enable features (e.g., system tray on desktop only)
- Handle mobile-specific APIs (biometric auth, share extensions)

---

## Implementation Priorities (MVP Phase 1)

### Week 1-2: Foundation
1. Set up project structure (Tauri + React + TypeScript + Tailwind)
2. Configure Docker Compose for CouchDB
3. Implement PouchDB database layer with basic CRUD
4. Build basic UI shell (sidebar, editor area, note list)

### Week 3-4: Core Features
5. Implement markdown editor with checkbox support
6. Add folder organization and navigation
7. Implement search functionality
8. Add note creation/editing/deletion flows

### Week 5-6: Security
9. Implement encryption layer (Web Crypto API for AES-256-GCM)
10. Implement ML-KEM-768 for key exchange (integrate library)
11. Add key derivation from password
12. Implement 24-word recovery phrase generation
13. Encrypt all note content before storage (local and sync)
14. Implement at-rest encryption for all local files

### Week 7-8: Sync & Authentication
14. Implement user registration with CouchDB (admin API calls)
15. Implement TOTP secret generation and QR code display
16. Implement login flow with CouchDB session authentication (password + TOTP)
17. Implement PouchDB ↔ CouchDB sync using session cookies
18. Add sync status UI
19. Implement 2FA backup codes generation and recovery flow

### Week 9-10: Conflict Handling
17. Implement conflict detection
18. Create conflict copy mechanism
19. Add conflict notification UI
20. Test offline/online scenarios

### Week 11-12: Mobile & Polish
21. Build and test on iOS and Android
22. Add platform-specific features (biometric auth)
23. Implement auto-save and session management
24. Test cross-platform sync thoroughly

### Week 13-14: Testing & Release
25. Write comprehensive test suite
26. Fix critical bugs
27. Performance optimization
28. Prepare for initial release

---

## Testing Requirements

### Unit Tests
- Database operations (CRUD, search, queries)
- Encryption/decryption functions
- Key derivation
- Conflict detection logic
- Markdown parsing and rendering

### Integration Tests
- End-to-end sync flow (local ↔ remote)
- Authentication and 2FA flows
- Note creation and editing with encryption
- Folder operations
- Search across encrypted notes

### E2E Tests
- User registration and login
- Creating, editing, deleting notes
- Sync between two devices (simulate)
- Conflict resolution scenarios
- Offline → online transition
- App restart with session recovery

### Platform-Specific Tests
- Test on Windows, macOS, Linux
- Test on iOS and Android
- Test different screen sizes and orientations
- Test OS-specific features (notifications, file dialogs)

---

## Performance Targets

- **App startup**: < 2 seconds (cold start)
- **Note opening**: < 50ms (decryption + render)
- **Search**: < 200ms for 10,000 notes
- **Sync**: < 5 seconds for 100 notes on good connection
- **Auto-save**: 2-3 second debounce, < 10ms save operation

---

## Security Considerations

### Critical Security Rules

1. **Never store plaintext encryption keys or unencrypted content**
   - Always derive keys from password on-demand
   - Clear keys from memory when app locks
   - All files encrypted at rest on device
   - No plaintext metadata in local storage

2. **Never send unencrypted content to server**
   - All encryption happens client-side
   - CouchDB only receives encrypted blobs

3. **Validate all user input**
   - Sanitize markdown content to prevent XSS
   - Validate folder names and paths
   - Rate-limit authentication attempts

4. **Secure credential storage**
   - Use OS keychain/keystore for auth tokens
   - Never log sensitive data
   - Clear sensitive data on logout

5. **Session management**
   - Auto-lock after inactivity
   - Require full authentication on app restart
   - Invalidate tokens on password change

### Encryption Best Practices

- Use Web Crypto API for AES-256-GCM (audited, native implementation)
- Use ML-KEM-768 (CRYSTALS-Kyber) for post-quantum secure key exchange
- Generate cryptographically secure random IVs for each encryption operation
- Use authenticated encryption (GCM mode)
- Implement proper key derivation (high iteration count, 100k+ for PBKDF2)
- Store salt securely (per-user, encrypted in database)
- Encrypt all files at rest on device
- Version encryption format for future upgrades
- Consider hybrid encryption: ML-KEM-768 for key exchange, AES-256-GCM for content

---

## Self-Hosting Documentation Requirements

### Provide Clear Instructions For:

1. **CouchDB Installation**
   - Docker Compose file with proper configuration
   - Environment variables for admin credentials
   - CORS configuration for Tauri apps
   - SSL/TLS setup instructions
   - Backup and restore procedures

2. **Network Configuration**
   - Port requirements (5984 for CouchDB)
   - Firewall rules
   - Reverse proxy setup (nginx/Caddy examples)
   - Domain and SSL certificate setup

3. **User Management**
   - How to create user accounts in CouchDB with 2FA
   - User document structure with TOTP secrets
   - Database per-user setup
   - Setting up replication with session authentication
   - Managing permissions and roles
   - 2FA backup code management

4. **Security Hardening**
   - Change default admin credentials
   - Enable authentication
   - Configure HTTPS only
   - Rate limiting recommendations
   - Regular security updates

5. **Monitoring & Maintenance**
   - Health check endpoints
   - Log locations
   - Backup strategies
   - Database compaction
   - Performance tuning

---

## Platform-Specific Considerations

### Desktop (Windows, macOS, Linux)

**Features to Implement:**
- System tray integration (show sync status, quick actions)
- Global keyboard shortcuts (quick note creation)
- File associations (open .md files with app)
- Native menus (File, Edit, View, etc.)
- Auto-updater (Tauri's built-in updater)

**Platform Differences:**
- Windows: Installer via NSIS or MSI
- macOS: DMG or App Store bundle, code signing required
- Linux: AppImage, .deb, or .rpm packages

### Mobile (iOS, Android)

**Features to Implement:**
- Share extension (save content from other apps)
- Widgets (quick note access, recent notes)
- Biometric authentication (Face ID, Touch ID, fingerprint)
- Dark mode support (follow system preference)
- Keyboard toolbar (markdown shortcuts)

**Platform Differences:**
- iOS: Xcode build, App Store submission, requires macOS
- Android: Android Studio build, Play Store or F-Droid
- Handle different screen sizes and aspect ratios
- Test on tablets and phones

---

## Development Workflow Best Practices

### During Development:

1. **Always test offline scenarios**
   - Disconnect network and verify app works
   - Test sync recovery when reconnecting
   - Ensure no data loss during interruptions

2. **Test on actual devices early**
   - Don't wait until late in development
   - Mobile emulators miss real-world issues
   - Test on low-end and high-end devices

3. **Monitor performance continuously**
   - Profile slow operations
   - Test with large datasets (1000+ notes)
   - Check memory usage during long sessions

4. **Handle edge cases**
   - Empty states (no notes, no folders)
   - Very long note titles or content
   - Special characters in filenames
   - Concurrent edits (conflict scenarios)

5. **Implement proper error handling**
   - Never crash silently
   - Show user-friendly error messages
   - Log errors for debugging
   - Provide recovery options

---

## Known Challenges & Solutions

### Challenge: Encryption Key Management
**Issue**: Users lose recovery phrase = permanent data loss
**Mitigation**:
- Force users to write down recovery phrase before proceeding
- Show multiple warnings about importance
- Provide printable recovery phrase template
- Consider optional encrypted cloud backup of recovery phrase (Phase 2)

### Challenge: Sync Conflicts
**Issue**: Complex conflict scenarios with multiple devices
**Mitigation**:
- Thoroughly test conflict detection
- Start with simple conflict resolution (copy creation)
- Build visual merge UI in Phase 2
- Document expected behavior clearly

### Challenge: Cross-Platform Consistency
**Issue**: Different OS behaviors and APIs
**Mitigation**:
- Abstract platform-specific code behind interfaces
- Test on all platforms frequently
- Use Tauri's platform detection properly
- Document platform-specific limitations

---

## Success Criteria

### MVP is considered complete when:

✅ Users can create, edit, delete notes offline  
✅ Markdown rendering works correctly with checkboxes  
✅ Folder organization functions properly  
✅ Search returns accurate results quickly  
✅ E2E encryption is implemented and tested  
✅ Sync works bidirectionally between devices  
✅ Conflicts are detected and handled (copy creation)  
✅ 2FA authentication is functional  
✅ App works on all target platforms (Windows, macOS, Linux, iOS, Android)  
✅ Self-hosting documentation is complete and tested  
✅ Recovery phrase generation and restoration works  
✅ No critical bugs or data loss scenarios  

---

## Resources & References

### Official Documentation
- Tauri v2: https://v2.tauri.app/
- PouchDB: https://pouchdb.com/
- CouchDB: https://docs.couchdb.org/
- React: https://react.dev/
- TypeScript: https://www.typescriptlang.org/
- Tailwind CSS: https://tailwindcss.com/

### Security Standards
- Web Crypto API: https://developer.mozilla.org/en-US/docs/Web/API/Web_Crypto_API
- OWASP Encryption Guidelines
- BIP39 Mnemonic: https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki
- TOTP RFC 6238: https://datatracker.ietf.org/doc/html/rfc6238

### Helpful Libraries
- react-markdown: Markdown rendering
- remark/rehype: Markdown processing
- codemirror: Code editor
- zxcvbn: Password strength estimation
- otpauth: TOTP generation/validation (client-side)
- qrcode: QR code generation for TOTP enrollment
- liboqs-js or kyber-crystals: ML-KEM-768 implementation
- bip39: Mnemonic phrase generation

Remember: Focus on getting MVP features stable before adding enhancements. Better to ship a solid, limited app than a buggy full-featured one.