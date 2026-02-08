<script lang="ts">
	import { slide } from 'svelte/transition';
	import { flip } from 'svelte/animate';
	import { quintOut } from 'svelte/easing';
	import { onMount } from 'svelte';
	import { browser } from '$app/environment';
	import {
		sortedSessions,
		expandedSessionId,
		currentConversation,
		statusSummary
	} from '$lib/stores/sessions';
	import { getConversation, stopSession, openSession } from '$lib/api';
	import { isDemoMode, toggleDemoMode } from '$lib/demo';
	import StatusBar from '$lib/components/StatusBar.svelte';
	import SessionCard from '$lib/components/SessionCard.svelte';
	import ExpandedCardOverlay from '$lib/components/ExpandedCardOverlay.svelte';
	import type { Session } from '$lib/types';
	import { SessionStatus } from '$lib/types';

	let demoActive = $derived($isDemoMode);

	let sessions = $derived($sortedSessions);
	let summary = $derived($statusSummary);
	let expandedId = $derived($expandedSessionId);
	let conversation = $derived($currentConversation);

	let viewMode = $state<'project' | 'all'>('project');

	onMount(() => {
		if (browser) {
			const saved = localStorage.getItem('sessionViewMode');
			if (saved === 'project' || saved === 'all') {
				viewMode = saved;
			}
		}
	});

	$effect(() => {
		if (browser) {
			localStorage.setItem('sessionViewMode', viewMode);
		}
	});

	let isCompact = $state(false);

	onMount(() => {
		if (browser) {
			const saved = localStorage.getItem('sessionViewMode');
			if (saved === 'project' || saved === 'all') {
				viewMode = saved;
			}
			const savedCompact = localStorage.getItem('sessionViewCompact');
			if (savedCompact === 'true') {
				isCompact = true;
			}
		}
	});

	$effect(() => {
		if (browser) {
			localStorage.setItem('sessionViewCompact', String(isCompact));
		}
	});

	// Helper function to group sessions by project path, then by status
	function groupByProjectAndStatus(sessions: Session[]) {
		const groups: Array<{
			path: string;
			displayName: string;
			attention: Session[];
			idle: Session[];
			working: Session[];
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
					lastModified: 0
				};
				groups.push(group);
			}

			addToGroup(group, session);
		});

		return sortGroups(groups);
	}

	function groupSessionsByStatus(sessions: Session[]) {
		const groups = {
			attention: [] as Session[],
			idle: [] as Session[],
			working: [] as Session[]
		};

		sessions.forEach(session => {
			if (session.status === SessionStatus.NeedsPermission) {
				groups.attention.push(session);
			} else if (session.status === SessionStatus.WaitingForInput) {
				groups.idle.push(session);
			} else if (session.status === SessionStatus.Working || session.status === SessionStatus.Connecting) {
				groups.working.push(session);
			}
		});

		const sortSessions = (a: Session, b: Session) => new Date(a.modified).getTime() - new Date(b.modified).getTime();

		return [
			{ id: 'attention', label: 'Needs Attention', sessions: groups.attention.sort(sortSessions), type: 'attention' },
			{ id: 'idle', label: 'Idle', sessions: groups.idle.sort(sortSessions), type: 'idle' },
			{ id: 'working', label: 'Working', sessions: groups.working.sort(sortSessions), type: 'working' }
		].filter(g => g.sessions.length > 0);
	}

	function addToGroup(group: any, session: Session) {
		const modified = new Date(session.modified).getTime();
		if (modified > group.lastModified) {
			group.lastModified = modified;
		}

		if (session.status === SessionStatus.NeedsPermission) {
			group.attention.push(session);
		} else if (session.status === SessionStatus.WaitingForInput) {
			group.idle.push(session);
		} else if (session.status === SessionStatus.Working || session.status === SessionStatus.Connecting) {
			group.working.push(session);
		}
	}

	function sortGroups(groups: any[]) {
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
	let allStatusGroups = $derived(groupSessionsByStatus(sessions));

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
		if (e.key === 'd' && (e.metaKey || e.ctrlKey)) {
			e.preventDefault();
			toggleDemoMode();
			return;
		}
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
		<div class="sections-container">
			<section class="system-section">
				<div class="project-header">
					<span class="project-name">System status</span>
					<span class="project-count">{sessions.length}</span>
					<button
						class="toggle-btn demo-toggle"
						class:active={demoActive}
						onclick={() => toggleDemoMode()}
						title="Try with Sample Data (Cmd+D)"
					>
						<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
							<path d="M10 2v7.31" />
							<path d="M14 2v7.31" />
							<path d="M8.5 2h7" />
							<path d="M14 9.3c.7.4 1.3.9 1.8 1.5l3.8 4.4a3 3 0 0 1-2.3 4.8H6.7a3 3 0 0 1-2.3-4.8l3.8-4.4c.5-.6 1.1-1.1 1.8-1.5" />
						</svg>
						<span class="demo-label">DEMO</span>
					</button>
					<div class="header-spacer"></div>
					<div class="view-toggle">
						<button
							class="toggle-btn"
							class:active={isCompact}
							onclick={() => isCompact = !isCompact}
							title="Compact View"
						>
							<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
								<polyline points="4 14 10 14 10 20" />
								<polyline points="20 10 14 10 14 4" />
								<line x1="14" y1="10" x2="21" y2="3" />
								<line x1="3" y1="21" x2="10" y2="14" />
							</svg>
						</button>
					</div>
					<div class="view-toggle">
						<button 
							class="toggle-btn" 
							class:active={viewMode === 'project'} 
							onclick={() => viewMode = 'project'}
							title="Group by Project"
						>
							<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
								<line x1="8" y1="6" x2="21" y2="6"></line>
								<line x1="8" y1="12" x2="21" y2="12"></line>
								<line x1="8" y1="18" x2="21" y2="18"></line>
								<line x1="3" y1="6" x2="3.01" y2="6"></line>
								<line x1="3" y1="12" x2="3.01" y2="12"></line>
								<line x1="3" y1="18" x2="3.01" y2="18"></line>
							</svg>
						</button>
						<button 
							class="toggle-btn" 
							class:active={viewMode === 'all'} 
							onclick={() => viewMode = 'all'}
							title="Show All"
						>
							<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
								<rect x="3" y="3" width="7" height="7" rx="1" />
								<rect x="14" y="3" width="7" height="7" rx="1" />
								<rect x="14" y="14" width="7" height="7" rx="1" />
								<rect x="3" y="14" width="7" height="7" rx="1" />
							</svg>
						</button>
					</div>
				</div>
				
				{#if sessions.length > 0}
					<div class="system-status-container">
						<StatusBar total={sessions.length} {summary} />
					</div>
				{/if}
			</section>

			{#if sessions.length === 0}
				<div class="empty-state">
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

				{#if viewMode === 'project'}
					{#each projectGroups as group (group.path)}
						<section class="project-section" animate:flip={{ duration: 400 }}>
							<div class="project-header">
								<span class="project-name">{group.displayName}</span>
								<span class="project-count">
									{group.attention.length + group.idle.length + group.working.length}
								</span>
							</div>

							<div class="status-groups">
								<div class="status-group" class:empty={group.attention.length === 0} class:compact={isCompact}>
									<div class="status-header attention">
										<span class="status-indicator attention"></span>
										<span class="status-title">Needs Attention</span>
										<span class="status-count">{group.attention.length}</span>
									</div>
									<div class="session-grid">
										{#each group.attention as session (session.id)}
											<div
												class="card-wrapper"
												transition:slide={{ duration: 400, easing: quintOut }}
												animate:flip={{ duration: 400 }}
											>
												<SessionCard
													{session}
													compact={isCompact}
													onexpand={() => handleExpand(session)}
													onstop={() => handleStop(session.pid)}
													onopen={() => handleOpen(session.pid, session.projectPath)}
												/>
											</div>
										{/each}
									</div>
								</div>

								<div class="status-group" class:empty={group.idle.length === 0} class:compact={isCompact}>
									<div class="status-header idle">
										<span class="status-indicator idle"></span>
										<span class="status-title">Idle</span>
										<span class="status-count">{group.idle.length}</span>
									</div>
									<div class="session-grid">
										{#each group.idle as session (session.id)}
											<div
												class="card-wrapper"
												transition:slide={{ duration: 400, easing: quintOut }}
												animate:flip={{ duration: 400 }}
											>
												<SessionCard
													{session}
													compact={isCompact}
													onexpand={() => handleExpand(session)}
													onstop={() => handleStop(session.pid)}
													onopen={() => handleOpen(session.pid, session.projectPath)}
												/>
											</div>
										{/each}
									</div>
								</div>

								<div class="status-group" class:empty={group.working.length === 0} class:compact={isCompact}>
									<div class="status-header working">
										<span class="status-indicator working"></span>
										<span class="status-title">Working</span>
										<span class="status-count">{group.working.length}</span>
									</div>
									<div class="session-grid">
										{#each group.working as session (session.id)}
											<div
												class="card-wrapper"
												transition:slide={{ duration: 400, easing: quintOut }}
												animate:flip={{ duration: 400 }}
											>
												<SessionCard
													{session}
													compact={isCompact}
													onexpand={() => handleExpand(session)}
													onstop={() => handleStop(session.pid)}
													onopen={() => handleOpen(session.pid, session.projectPath)}
												/>
											</div>
										{/each}
									</div>
								</div>

							</div>
						</section>
					{/each}
				{:else}
					{#each allStatusGroups as group (group.id)}
						<section class="project-section" animate:flip={{ duration: 400 }}>
							<div class="status-header all-view {group.type}">
								<span class="status-indicator {group.type}" style="width: 8px; height: 8px;"></span>
								<span class="project-name" style="font-size: 16px;">{group.label}</span>
								<span class="project-count">{group.sessions.length}</span>
							</div>

							<div class="all-sessions-grid" class:compact={isCompact}>
								{#each group.sessions as session (session.id)}
									<div
										class="card-wrapper"
										transition:slide={{ duration: 400, easing: quintOut }}
										animate:flip={{ duration: 400 }}
									>
										<SessionCard
											{session}
											compact={isCompact}
											onexpand={() => handleExpand(session)}
											onstop={() => handleStop(session.pid)}
											onopen={() => handleOpen(session.pid, session.projectPath)}
										/>
									</div>
								{/each}
							</div>
						</section>
					{/each}
				{/if}
			{/if}
		</div>
	</main>

	{#if expandedSession}
		<ExpandedCardOverlay
			session={expandedSession}
			{conversation}
			onclose={handleClose}
			onstop={() => handleStop(expandedSession.pid)}
			onopen={() => handleOpen(expandedSession.pid, expandedSession.projectPath)}
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
		margin: 0 auto;
		width: 100%;
	}

	.project-section {
		display: flex;
		flex-direction: column;
		gap: var(--space-xl);
	}

	.project-header {
		display: flex;
		align-items: center;
		gap: var(--space-md);
		padding-bottom: var(--space-md);
		border-bottom: 1px solid var(--text-primary);
		margin-bottom: var(--space-md);
	}

	.project-name {
		font-family: var(--font-pixel);
		font-size: 22px;
		font-weight: 600;
		color: var(--text-primary);
		text-transform: uppercase;
		letter-spacing: 0.1em;
		line-height: 1;
	}

	.project-count {
		font-family: var(--font-pixel);
		font-size: 18px;
		font-weight: 500;
		line-height: 1;
		color: var(--text-secondary);
	}

	.status-groups {
		display: flex;
		flex-direction: row;
		gap: var(--space-xl);
		overflow-x: auto;
		padding-bottom: var(--space-lg);
	}

	.status-group {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
		min-width: 350px;
		max-width: 400px;
		flex: 1;
	}

	.status-header {
		display: flex;
		align-items: center;
		padding: var(--space-sm) var(--space-md);
		background: rgba(255, 255, 255, 0.03);
		border-left: 3px solid var(--border-default);
		gap: var(--space-sm);
	}

	.status-header.attention { border-left-color: var(--status-permission); }
	.status-header.idle { border-left-color: var(--status-input); }
	.status-header.working { border-left-color: var(--status-working); }
	.status-header.connecting { border-left-color: var(--status-connecting); }

	.status-header.all-view {
		background: transparent;
		padding-left: 0;
		margin-bottom: var(--space-md);
		border-left: none;
	}

	.status-group.empty {
		opacity: 0.5;
	}

	.status-group.empty .status-header {
		background: transparent;
		border-left-style: dashed;
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
		font-size: 12px;
		font-weight: 500;
		color: var(--text-secondary);
		text-transform: uppercase;
		letter-spacing: 0.1em;
	}

	.status-count {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-muted);
	}

	.session-grid {
		display: flex;
		flex-direction: column;
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
		font-size: 14px;
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
		font-size: 13px;
		color: var(--text-muted);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.hint-icon {
		display: flex;
		color: var(--text-muted);
	}



	.header-spacer {
		flex: 1;
	}

	.view-toggle {
		display: flex;
		gap: var(--space-xs);
		background: rgba(255, 255, 255, 0.03);
		padding: 2px;
		border: 1px solid var(--border-default);
	}

	.toggle-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border: 1px solid transparent;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		transition: all 0.2s ease;
	}

	.toggle-btn:hover {
		color: var(--text-secondary);
		background: rgba(255, 255, 255, 0.05);
	}

	.toggle-btn.active {
		color: var(--text-primary);
		background: rgba(255, 255, 255, 0.1);
		border-color: var(--border-default);
	}

	.all-sessions-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
		gap: var(--space-lg);
	}

	.all-sessions-grid.compact {
		grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
		gap: var(--space-md);
	}

	.toggle-divider {
		width: 1px;
		background: var(--border-default);
		margin: 2px 4px;
	}

	.demo-toggle {
		width: auto;
		padding: 0 var(--space-sm);
		gap: var(--space-xs);
		color: var(--accent-amber);
	}

	.demo-toggle:hover {
		color: var(--accent-amber);
		background: rgba(255, 102, 0, 0.1);
	}

	.demo-toggle.active {
		background: var(--accent-amber);
		color: #000;
		border-color: var(--accent-amber);
	}

	.demo-label {
		font-family: var(--font-pixel);
		font-size: 10px;
		font-weight: 700;
		letter-spacing: 0.05em;
	}

	.demo-badge {
		font-family: var(--font-mono);
		font-size: 10px;
		font-weight: 700;
		color: var(--accent-amber);
		background: rgba(255, 102, 0, 0.12);
		padding: 2px 6px;
		border: 1px solid var(--accent-amber);
		text-transform: uppercase;
		letter-spacing: 0.1em;
		line-height: 1;
	}

	.empty-header-row {
		display: flex;
		align-items: center;
		gap: var(--space-md);
		width: 100%;
	}

	.empty-header-row :global(.status-bar) {
		flex: 1;
	}
</style>
