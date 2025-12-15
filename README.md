# RAVN

**The modern email client for digital natives**

RAVN is a feature-rich desktop email client built with modern web technologies. It combines the power of a native desktop application with the flexibility of a web frontend, offering a superior email experience for power users.

![Version](https://img.shields.io/badge/version-25.12.15-blue)
![Nuxt](https://img.shields.io/badge/Nuxt-4.2-00DC82)
![Tauri](https://img.shields.io/badge/Tauri-2.9-FFC131)
![License](https://img.shields.io/github/license/ravnmail/ravn)

## Features

### Email Management
- **Multi-Account Support** - Manage Gmail, Office365, Apple Mail, and any IMAP account
- **Unified Views** - View emails from all accounts in one place
- **Smart Organization** - Folders, labels, conversation threading, and full-text search
- **Intelligent Sync** - Incremental synchronization with configurable intervals per folder

### Composition & Sending
- **Rich Text Editor** - Powered by TipTap with formatting, code blocks, tables, and more
- **Email Signatures** - Per-account signatures with HTML support
- **Keyboard Shortcuts** - Cmd/Ctrl+Enter to send, Cmd/Ctrl+S for drafts

### Advanced Features
- **Full-Text Search** - Fast search across all accounts using Tantivy
- **AI Integration** - Email analysis and enhancement with Corvus AI
- **Contact Management** - Auto-extracted contacts with avatars and statistics
- **Secure Credentials** - OS-level keyring storage (Keychain, Credential Manager, Secret Service)
- **Deep Linking** - Custom `ravn://` URL scheme for navigation

## Technology Stack

### Frontend
- **Framework**: Nuxt 4.2 (Vue 3)
- **Styling**: Tailwind CSS 4
- **UI Components**: shadcn, reka UI, custom component library
- **Rich Text**: TipTap 2.27 (ProseMirror)
- **Icons**: Lucide Icons
- **Utilities**: vueUse, dayjs, lodash

### Desktop & Backend
- **Runtime**: Tauri 2.9 (Rust-based)
- **Database**: SQLite with sqlx
- **Email Protocols**: async-imap, lettre (SMTP), mail-parser
- **Authentication**: OAuth2 for cloud providers
- **Search**: Tantivy full-text search engine
- **Security**: OS keyring integration, AES-GCM encryption

## Prerequisites

- **Node.js**: 22.x or higher
- **Package Manager**: bun
- **Rust**: 1.19.1 or higher (for Tauri)
- **Cargo**: Latest stable

## Installation

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd ravn
   ```

2. **Install dependencies**
   ```bash
   bun install
   ```

3. **Set up environment variables** (optional)
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

## Development

### Web Development Server

Run the desktop application in development mode:

```bash
npm run tauri:dev
```

This will start both the Nuxt dev server and the Tauri desktop application with hot-reload enabled.

## Building for Production

### Build Desktop Application

```bash
npm run tauri:build
```

This will:
1. Generate the optimized Nuxt static site
2. Compile the Rust backend
3. Package the desktop application for your platform

The built application will be available in `src-tauri/target/release`.

## Project Structure

```
ravn/
├── app/                    # Nuxt frontend application
│   ├── components/         # Vue components
│   │   ├── ui/             # UI component library
│   │   └── ...             # Email, sidebar, navigation components
│   ├── pages/              # Route pages
│   │   ├── auth/           # Authentication flows
│   │   ├── mail/           # Email views
│   │   ├── settings/       # Settings pages
│   │   └── ...
│   ├── composables/        # Vue composables
│   ├── lib/                # Utilities and libraries
│   │   └── editor/         # TipTap editor configuration
│   ├── layouts/            # Page layouts
│   ├── assets/             # Static assets
│   └── types/              # TypeScript types
│
├── src-tauri/              # Tauri backend (Rust)
│   ├── src/
│   │   ├── commands/       # Tauri command handlers
│   │   ├── database/       # Database models & repositories
│   │   ├── sync/           # Email synchronization subsystem
│   │   │   ├── providers/  # Email provider implementations
│   │   │   └── ...         # Sync logic, OAuth, workers
│   │   ├── services/       # Business logic services
│   │   ├── search/         # Full-text search
│   │   └── main.rs         # Application entry point
│   ├── migrations/         # Database migrations
│   ├── Cargo.toml          # Rust dependencies
│   └── tauri.conf.json     # Tauri configuration
│
├── nuxt.config.ts          # Nuxt configuration
├── tailwind.config.ts      # Tailwind CSS configuration
└── package.json            # Node dependencies
```

## Configuration

### Email Accounts

RAVN supports multiple email providers:

- **Gmail**: OAuth2 authentication
- **Office365**: OAuth2 authentication
- **Apple/iCloud**: Standard IMAP
- **Custom IMAP**: Any IMAP-compatible server

Configure accounts through the application's account setup wizard (`/auth/add-account`).

### AI Features

RAVN includes AI-powered features via Corvus AI. Configure in Settings > AI.

## Database

RAVN uses SQLite for local data storage with the following key tables:

- `accounts` - Email account configurations
- `folders` - Mailbox folder hierarchy
- `emails` - Email messages with metadata
- `attachments` - File attachments with caching
- `conversations` - Threaded email conversations
- `contacts` - Auto-extracted contact information
- `labels` - User-defined email labels
- `signatures` - Email signatures per account

Database migrations are managed through sqlx and located in `src-tauri/migrations/`.

## Security

RAVN takes security seriously:

- **Credential Storage**: Uses OS-level keyring (Keychain on macOS, Credential Manager on Windows, Secret Service on Linux)
- **Encrypted Fallback**: AES-GCM encryption when keyring is unavailable
- **OAuth2 with PKCE**: Secure authentication for cloud providers
- **No Credential Logging**: Credentials never appear in logs
- **Zero-Trust Rendering**: Blocks remove images by default

## Architecture Highlights

### Sync Subsystem
- Provider-agnostic design with trait-based architecture
- Incremental sync (only fetch new emails)
- Configurable sync intervals per folder type
- Background workers for sync, body fetching, avatars, and AI analysis
- Real-time UI updates via Tauri events

### Email Rendering
- HTML rendering with CSS inlining for client compatibility
- Automatic plain-text fallback generation
- Inline and regular attachment support
- CID (Content-ID) mapping for embedded images

### Search System
- Full-text search using Tantivy
- Multi-account search support
- Real-time index updates
- Efficient query parsing

## Authors

**Michael Wallner @ Coder's Cantina**

## License

This project is licensed under the GNU General Public License v3.0. See the [LICENSE](LICENSE) file for details.
