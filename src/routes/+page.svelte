<script lang="ts">
	import {
		sortedSessions,
		expandedSessionId,
		currentConversation,
		statusSummary
	} from '$lib/stores/sessions';
	import { getConversation, sendPrompt, stopSession, openSession, approveSession } from '$lib/api';
	import StatusBar from '$lib/components/StatusBar.svelte';
	import SessionCard from '$lib/components/SessionCard.svelte';
	import ExpandedCardOverlay from '$lib/components/ExpandedCardOverlay.svelte';
	import type { Session } from '$lib/types';
	import { SessionStatus } from '$lib/types';

	let sessions = $derived($sortedSessions);
	let summary = $derived($statusSummary);
	let expandedId = $derived($expandedSessionId);
	let conversation = $derived($currentConversation);

	// Helper function to group sessions by project path, then by status
	function groupByProjectAndStatus(sessions: Session[]) {
		const groups: Array<{
			path: string;
			displayName: string;
			attention: Session[];
			idle: Session[];
			working: Session[];
			connecting: Session[];
			lastModified: number;
		}> = [];

		sessions.forEach(session => {
			let group = groups.find(g => g.path === session.projectPath);
			if (!group) {
				const parts = session.projectPath.split(/[/\\]/);
				const folderName = parts.filter(Boolean).pop() || session.projectPath;
				group = {
					path: session.projectPath,
					displayName: folderName,
					attention: [],
					idle: [],
					working: [],
					connecting: [],
					lastModified: 0
				};
				groups.push(group);
			}

			const modified = new Date(session.modified).getTime();
			if (modified > group.lastModified) {
				group.lastModified = modified;
			}

			if (session.status === SessionStatus.NeedsPermission) {
				group.attention.push(session);
			} else if (session.status === SessionStatus.WaitingForInput) {
				group.idle.push(session);
			} else if (session.status === SessionStatus.Working) {
				group.working.push(session);
			} else if (session.status === SessionStatus.Connecting) {
				group.connecting.push(session);
			}
		});

		// Sort groups: priority to those needing attention, then by modification time
		return groups.sort((a, b) => {
			const aNeedsAttention = a.attention.length > 0;
			const bNeedsAttention = b.attention.length > 0;
			if (aNeedsAttention !== bNeedsAttention) return aNeedsAttention ? -1 : 1;

			const aNeedsIdle = a.idle.length > 0;
			const bNeedsIdle = b.idle.length > 0;
			if (aNeedsIdle !== bNeedsIdle) return aNeedsIdle ? -1 : 1;

			return b.lastModified - a.lastModified;
		});
	}

	let projectGroups = $derived(groupByProjectAndStatus(sessions));

	let expandedSession = $derived(sessions.find((s) => s.id === expandedId) || null);

	$effect(() => {
		if (expandedId) {
			getConversation(expandedId)
				.then((conv) => {
					currentConversation.set(conv);
				})
				.catch((error) => {
					console.error('Failed to fetch conversation:', error);
					currentConversation.set(null);
				});
		} else {
			currentConversation.set(null);
		}
	});

	function handleExpand(session: Session) {
		expandedSessionId.set(session.id);
	}

	function handleClose() {
		expandedSessionId.set(null);
	}

	async function handleApprove(session: Session) {
		try {
			await approveSession(session.pid, session.projectPath);
		} catch (error) {
			console.error('Failed to approve:', error);
		}
	}

	async function handleSend(sessionId: string, prompt: string) {
		try {
			await sendPrompt(sessionId, prompt);
		} catch (error) {
			console.error('Failed to send prompt:', error);
		}
	}

	async function handleStop(pid: number) {
		try {
			await stopSession(pid);
		} catch (error) {
			console.error('Failed to stop session:', error);
		}
	}

	async function handleOpen(pid: number, projectPath: string) {
		try {
			await openSession(pid, projectPath);
		} catch (error) {
			console.error('Failed to open session:', error);
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key >= '1' && e.key <= '9' && !expandedId) {
			const index = parseInt(e.key) - 1;
			if (index < sessions.length) {
				handleExpand(sessions[index]);
			}
		}
		if (e.key === 'Tab' && !expandedId) {
			// Find first session needing attention across all projects
			const needsAction = sessions.filter(s =>
				s.status === SessionStatus.NeedsPermission ||
				s.status === SessionStatus.WaitingForInput
			);
			if (needsAction.length > 0) {
				e.preventDefault();
				handleExpand(needsAction[0]);
			}
		}
	}
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="dashboard">
	<div class="window-drag-handle" data-tauri-drag-region></div>

	<main class="grid-container">
		{#if sessions.length === 0}
			<div class="empty-state">
				<div class="system-status-container" style="width: 100%; margin-bottom: var(--space-3xl);">
					<StatusBar total={0} summary={{ working: 0, permission: 0, input: 0, connecting: 0 }} />
				</div>
				<div class="empty-visual">
					<div class="empty-orb">
						<div class="orb-core"></div>
						<div class="orb-ring ring-1"></div>
						<div class="orb-ring ring-2"></div>
						<div class="orb-ring ring-3"></div>
					</div>
				</div>
				<div class="empty-content">
					<h2>No Active Sessions</h2>
					<p>Start a Claude Code session in your terminal or IDE</p>
					<div class="empty-hint">
						<span class="hint-icon">
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
								<circle cx="12" cy="12" r="10" />
								<path d="M12 16v-4" />
								<path d="M12 8h.01" />
							</svg>
						</span>
						Sessions are detected automatically
					</div>
				</div>
			</div>
		{:else}
			<div class="sections-container">
				<section class="system-section">
					<div class="project-header">
						<span class="project-name">System status</span>
					</div>
					<div class="system-status-container">
						<StatusBar total={sessions.length} {summary} />
					</div>
				</section>

				{#each projectGroups as group (group.path)}
					<section class="project-section">
						<div class="project-header">
							<span class="project-name">{group.displayName}</span>
							<span class="project-count">
								{group.attention.length + group.idle.length + group.working.length + group.connecting.length}
							</span>
						</div>

						<div class="status-groups">
							{#if group.attention.length > 0}
								<div class="status-group">
									<div class="status-header">
										<span class="status-indicator attention"></span>
										<span class="status-title">Needs Attention</span>
										<span class="status-count">{group.attention.length}</span>
									</div>
									<div class="session-grid">
										{#each group.attention as session (session.id)}
											<SessionCard
												{session}
												onexpand={() => handleExpand(session)}
												onapprove={() => handleApprove(session)}
												onsend={(prompt) => handleSend(session.id, prompt)}
												onstop={() => handleStop(session.pid)}
												onopen={() => handleOpen(session.pid, session.projectPath)}
											/>
										{/each}
									</div>
								</div>
							{/if}

							{#if group.idle.length > 0}
								<div class="status-group">
									<div class="status-header">
										<span class="status-indicator idle"></span>
										<span class="status-title">Idle</span>
										<span class="status-count">{group.idle.length}</span>
									</div>
									<div class="session-grid">
										{#each group.idle as session (session.id)}
											<SessionCard
												{session}
												onexpand={() => handleExpand(session)}
												onapprove={() => handleApprove(session)}
												onsend={(prompt) => handleSend(session.id, prompt)}
												onstop={() => handleStop(session.pid)}
												onopen={() => handleOpen(session.pid, session.projectPath)}
											/>
										{/each}
									</div>
								</div>
							{/if}

							{#if group.working.length > 0}
								<div class="status-group">
									<div class="status-header">
										<span class="status-indicator working"></span>
										<span class="status-title">Working</span>
										<span class="status-count">{group.working.length}</span>
									</div>
									<div class="session-grid">
										{#each group.working as session (session.id)}
											<SessionCard
												{session}
												onexpand={() => handleExpand(session)}
												onapprove={() => handleApprove(session)}
												onsend={(prompt) => handleSend(session.id, prompt)}
												onstop={() => handleStop(session.pid)}
												onopen={() => handleOpen(session.pid, session.projectPath)}
											/>
										{/each}
									</div>
								</div>
							{/if}

							{#if group.connecting.length > 0}
								<div class="status-group">
									<div class="status-header">
										<span class="status-indicator connecting"></span>
										<span class="status-title">Connecting</span>
										<span class="status-count">{group.connecting.length}</span>
									</div>
									<div class="session-grid">
										{#each group.connecting as session (session.id)}
											<SessionCard
												{session}
												onexpand={() => handleExpand(session)}
												onapprove={() => handleApprove(session)}
												onsend={(prompt) => handleSend(session.id, prompt)}
												onstop={() => handleStop(session.pid)}
												onopen={() => handleOpen(session.pid, session.projectPath)}
											/>
										{/each}
									</div>
								</div>
							{/if}
						</div>
					</section>
				{/each}
			</div>
		{/if}
	</main>

	{#if expandedSession}
		<ExpandedCardOverlay
			session={expandedSession}
			{conversation}
			onclose={handleClose}
			onsend={(prompt) => handleSend(expandedSession.id, prompt)}
			onstop={() => handleStop(expandedSession.pid)}
			onopen={() => handleOpen(expandedSession.pid, expandedSession.projectPath)}
			onapprove={() => handleApprove(expandedSession)}
		/>
	{/if}

</div>

<style>
	.dashboard {
		display: flex;
		flex-direction: column;
		height: 100vh;
		width: 100vw;
		overflow: hidden;
		background: var(--bg-base);
	}

	.window-drag-handle {
		height: 28px;
		width: 100%;
		flex-shrink: 0;
		background: transparent;
		z-index: 1000;
	}

	.grid-container {
		flex: 1;
		overflow-y: auto;
		padding: var(--space-xl);
	}

	.sections-container {
		display: flex;
		flex-direction: column;
		gap: var(--space-3xl);
		max-width: 1200px;
		margin: 0 auto;
	}

	.project-section {
		display: flex;
		flex-direction: column;
		gap: var(--space-xl);
	}

	.project-header {
		display: flex;
		align-items: baseline;
		gap: var(--space-md);
		padding-bottom: var(--space-lg);
		border-bottom: 2px solid var(--border-default);
		margin-bottom: var(--space-sm);
	}

	.project-name {
		font-family: var(--font-pixel-grid);
		font-size: 18px;
		font-weight: 500;
		color: var(--text-primary);
		text-transform: uppercase;
		letter-spacing: 0.15em;
		line-height: 1;
	}

	.project-count {
		font-family: var(--font-pixel-grid);
		font-size: 18px;
		font-weight: 500;
		line-height: 1;
		color: var(--text-muted);
	}

	.status-groups {
		display: flex;
		flex-direction: column;
		gap: var(--space-xl);
	}

	.status-group {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
	}

	.status-header {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding-left: var(--space-lg);
	}

	.status-indicator {
		width: 6px;
		height: 6px;
	}

	.status-indicator.attention {
		background: var(--status-permission);
	}

	.status-indicator.idle {
		background: var(--status-input);
	}

	.status-indicator.working {
		background: var(--status-working);
	}

	.status-indicator.connecting {
		background: var(--status-connecting);
	}

	.status-title {
		font-family: var(--font-mono);
		font-size: 10px;
		font-weight: 500;
		color: var(--text-secondary);
		text-transform: uppercase;
		letter-spacing: 0.1em;
	}

	.status-count {
		font-family: var(--font-mono);
		font-size: 10px;
		color: var(--text-muted);
	}

	.session-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
		gap: var(--space-lg);
	}

	/* Empty State */
	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100%;
		text-align: center;
		gap: var(--space-3xl);
		max-width: 1200px;
		margin: 0 auto;
		padding: var(--space-3xl) 0;
	}

	.empty-visual {
		position: relative;
		width: 80px;
		height: 80px;
		border: 1px solid var(--border-default);
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.empty-orb {
		position: relative;
		width: 100%;
		height: 100%;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.orb-core {
		width: 8px;
		height: 8px;
		background: var(--text-muted);
		animation: pulse-glow 2s linear infinite;
	}

	.orb-ring {
		display: none;
	}

	.empty-content {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-md);
	}

	.empty-content h2 {
		font-family: var(--font-pixel-grid);
		font-size: 18px;
		font-weight: 500;
		color: var(--text-primary);
		text-transform: uppercase;
		letter-spacing: 0.15em;
	}

	.empty-content p {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-muted);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.empty-hint {
		display: inline-flex;
		align-items: center;
		gap: var(--space-sm);
		margin-top: var(--space-md);
		padding: var(--space-sm) var(--space-lg);
		border: 1px solid var(--border-default);
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--text-muted);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.hint-icon {
		display: flex;
		color: var(--text-muted);
	}
</style>
