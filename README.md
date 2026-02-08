# c9watch

> Monitor and control all your Claude Code sessions from one place.

**c9watch** (short for **c**laude cod**e** watch, like k8s for Kubernetes) is a desktop app for **macOS** and **Windows** that gives you a real-time dashboard of every Claude Code session running on your machine. No more switching between terminals to check which agent needs permission, which one is working, and which one is idle.

## Demo

[![Watch Demo](https://img.youtube.com/vi/9PdN7joYmUk/maxresdefault.jpg)](https://youtu.be/9PdN7joYmUk)

## Works with everything. Tied to nothing.

Unlike other Claude Code management tools that require you to launch sessions from within their app, **c9watch doesn't care where you start your sessions**. It discovers them automatically by scanning running processes at the OS level.

Start Claude Code from any terminal or IDE you already use -- VS Code, Zed, Cursor, Windows Terminal, iTerm2, you name it -- and c9watch picks them all up. No plugins to install. No workflows to change. No vendor lock-in.

Just open c9watch and see everything.

## Lightweight and fast.

Built with **Tauri**, **Rust**, and **Svelte** -- not Electron. The app binary is small, memory usage is minimal, and the UI stays snappy. Rust handles process scanning and file parsing at native speed. Svelte compiles away the framework overhead. You're already running multiple Claude Code agents eating up resources -- your monitoring tool shouldn't add to the pile.

## Install

### macOS

#### Quick install

```bash
curl -fsSL https://raw.githubusercontent.com/minchenlee/c9watch/main/install.sh | bash
```

#### Download

Grab the latest `.dmg` from the [Releases](https://github.com/minchenlee/c9watch/releases) page.

### Windows

#### Quick install (PowerShell)

```powershell
irm https://raw.githubusercontent.com/minchenlee/c9watch/main/install.ps1 | iex
```

This downloads and runs the latest NSIS or MSI installer silently.

#### Download

Grab the latest `.exe` (NSIS installer) or `.msi` from the [Releases](https://github.com/minchenlee/c9watch/releases) page.

#### System requirements

- Windows 10 (1803+) or Windows 11
- [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (pre-installed on Windows 11; the NSIS installer will prompt to install it if missing)

### Build from source

Prerequisites: [Rust](https://rustup.rs/), [Node.js](https://nodejs.org/) (v18+), and the [Tauri CLI](https://v2.tauri.app/start/prerequisites/).

**macOS additional prerequisites:** Xcode Command Line Tools.

**Windows additional prerequisites:** [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with the "Desktop development with C++" workload, and [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/).

```bash
git clone https://github.com/minchenlee/c9watch.git
cd c9watch
npm install
npm run tauri build
```

Output locations:
- **macOS:** `src-tauri/target/release/bundle/macos/c9watch.app`
- **Windows:** `src-tauri/target/release/bundle/nsis/c9watch_<version>_x64-setup.exe`
- **Windows (MSI):** `src-tauri/target/release/bundle/msi/c9watch_<version>_x64_en-US.msi`

## Running

Launch c9watch from:
- **macOS:** Spotlight, Launchpad, or `open /Applications/c9watch.app`
- **Windows:** Start Menu, or search for "c9watch"

Once running:
1. c9watch appears in the **system tray** (macOS menu bar / Windows notification area)
2. Click the tray icon to open the dashboard
3. All running Claude Code sessions are detected automatically -- no configuration needed
4. Sessions are sorted by priority: sessions needing permission surface to the top

### Keyboard shortcuts

| Action | macOS | Windows |
|--------|-------|---------|
| Toggle demo mode | `Cmd+D` | `Ctrl+D` |

## Screenshots

### Status view -- see what needs your attention first

Sessions grouped by status. Permission requests surface to the top so you never leave an agent stuck waiting.

![Status view](docs/screenshots/status-view.png)

### Project view -- organize by codebase

Sessions grouped by project, each with its own status columns. See what's happening across all your repos.

![Project view](docs/screenshots/project-view.png)

### Compact view -- monitor at a glance

Minimal cards for when you just need a quick status check without the details.

![Compact view](docs/screenshots/compact-view.png)

### Conversation viewer -- inspect any session

Expand any card to see the full conversation history with formatted code, tool usage, and a navigation map.

![Conversation viewer](docs/screenshots/conversation-view.png)

## Features

- **Zero-integration setup** -- Works with any terminal or IDE, no plugins or extensions required
- **Auto-discovery** -- Detects all running Claude Code sessions by scanning processes at the OS level
- **Real-time status** -- See at a glance which sessions are Working, Need Permission, or Idle
- **Conversation viewer** -- Expand any session to view the full conversation with formatted markdown and code blocks
- **Session control** -- Stop sessions, open their parent terminal/IDE, or rename them for easier tracking
- **Multi-project view** -- Sessions grouped by project with git branch info
- **System tray integration** -- Quick access from the macOS menu bar or Windows notification area
- **Cross-platform** -- Native support for macOS and Windows 11

### Supported terminals and IDEs

| Application | macOS | Windows |
|-------------|-------|---------|
| VS Code | Yes | Yes |
| Cursor | Yes | Yes |
| Windsurf | Yes | Yes |
| Zed | Yes | Yes |
| Terminal.app | Yes | -- |
| iTerm2 | Yes | -- |
| Windows Terminal | -- | Yes |
| PowerShell | -- | Yes |
| Command Prompt | -- | Yes |
| Alacritty | Yes | Yes |
| kitty | Yes | Yes |
| WezTerm | -- | Yes |
| Warp | Yes | -- |
| Hyper | Yes | Yes |

## How it works

1. A background thread polls every 2 seconds, scanning for running `claude` processes using `sysinfo`
2. Each process is matched to its session file in `~/.claude/projects/` via path encoding and timestamp correlation
3. The last N entries of each session's JSONL file are parsed to determine status:
   - **Working** -- Claude is generating a response or executing tools
   - **Needs Permission** -- A tool is pending that requires user approval
   - **Idle** -- Session is waiting for your next prompt
4. Status updates are pushed to the Svelte frontend via Tauri events
5. The UI reactively updates, sorting sessions by priority (permission requests surface first)

### Platform-specific details

- **macOS:** Process tree walking via `ps`, window activation via AppleScript, IDE CLI detection in `/Applications/`
- **Windows:** Process tree walking via WMIC, window activation via PowerShell/Win32 `SetForegroundWindow`, IDE CLI detection in `%LOCALAPPDATA%` and `%PROGRAMFILES%`
- **Process termination:** `SIGTERM` on Unix, `taskkill` on Windows (graceful first, then force)

## Tech stack

| Layer | Technology |
|-------|-----------|
| Desktop framework | [Tauri 2](https://v2.tauri.app/) |
| Frontend | [SvelteKit](https://svelte.dev/) + [Svelte 5](https://svelte.dev/docs/svelte/overview) |
| Backend | Rust |
| Process discovery | [sysinfo](https://crates.io/crates/sysinfo) (cross-platform) |
| Windows APIs | [windows-sys](https://crates.io/crates/windows-sys) |
| Design system | Vercel Noir (true black, [Geist](https://vercel.com/font) fonts) |

## Development

```bash
npm install
npm run tauri dev
```

This starts both the Vite dev server (hot-reload for the frontend) and the Tauri Rust backend.

### Project structure

```
c9watch/
├── src/                    # SvelteKit frontend
│   ├── routes/             # Pages (+page.svelte, +layout.svelte)
│   ├── lib/
│   │   ├── components/     # Svelte components (SessionCard, MessageBubble, etc.)
│   │   ├── stores/         # Reactive state management
│   │   ├── demo/           # Demo mode with mock data
│   │   ├── api.ts          # Tauri command wrappers
│   │   └── types.ts        # TypeScript types
│   └── app.css             # Global styles
├── src-tauri/              # Rust backend (Tauri)
│   └── src/
│       ├── lib.rs          # Tauri commands and app setup
│       ├── polling.rs      # Background session detection loop
│       ├── actions.rs      # Stop/open session actions (cross-platform)
│       └── session/
│           ├── detector.rs # Process-to-session matching
│           ├── status.rs   # Status determination logic
│           ├── parser.rs   # JSONL file parsing
│           └── permissions.rs # Auto-approval rule checking
├── install.sh              # macOS installer script
├── install.ps1             # Windows installer script (PowerShell)
└── .github/workflows/      # CI/CD for macOS + Windows
```

### Testing

```bash
# Rust unit + integration tests
cargo test --manifest-path src-tauri/Cargo.toml

# Svelte type checking
npx svelte-kit sync && npx svelte-check

# Frontend production build
npm run build
```

## Demo mode

Press `Cmd+D` (macOS) or `Ctrl+D` (Windows) to toggle demo mode, which loads simulated sessions with animated status transitions. Useful for testing the UI without running real Claude Code sessions.

## License

MIT
