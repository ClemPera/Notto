# Note App Project Charter

## Project Overview

A cross-platform, privacy-focused note-taking application with end-to-end encryption and self-hostable sync capabilities. The application prioritizes local-first architecture with optional cloud synchronization, enabling users to maintain full control over their data.

## Core Principles

- **Local-first**: All data stored and accessible locally; sync is optional
- **Privacy-focused**: End-to-end encryption with user-controlled keys
- **Self-hostable**: Users can run their own sync infrastructure
- **Open source**: Codebase publicly available; official hosted instance available as paid service
- **Cross-platform**: Single codebase for desktop and mobile

## Target Audience

General users seeking a private, reliable note-taking solution with cross-device synchronization capabilities.

---

## Technical Architecture

### Frontend & Application Shell
- **Framework**: Tauri v2
- **UI Framework**: React/Svelte/Vue (TBD)
- **Platforms**: 
  - Desktop: Windows, macOS, Linux
  - Mobile: iOS, Android
- **Storage**: File-based markdown files on local filesystem

### Backend & Sync
- **Local Database**: PouchDB (client-side)
- **Sync Protocol**: CouchDB replication protocol
- **Sync Server**: Self-hosted CouchDB
  - Users can self-host (Docker image provided)
  - Official hosted instance available (paid)
- **Authentication**: CouchDB native authentication with TOTP 2FA support

### Security & Encryption
- **E2E Encryption**: Client-side encryption before sync and at rest
- **Key Exchange**: ML-KEM-768 (post-quantum secure)
- **File Encryption**: AES-256-GCM for note content
- **Key Derivation**: Password-based (PBKDF2/Argon2) for local key generation
- **Authentication**: Username/password with 2FA (server cannot decrypt content)
- **Recovery**: 24-word recovery phrase (user-stored)
- **2FA Recovery**: Backup codes provided during setup
- **Local Storage**: All files encrypted at rest on device

### Data Format
- **Notes**: Markdown files (.md)
- **Metadata**: JSON (frontmatter or sidecar files)
- **Organization**: Folder/subfolder hierarchy on filesystem

---

## Phase 1: MVP Features

### Core Note-Taking
- Markdown editor with live preview
- Checkbox/task list support
- Rich text formatting (bold, italic, headers, lists, code blocks)
- Note creation, editing, deletion
- All notes encrypted at rest on local device

### Organization
- Folder and subfolder hierarchy
- Single vault support
- Note search (title and content)
- Tag support (optional)

### Sync & Storage
- Local-first storage (works completely offline)
- Optional cloud sync via CouchDB
- Automatic conflict detection
- Basic conflict resolution (create conflict copies)
- Real-time sync when online

### Security
- End-to-end encryption (AES-256-GCM for content, ML-KEM-768 for key exchange)
- CouchDB native authentication (username/password + TOTP 2FA)
- 24-word recovery phrase generation
- Client-side encryption/decryption
- All files encrypted at rest on device
- Zero-knowledge architecture (server cannot decrypt)
- 2FA backup codes for account recovery

### Platform Support
- Desktop applications (Windows, macOS, Linux)
- Mobile applications (iOS, Android)
- Native installers for all platforms

### User Experience
- Offline-first (full functionality without internet)
- Fast startup and note loading
- Auto-save
- Dark/light theme
- Keyboard shortcuts

---

## Phase 2: Enhanced Features

### Advanced Organization
- **Multiple vaults**: Separate note collections with independent encryption
- Cross-vault search
- Note linking/backlinking
- Templates

### Enhanced Editor
- WYSIWYG mode (optional)
- Tables support
- Mermaid diagrams
- LaTeX/math equations
- Code syntax highlighting
- Image embeds and attachments
- Drawing/sketching canvas

### Sync Improvements
- Visual conflict resolution UI (side-by-side comparison)
- Sync status indicators
- Selective sync (sync only specific folders)
- Bandwidth optimization

### Collaboration (Future Consideration)
- Shared vaults (multiple users, same vault)
- Read-only sharing
- Comment threads

### UI/UX Enhancements
- Customizable themes
- Plugin system
- Custom CSS support
- Graph view (note connections)
- Daily notes
- Calendar view

### Mobile-Specific
- Widget support
- Share extension (save from other apps)
- Biometric authentication

---

## Technical Requirements

### Performance
- Note list rendering: <100ms for 1,000 notes
- Note opening: <50ms
- Search: <200ms for 10,000 notes
- App startup: <2s cold start

### Storage
- Efficient local storage (SQLite via PouchDB)
- Support for large vaults (10,000+ notes)
- Minimal sync bandwidth usage

### Security Requirements
- Zero-knowledge architecture (server cannot decrypt notes)
- All files encrypted at rest on device
- ML-KEM-768 for post-quantum secure key exchange
- AES-256-GCM for content encryption
- Secure key storage (OS keychain/keystore)
- No plaintext data in sync or local storage
- Regular security audits

### Compatibility
- Markdown compatibility (CommonMark standard)
- Export/import: Markdown, JSON, plain text
- Interoperability with other markdown tools

---

## Development Considerations

### Conflict Resolution Strategy (Phase 1)
- Automatic conflict detection via PouchDB
- Create conflict copies with timestamps
- User notification of conflicts
- Manual resolution (user chooses version)

### Self-Hosting Setup
- Provide Docker Compose configuration for CouchDB
- Documentation for CouchDB deployment and 2FA setup
- Environment variables for configuration
- SSL/TLS configuration guide
- User creation scripts with 2FA enrollment
- Health check endpoints

### Testing Requirements
- Unit tests for core logic
- Integration tests for sync
- E2E tests for critical user flows
- Cross-platform testing (all supported OSs)
- Offline/online transition testing
- Conflict scenario testing

### Documentation Needed
- User documentation (setup, usage, self-hosting)
- Developer documentation (architecture, contributing)
- API documentation
- Security documentation (encryption, key management)

---

## Success Criteria

### Phase 1 (MVP)
- ✅ Full offline functionality
- ✅ Reliable cross-device sync
- ✅ Zero data loss (conflict handling)
- ✅ E2E encryption verified
- ✅ Stable on all target platforms
- ✅ Self-hosting documentation complete

### Phase 2
- ✅ Multiple vault support
- ✅ Advanced markdown features
- ✅ Improved conflict resolution UX
- ✅ Performance targets met

---

## Out of Scope

### Not Included
- Real-time collaborative editing (Phase 1 & 2)
- Web browser version
- Built-in AI features
- Third-party integrations (initially)
- Video/audio recording
- Handwriting recognition

---

## Technical Stack Summary

| Component | Technology |
|-----------|-----------|
| App Framework | Tauri v2 |
| Frontend | React/Svelte/Vue |
| Local DB | PouchDB |
| Sync Server | CouchDB |
| Language (Backend) | Rust |
| Language (Frontend) | TypeScript/JavaScript |
| Encryption | AES-256-GCM (content), ML-KEM-768 (key exchange) |
| Storage Format | Markdown + JSON |
| Platforms | Windows, macOS, Linux, iOS, Android |

---

## Risk Considerations

### Technical Risks
- **Sync complexity**: CouchDB conflict resolution learning curve
- **Mobile maturity**: Tauri v2 mobile support is newly stable (Oct 2024)
- **Key management**: User recovery phrase loss = permanent data loss
- **Cross-platform bugs**: Different OS behaviors

### Mitigation Strategies
- Comprehensive testing of sync scenarios
- Early mobile platform testing
- Clear user education on recovery phrase importance
- Platform-specific QA process
- Extensive offline/online transition testing