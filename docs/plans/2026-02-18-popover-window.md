# Popover Window Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the tray-click-shows-main-window behavior with a native-look popover anchored below the status bar icon, displaying session status dots and a minimal session list.

**Architecture:** A second Tauri WebviewWindow (`popover`) loads the existing `/popover` SvelteKit route. The Rust tray click handler uses the click event's `rect` position to place the frameless window just below the tray icon. The popover auto-hides on blur (click-outside). The frontend is a minimal Svelte page with status dots header, one-line session cards, and an "Open Dashboard" footer.

**Tech Stack:** Tauri 2 (Rust), SvelteKit, Svelte 5 runes, existing CSS design tokens (Vercel Noir)

---

### Task 1: Add popover window to Tauri config and capabilities

**Files:**
- Modify: `src-tauri/tauri.conf.json:13-25` (add popover window to `app.windows` array)
- Modify: `src-tauri/capabilities/default.json:5` (add `"popover"` to windows array)

**Step 1: Add popover window config**

In `src-tauri/tauri.conf.json`, add a second entry to the `app.windows` array after the main window:

```json
{
  "label": "popover",
  "title": "c9watch",
  "url": "/popover",
  "width": 320,
  "height": 400,
  "visible": false,
  "center": false,
  "resizable": false,
  "decorations": false,
  "transparent": true,
  "alwaysOnTop": true,
  "skipTaskbar": true,
  "shadow": false,
  "focus": false
}
```

**Step 2: Add popover to capabilities**

In `src-tauri/capabilities/default.json`, change `"windows": ["main"]` to `"windows": ["main", "popover"]`.

**Step 3: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: Compiles without errors.

**Step 4: Commit**

```bash
git add src-tauri/tauri.conf.json src-tauri/capabilities/default.json
git commit -m "feat: add popover window config to Tauri"
```

---

### Task 2: Wire tray click to toggle popover window with positioning

**Files:**
- Modify: `src-tauri/src/lib.rs:250-269` (replace tray click handler)

**Step 1: Update imports**

At the top of `src-tauri/src/lib.rs`, add `WebviewWindowBuilder` and `PhysicalPosition` to the Tauri imports. The existing import block at line 25-28 becomes:

```rust
use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, WebviewUrl,
};
use tauri::{AppHandle, Manager};
```

Note: `WebviewUrl` is needed for building the popover window programmatically if it doesn't exist yet.

**Step 2: Replace tray click handler**

Replace lines 250-269 in `lib.rs` (the tray icon section) with:

```rust
            // ── Tray icon ───────────────────────────────────────
            let app_handle = app.handle().clone();
            TrayIconBuilder::new()
                .icon(tauri::include_image!("icons/tray-icon.png"))
                .icon_as_template(true)
                .tooltip("c9watch")
                .on_tray_icon_event(move |_tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        rect,
                        ..
                    } = event
                    {
                        if let Some(popover) = app_handle.get_webview_window("popover") {
                            if popover.is_visible().unwrap_or(false) {
                                let _ = popover.hide();
                            } else {
                                // Position below the tray icon, centered horizontally
                                let tray_x = rect.position.x;
                                let tray_y = rect.position.y;
                                let tray_width = rect.size.width;
                                let popover_width = 320.0;

                                let x = tray_x + (tray_width / 2.0) - (popover_width / 2.0);
                                let y = tray_y + rect.size.height + 4.0;

                                let _ = popover.set_position(
                                    tauri::PhysicalPosition::new(x as i32, y as i32),
                                );
                                let _ = popover.show();
                                let _ = popover.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;
```

**Step 3: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: Compiles without errors. The popover window is created from config, so `get_webview_window("popover")` will find it.

**Step 4: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: wire tray click to toggle popover with positioning"
```

---

### Task 3: Rewrite the popover frontend with minimal design

**Files:**
- Modify: `src/routes/popover/+page.svelte` (full rewrite)

**Step 1: Rewrite the popover page**

Replace the entire contents of `src/routes/popover/+page.svelte` with the new minimal design:

```svelte
<script lang="ts">
	import { onMount } from 'svelte';
	import { get } from 'svelte/store';
	import { sortedSessions, statusSummary, sessions as sessionsStore, initializeSessionListeners } from '$lib/stores/sessions';
	import { openSession, getSessions } from '$lib/api';
	import { SessionStatus } from '$lib/types';
	import type { Session } from '$lib/types';
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
	import { isDemoMode, loadDemoDataIfActive } from '$lib/demo';

	let sessions = $derived($sortedSessions);
	let summary = $derived($statusSummary);
	let totalSessions = $derived(sessions.length);

	onMount(() => {
		let unlistenFocus: (() => void) | null = null;
		let unlistenBlur: (() => void) | null = null;

		const init = async () => {
			const demoActive = loadDemoDataIfActive();
			await initializeSessionListeners();

			if (!demoActive) {
				try {
					const initialSessions = await getSessions();
					sessionsStore.set(initialSessions);
				} catch (error) {
					console.error('Failed to fetch sessions:', error);
				}
			}

			// Refresh when popover becomes visible
			unlistenFocus = await listen('tauri://focus', async () => {
				if (get(isDemoMode)) return;
				try {
					const freshSessions = await getSessions();
					sessionsStore.set(freshSessions);
				} catch (error) {
					console.error('Failed to refresh sessions:', error);
				}
			});

			// Hide popover when it loses focus (click-outside)
			const currentWindow = getCurrentWebviewWindow();
			unlistenBlur = await currentWindow.onFocusChanged(({ payload: focused }) => {
				if (!focused) {
					currentWindow.hide();
				}
			});
		};

		init();

		return () => {
			if (unlistenFocus) unlistenFocus();
			if (unlistenBlur) unlistenBlur();
		};
	});

	function getStatusColor(status: SessionStatus): string {
		switch (status) {
			case SessionStatus.NeedsPermission:
				return 'var(--status-permission)';
			case SessionStatus.WaitingForInput:
				return 'var(--status-input)';
			case SessionStatus.Working:
				return 'var(--status-working)';
			default:
				return 'var(--status-connecting)';
		}
	}

	async function handleOpen(session: Session) {
		try {
			await openSession(session.pid, session.projectPath);
		} catch (error) {
			console.error('Failed to open:', error);
		}
	}

	async function openMainWindow() {
		try {
			await invoke('show_main_window');
		} catch (error) {
			console.error('Failed to open main window:', error);
		}
	}
</script>

<div class="popover">
	<header class="popover-header">
		<div class="status-dots">
			{#if summary.working > 0}
				<span class="dot-pair">
					<span class="dot" style="background: var(--status-working)"></span>
					<span class="dot-count">{summary.working}</span>
				</span>
			{/if}
			{#if summary.permission > 0}
				<span class="dot-pair">
					<span class="dot" style="background: var(--status-permission)"></span>
					<span class="dot-count">{summary.permission}</span>
				</span>
			{/if}
			{#if summary.input > 0}
				<span class="dot-pair">
					<span class="dot" style="background: var(--status-input)"></span>
					<span class="dot-count">{summary.input}</span>
				</span>
			{/if}
		</div>
		<span class="total-count">{totalSessions} session{totalSessions !== 1 ? 's' : ''}</span>
	</header>

	<main class="popover-content">
		{#if sessions.length === 0}
			<div class="empty-state">
				<p>No active sessions</p>
			</div>
		{:else}
			<div class="session-list">
				{#each sessions as session (session.id)}
					<button class="session-row" onclick={() => handleOpen(session)}>
						<span class="session-dot" style="background: {getStatusColor(session.status)}"></span>
						<span class="session-name">{session.customTitle || session.sessionName}</span>
						<svg class="open-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
							<path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
							<polyline points="15 3 21 3 21 9" />
							<line x1="10" y1="14" x2="21" y2="3" />
						</svg>
					</button>
				{/each}
			</div>
		{/if}
	</main>

	<footer class="popover-footer">
		<button class="dashboard-btn" onclick={openMainWindow}>
			Open Dashboard
			<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
				<path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
				<polyline points="15 3 21 3 21 9" />
				<line x1="10" y1="14" x2="21" y2="3" />
			</svg>
		</button>
	</footer>
</div>

<style>
	.popover {
		display: flex;
		flex-direction: column;
		height: 100vh;
		background: var(--bg-base);
		border: 1px solid var(--border-default);
		border-radius: 10px;
		overflow: hidden;
		font-family: var(--font-mono);
		user-select: none;
		-webkit-user-select: none;
	}

	.popover-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px 16px;
		border-bottom: 1px solid var(--border-default);
	}

	.status-dots {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.dot-pair {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.dot-count {
		font-size: 12px;
		font-weight: 600;
		color: var(--text-primary);
	}

	.total-count {
		font-size: 11px;
		color: var(--text-muted);
	}

	.popover-content {
		flex: 1;
		overflow-y: auto;
	}

	.empty-state {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 100%;
		color: var(--text-muted);
		font-size: 11px;
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.session-list {
		display: flex;
		flex-direction: column;
	}

	.session-row {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 8px 16px;
		border: none;
		background: transparent;
		color: var(--text-primary);
		font-family: var(--font-mono);
		font-size: 12px;
		cursor: pointer;
		transition: background var(--transition-fast);
		text-align: left;
		width: 100%;
	}

	.session-row:hover {
		background: var(--bg-elevated);
	}

	.session-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.session-name {
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.open-icon {
		flex-shrink: 0;
		opacity: 0;
		color: var(--text-muted);
		transition: opacity var(--transition-fast);
	}

	.session-row:hover .open-icon {
		opacity: 1;
	}

	.popover-footer {
		padding: 8px 12px;
		border-top: 1px solid var(--border-default);
	}

	.dashboard-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		width: 100%;
		padding: 6px 12px;
		border: 1px solid var(--border-default);
		border-radius: 6px;
		background: transparent;
		color: var(--text-secondary);
		font-family: var(--font-mono);
		font-size: 10px;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		cursor: pointer;
		transition: all var(--transition-fast);
	}

	.dashboard-btn:hover {
		background: var(--bg-elevated);
		color: var(--text-primary);
		border-color: var(--text-muted);
	}
</style>
```

**Step 2: Verify the dev server loads the route**

Run: `npm run dev` (in the project root)
Navigate to `http://localhost:1420/popover` in browser.
Expected: The popover UI renders (will show empty state if no sessions).

**Step 3: Commit**

```bash
git add src/routes/popover/+page.svelte
git commit -m "feat: rewrite popover with minimal session list design"
```

---

### Task 4: Update show_main_window to hide popover

**Files:**
- Modify: `src-tauri/src/lib.rs:168-178` (show_main_window command, already does this)

**Step 1: Verify existing behavior**

The `show_main_window` command at line 168-178 already hides the popover:

```rust
if let Some(popover) = app.get_webview_window("popover") {
    let _ = popover.hide();
}
```

No code change needed. This task confirms the behavior is already correct.

**Step 2: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: Compiles without errors.

---

### Task 5: End-to-end test

**Step 1: Run the full app**

Run: `cd /Users/liminchen/Documents/GitHub/c9watch && npm run tauri dev`

**Step 2: Verify tray behavior**

1. Click the tray icon — popover should appear anchored below the icon
2. Click the tray icon again — popover should hide (toggle)
3. Click away from the popover — it should auto-hide (blur)
4. Click tray icon, then click "Open Dashboard" — main window shows, popover hides
5. If Claude sessions are running, verify they appear in the session list with correct status colors
6. Click a session row — should open/focus the terminal with that session

**Step 3: Verify edge cases**

1. No active sessions — should show "No active sessions"
2. Many sessions (if possible) — popover should scroll

**Step 4: Commit all remaining changes (if any adjustments were needed)**

```bash
git add -A
git commit -m "feat: complete popover window implementation"
```
