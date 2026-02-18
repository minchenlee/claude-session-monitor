# Popover Window Design

## Summary

Replace the current tray-click behavior (showing the main window) with a native-look popover that anchors below the status bar icon, showing a quick overview of all Claude Code sessions.

## Layout

```
┌──────────────────────────────────────┐
│  ●3  ●1  ●2              6 sessions │  header: colored dots + total
├──────────────────────────────────────┤
│ ● c9watch                      [↗] │  session list: dot + name + open
│ ● my-api                       [↗] │  (sorted by attention priority)
│ ● frontend                     [↗] │
│ ● docs-site                    [↗] │
├──────────────────────────────────────┤
│          Open Dashboard          [↗]│  footer: opens main window
└──────────────────────────────────────┘
```

## Design Decisions

- **Header**: Three colored dots (purple=Working, orange=Permission, green=Idle) each with count, plus total session count on the right. Dots with zero count are hidden.
- **Session list**: Minimal one-line cards. Status dot (colored by status) + project name + open-window button. All sessions shown, sorted by attention priority (Permission > Idle > Working > Connecting).
- **Footer**: "Open Dashboard" button that shows the main window and hides the popover.
- **Empty state**: "No active sessions" message when no sessions exist.

## Technical Approach

### Tauri WebView Popover Window

Create a second Tauri window that loads the `/popover` route. On tray click, position and show a frameless, borderless window anchored below the tray icon.

### Changes Required

1. **`tauri.conf.json`** — Add `popover` window config (frameless, transparent, ~320x400px, hidden by default)
2. **`src-tauri/src/lib.rs`** — Change tray click handler to create/show/position the popover window below the tray icon instead of showing the main window. Add a `toggle_popover` command.
3. **`src/routes/popover/+page.svelte`** — Rewrite with the minimal design (dots header, one-line session cards, footer)
4. **Click-outside dismiss** — Use Tauri's `blur` event on the popover window to auto-hide it

### Popover Window Properties

- Frameless (no title bar)
- Not resizable
- Skip taskbar
- Always on top
- Hidden by default
- Transparent background (for rounded corners if desired)
- Decorations disabled

### Positioning

Use `TrayIconEvent::Click` position data to place the popover window just below the tray icon, aligned to the right edge of the screen (standard macOS behavior for menu bar extras).
