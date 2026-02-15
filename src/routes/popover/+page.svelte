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

	// Initialize data for this window context
	let unlistenFocus: (() => void) | null = null;

	onMount(() => {
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

			// Refresh when window becomes visible (skip in demo mode)
			unlistenFocus = await listen('tauri://focus', async () => {
				if (get(isDemoMode)) return;
				try {
					const freshSessions = await getSessions();
					sessionsStore.set(freshSessions);
				} catch (error) {
					console.error('Failed to refresh sessions:', error);
				}
			});
		};

		init();

		return () => {
			if (unlistenFocus) unlistenFocus();
		};
	});

	// Get top priority sessions (max 5)
	let prioritySessions = $derived(
		sessions
			.filter(s =>
				s.status === SessionStatus.NeedsPermission ||
				s.status === SessionStatus.WaitingForInput
			)
			.slice(0, 5)
	);

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

	function getStatusColor(status: SessionStatus): string {
		switch (status) {
			case SessionStatus.NeedsPermission:
				return 'var(--status-permission)';
			case SessionStatus.WaitingForInput:
				return 'var(--status-input)';
			case SessionStatus.Working:
				return 'var(--status-working)';
			default:
				return 'var(--status-working)';
		}
	}

	function getStatusLabel(status: SessionStatus): string {
		switch (status) {
			case SessionStatus.NeedsPermission:
				return 'Permission';
			case SessionStatus.WaitingForInput:
				return 'Idle';
			case SessionStatus.Working:
				return 'Working';
			default:
				return 'Working';
		}
	}
</script>

<div class="popover">
	<header class="popover-header">
		<div class="header-title">Claude Sessions</div>
		<div class="header-summary">
			{#if summary.permission > 0}
				<span class="badge permission">{summary.permission}</span>
			{/if}
			{#if summary.input > 0}
				<span class="badge idle">{summary.input}</span>
			{/if}
			{#if summary.working > 0}
				<span class="badge working">{summary.working}</span>
			{/if}
		</div>
	</header>

	<main class="popover-content">
		{#if sessions.length === 0}
			<div class="empty-state">
				<div class="empty-icon">
					<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
						<circle cx="12" cy="12" r="10" />
						<path d="M12 6v6l4 2" />
					</svg>
				</div>
				<p>No active sessions</p>
			</div>
		{:else if prioritySessions.length > 0}
			<div class="session-list">
				{#each prioritySessions as session (session.id)}
					<div class="session-item" style="--status-color: {getStatusColor(session.status)}">
						<div class="session-info">
							<div class="session-project">{session.sessionName}</div>
							<div class="session-prompt">{session.firstPrompt}</div>
							<div class="session-meta">
								<span class="status-badge">{getStatusLabel(session.status)}</span>
								{#if session.gitBranch}
									<span class="branch">{session.gitBranch}</span>
								{/if}
							</div>
						</div>
						<div class="session-actions">
							<button class="action-btn open" onclick={() => handleOpen(session)} title="Open">
								<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
									<path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
									<polyline points="15 3 21 3 21 9" />
									<line x1="10" y1="14" x2="21" y2="3" />
								</svg>
							</button>
						</div>
					</div>
				{/each}
			</div>
		{:else}
			<div class="all-working">
				<div class="working-indicator"></div>
				<p>{summary.working} session{summary.working !== 1 ? 's' : ''} working</p>
			</div>
		{/if}
	</main>

	<footer class="popover-footer">
		<button class="open-app-btn" onclick={openMainWindow}>
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
		border-radius: 8px;
		overflow: hidden;
		font-family: var(--font-mono);
	}

	.popover-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-md) var(--space-lg);
		border-bottom: 1px solid var(--border-default);
		background: var(--bg-elevated);
	}

	.header-title {
		font-family: var(--font-pixel);
		font-size: 11px;
		font-weight: 500;
		color: var(--text-primary);
		text-transform: uppercase;
		letter-spacing: 0.1em;
	}

	.header-summary {
		display: flex;
		gap: var(--space-xs);
	}

	.badge {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		min-width: 18px;
		height: 18px;
		padding: 0 var(--space-xs);
		border-radius: 9px;
		font-size: 10px;
		font-weight: 600;
		color: var(--bg-base);
	}

	.badge.permission {
		background: var(--status-permission);
	}

	.badge.idle {
		background: var(--status-input);
	}

	.badge.working {
		background: var(--status-working);
	}

	.popover-content {
		flex: 1;
		overflow-y: auto;
		padding: var(--space-sm);
	}

	.empty-state,
	.all-working {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100%;
		gap: var(--space-md);
		color: var(--text-muted);
	}

	.empty-icon {
		opacity: 0.5;
	}

	.empty-state p,
	.all-working p {
		font-size: 11px;
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.working-indicator {
		width: 12px;
		height: 12px;
		background: var(--status-working);
		border-radius: 50%;
		animation: pulse 1.5s ease-in-out infinite;
	}

	@keyframes pulse {
		0%, 100% { opacity: 1; transform: scale(1); }
		50% { opacity: 0.6; transform: scale(0.9); }
	}

	.session-list {
		display: flex;
		flex-direction: column;
		gap: var(--space-xs);
	}

	.session-item {
		display: flex;
		align-items: center;
		gap: var(--space-md);
		padding: var(--space-sm) var(--space-md);
		background: var(--bg-elevated);
		border: 1px solid var(--border-default);
		border-left: 3px solid var(--status-color);
		border-radius: 4px;
		transition: background 0.15s ease;
	}

	.session-item:hover {
		background: var(--bg-hover);
	}

	.session-info {
		flex: 1;
		min-width: 0;
	}

	.session-project {
		font-size: 11px;
		font-weight: 600;
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.session-prompt {
		font-size: 10px;
		color: var(--text-secondary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		margin-top: 2px;
	}

	.session-meta {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		margin-top: var(--space-xs);
	}

	.status-badge {
		font-size: 9px;
		color: var(--status-color);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.branch {
		font-size: 9px;
		color: var(--text-muted);
	}

	.session-actions {
		display: flex;
		gap: var(--space-xs);
	}

	.action-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border: 1px solid var(--border-default);
		border-radius: 4px;
		background: transparent;
		color: var(--text-secondary);
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.action-btn:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
	}

	.popover-footer {
		padding: var(--space-sm) var(--space-lg);
		border-top: 1px solid var(--border-default);
		background: var(--bg-elevated);
	}

	.open-app-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: var(--space-sm);
		width: 100%;
		padding: var(--space-sm) var(--space-md);
		border: 1px solid var(--border-default);
		border-radius: 4px;
		background: transparent;
		color: var(--text-secondary);
		font-family: var(--font-mono);
		font-size: 10px;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.open-app-btn:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
		border-color: var(--text-muted);
	}
</style>
