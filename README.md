# c9watch

> Monitor and control all your Claude Code sessions from one place.

**c9watch** (short for **c**laude cod**e** watch, like k8s for Kubernetes) is a macOS desktop app that gives you a real-time dashboard of every Claude Code session running on your machine. No more switching between terminals to check which agent needs permission, which one is working, and which one is idle.

## Works with everything. Tied to nothing.

Unlike other Claude Code management tools that require you to launch sessions from within their app, **c9watch doesn't care where you start your sessions**. It discovers them automatically by scanning running processes at the OS level.

Start Claude Code from anywhere you already work -- **VS Code**, **Zed**, **Antigravity**, **iTerm2**, **Terminal.app**, or any other terminal -- and c9watch picks them all up. No plugins to install. No workflows to change. No vendor lock-in.

Just open c9watch and see everything.

## Features

- **Zero-integration setup** -- Works with any terminal or IDE, no plugins or extensions required
- **Auto-discovery** -- Detects all running Claude Code sessions by scanning processes at the OS level
- **Real-time status** -- See at a glance which sessions are Working, Need Permission, or Idle
- **Conversation viewer** -- Expand any session to view the full conversation with formatted markdown and code blocks
- **Session control** -- Stop sessions, open their parent terminal/IDE, or rename them for easier tracking
- **Multi-project view** -- Sessions grouped by project with git branch info
- **Menu bar integration** -- Quick access from the macOS tray icon

## Screenshots

<!-- Add screenshots here -->

## Install

### Homebrew (recommended)

```bash
brew tap <your-username>/tap
brew install --cask c9watch
```

### Download

Grab the latest `.dmg` from the [Releases](https://github.com/anthropics/c9watch/releases) page.

### Build from source

Prerequisites: [Rust](https://rustup.rs/), [Node.js](https://nodejs.org/) (v18+), and the [Tauri CLI](https://v2.tauri.app/start/prerequisites/).

```bash
git clone https://github.com/anthropics/c9watch.git
cd c9watch
npm install
npm run tauri build
```

The built `.app` will be in `src-tauri/target/release/bundle/macos/`.

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
│       ├── actions.rs      # Stop/open session actions
│       └── session/
│           ├── detector.rs # Process-to-session matching
│           ├── status.rs   # Status determination logic
│           ├── parser.rs   # JSONL file parsing
│           └── permissions.rs # Auto-approval rule checking
└── homebrew/               # Homebrew cask distribution
```

## How it works

1. A background thread polls every 2 seconds, scanning for running `claude` processes using `sysinfo`
2. Each process is matched to its session file in `~/.claude/projects/` via path encoding and timestamp correlation
3. The last N entries of each session's JSONL file are parsed to determine status:
   - **Working** -- Claude is generating a response or executing tools
   - **Needs Permission** -- A tool is pending that requires user approval
   - **Idle** -- Session is waiting for your next prompt
4. Status updates are pushed to the Svelte frontend via Tauri events
5. The UI reactively updates, sorting sessions by priority (permission requests surface first)

## Tech stack

| Layer | Technology |
|-------|-----------|
| Desktop framework | [Tauri 2](https://v2.tauri.app/) |
| Frontend | [SvelteKit](https://svelte.dev/) + [Svelte 5](https://svelte.dev/docs/svelte/overview) |
| Backend | Rust |
| Process discovery | [sysinfo](https://crates.io/crates/sysinfo) |
| Design system | Vercel Noir (true black, [Geist](https://vercel.com/font) fonts) |

## Demo mode

Press `Cmd+D` to toggle demo mode, which loads simulated sessions with animated status transitions. Useful for testing the UI without running real Claude Code sessions.

## License

MIT
