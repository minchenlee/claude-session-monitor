<script lang="ts">
	import { onMount } from 'svelte';
	import { get } from 'svelte/store';
	import { sortedSessions, statusSummary, sessions as sessionsStore, initializeSessionListeners } from '$lib/stores/sessions';
	import { openSession, getSessions } from '$lib/api';
	import { SessionStatus } from '$lib/types';
	import type { Session } from '$lib/types';
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import { isDemoMode, loadDemoDataIfActive } from '$lib/demo';

	let sessions = $derived($sortedSessions);
	let summary = $derived($statusSummary);
	let totalSessions = $derived(sessions.length);

	onMount(() => {
		let unlistenFocus: (() => void) | null = null;
		let cancelled = false;

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

			const ul1 = await listen('tauri://focus', async () => {
				if (get(isDemoMode)) return;
				try {
					const freshSessions = await getSessions();
					sessionsStore.set(freshSessions);
				} catch (error) {
					console.error('Failed to refresh sessions:', error);
				}
			});

			if (cancelled) {
				ul1();
			} else {
				unlistenFocus = ul1;
			}
		};

		init();

		return () => {
			cancelled = true;
			if (unlistenFocus) unlistenFocus();
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
					<button class="session-card" onclick={() => handleOpen(session)}>
						<div class="card-top">
							<span class="session-dot" style="background: {getStatusColor(session.status)}"></span>
							<span class="session-project">{session.sessionName}</span>
							<svg aria-hidden="true" class="open-icon" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
								<path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
								<polyline points="15 3 21 3 21 9" />
								<line x1="10" y1="14" x2="21" y2="3" />
							</svg>
						</div>
						<div class="card-title">{session.customTitle || session.firstPrompt}</div>
						{#if session.latestMessage}
							<div class="card-latest">{session.latestMessage}</div>
						{/if}
					</button>
				{/each}
			</div>
		{/if}
	</main>

	<footer class="popover-footer">
		<button class="dashboard-btn" onclick={openMainWindow}>
			Open Dashboard
			<svg aria-hidden="true" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
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
		flex-shrink: 0;
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

	.empty-state p {
		margin: 0;
	}

	.session-list {
		display: flex;
		flex-direction: column;
	}

	.session-card {
		display: flex;
		flex-direction: column;
		gap: 3px;
		padding: 9px 16px;
		border: none;
		border-bottom: 1px solid var(--border-muted);
		background: transparent;
		color: var(--text-primary);
		font-family: var(--font-mono);
		cursor: pointer;
		transition: background var(--transition-fast);
		text-align: left;
		width: 100%;
	}

	.session-card:last-child {
		border-bottom: none;
	}

	.session-card:hover {
		background: var(--bg-elevated);
	}

	.session-card:focus-visible {
		outline: none;
	}

	.card-top {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.session-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.session-project {
		flex: 1;
		font-size: 12px;
		font-weight: 600;
		color: var(--text-primary);
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

	.session-card:hover .open-icon {
		opacity: 1;
	}

	.card-title {
		font-size: 11px;
		color: var(--text-secondary);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		padding-left: 14px;
	}

	.card-latest {
		font-size: 10px;
		color: var(--text-muted);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		padding-left: 14px;
	}

	.popover-footer {
		padding: 8px 12px;
		border-top: 1px solid var(--border-default);
		flex-shrink: 0;
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
