<div align="center">

<img src="client/EaseMC.png" width="120" alt="EaseMC" />

# EaseMC

A zero-setup, one-click Minecraft launcher for beginners.

[![Tauri](https://img.shields.io/badge/Tauri_2-FFC131?style=flat-square&logo=tauri&logoColor=black)](https://tauri.app)
[![Svelte 5](https://img.shields.io/badge/Svelte_5-FF3E00?style=flat-square&logo=svelte&logoColor=white)](https://svelte.dev)
[![Rust](https://img.shields.io/badge/Rust-000000?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows-blue?style=flat-square)](#)

**English** | [简体中文](README.zh.md)

</div>

## About

EaseMC is an open-source Minecraft launcher focused on giving beginners the easiest possible way into the game. No manual Java installation, no version manifests to wrangle — click the download button and the launcher takes care of everything: fetching the game files, installing a bundled Java runtime, then launching with a single click.

It also ships with a built-in AI game assistant (powered by DeepSeek) that answers beginner questions about survival, crafting, and exploration.

> EaseMC targets Minecraft **1.21.1** (vanilla, offline mode). The UI is currently Chinese only.

## Features

- **Zero setup** — Click the download button and the app fetches all game files plus a Java 21 runtime (Azul Zulu). Nothing to install manually.
- **Fast, resilient downloads** — 8 concurrent connections with real-time progress; BMCLAPI mirror acceleration for China, with automatic fallback to official Mojang sources.
- **One-click launch / stop** — Start or end the game from a single button; the launcher manages the game process for you.
- **Built-in AI assistant** — Streaming chat powered by DeepSeek, pre-loaded with launcher-specific knowledge (installation, file locations, troubleshooting), with Markdown rendering and quick prompts.
- **Cross-platform** — macOS and Windows.
- **Lightweight** — Built on Tauri 2: a small Rust core driving a web-based UI.

## Tech Stack

| Layer | Technology |
|---|---|
| Frontend | Svelte 5 + Vite 6, `@tauri-apps/api`, `lucide-svelte`, `marked` |
| Backend | Tauri 2 + Rust (`tokio`, `reqwest`, `zip` / `flate2` / `tar`) |
| AI | `async-openai` calling DeepSeek with streaming responses |

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/)
- [Rust](https://rustup.rs/) (including the system dependencies Tauri requires for your platform — see the [Tauri prerequisites guide](https://tauri.app/start/prerequisites/))

### Development

```bash
cd client

# Install frontend dependencies
npm install

# Start the Tauri dev environment (launches Vite and Rust together)
npm run tauri dev

# Frontend-only hot reload (browser preview; backend features unavailable)
npm run dev
```

### Build

```bash
npm run tauri build
```

Installers are output to `client/src-tauri/target/release/bundle/`.

### Backend Tests

```bash
cd client/src-tauri

# Fast tests
cargo test

# Network-dependent tests (marked #[ignore], need real connectivity)
cargo test -- --ignored --nocapture
```

## Configuration

### Launcher Config — `client/src-tauri/config.json`

Bundled as a resource; copied to the system app-config directory on first run.

```json
{
  "gameIsInstalled": "false",
  "macJrePath": "https://cdn.azul.com/zulu/.../zulu21-macosx_x64.tar.gz",
  "winJrePath": "https://cdn.azul.com/zulu/.../zulu21-win_x64.zip",
  "serverUrl": "http://localhost:3000"
}
```

| Field | Description |
|---|---|
| `gameIsInstalled` | Whether the game is already installed; the launcher decides whether to auto-download |
| `macJrePath` / `winJrePath` | Download URLs for the platform's JRE 21 archive (tar.gz / zip) |
| `serverUrl` | Reserved for future use |

### AI Assistant

Create a `.env` file in the `client/` directory (the working directory of `npm run tauri dev`) with your DeepSeek API key:

```
DEEPSEEK_API_KEY=sk-xxxxxxxx
```

Do not commit this file. If the key is missing, only the AI assistant is disabled — game download and launch work normally.

## How It Works

1. **First run** — the launcher checks installation status; clicking the download button fetches all game artifacts concurrently (via the BMCLAPI mirror, falling back to Mojang), then downloads and extracts the Java 21 runtime.
2. **Launch** — the Rust backend assembles the classpath from `game/.minecraft/libraries` plus the client jar, then spawns the `java` process with the correct native-library arguments for each platform.
3. **Game files** live in a `game/` directory next to the executable — portable and self-contained.

## Project Structure

```
client/
├── index.html
├── package.json
├── vite.config.js
├── src/                      # Svelte frontend
│   ├── main.js
│   ├── App.svelte            # Root component: state orchestration + business logic
│   ├── App.css               # Global styles
│   ├── constants.js          # Constants (gameInfo / quickPrompts / isTauri)
│   ├── assets/
│   └── components/           # UI components
│       ├── LauncherView.svelte # Normal mode page (background + hero + launch card)
│       ├── AdvancedMode.svelte # Advanced mode page (own background & layout; placeholder, click the top-left icon)
│       ├── Scene.svelte      # Background scene (normal mode)
│       ├── TopBar.svelte     # Top bar (brand + assistant entry + mode toggle)
│       ├── HeroSection.svelte# Hero section
│       ├── LaunchCard.svelte # Launch card (status text)
│       ├── LaunchButton.svelte # Launch/download button (with wave animation)
│       ├── ChatPanel.svelte  # AI assistant drawer (message list + input)
│       └── ToastStack.svelte # Toast notifications
└── src-tauri/                # Tauri + Rust backend
    ├── tauri.conf.json
    ├── config.json           # Runtime config (see above)
    ├── capabilities/default.json
    └── src/
        ├── main.rs           # Entry point: logging + dotenv init
        ├── lib.rs            # Tauri command registration
        ├── game.rs           # Launch / stop the Minecraft process
        ├── download.rs       # Game file downloads (mirrors, concurrency, progress events)
        ├── jre.rs            # Download & extract the JRE
        ├── chat.rs           # DeepSeek streaming AI assistant
        ├── config.rs         # Read / write config
        └── game.json         # Minecraft 1.21.1 version manifest (embedded at compile time)
```

## Known Limitations

- Vanilla Minecraft 1.21.1 only; no mod support yet
- Offline mode — no Microsoft account login
- On Apple Silicon, the x64 JRE runs via Rosetta (native arm64 is not yet supported)
- The UI is Chinese only (English localization is welcome)

## Roadmap

- Multiple game versions
- Mod loader support (Fabric / Forge)
- Native Apple Silicon support
- SHA1 integrity verification for downloaded files
- English localization

## Contributing

Issues and pull requests are welcome. To work on the project locally, follow the steps in [Getting Started](#getting-started).

## Copyright / Legal

This project is an open-source third-party Minecraft launcher for **learning and non-commercial use only**.
All independently developed code is open-sourced under the corresponding license.

Minecraft is a trademark of Mojang AB and Microsoft Corporation.
This project is **not affiliated, endorsed, or sponsored** by Mojang or Microsoft.
All third-party optimization mods and resources belong to their respective authors.

Unauthorized commercial use, resale, repackaging for profit is strictly prohibited.

Copyright © 2025 All rights reserved.
