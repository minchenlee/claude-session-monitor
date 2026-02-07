<script lang="ts">
	import { onMount, tick } from 'svelte';
	import type { Session, Conversation } from '$lib/types';
	import { SessionStatus } from '$lib/types';
	import { getSessionUIState, updateSessionUIState } from '$lib/stores/sessions';
	import MessageBubble from './MessageBubble.svelte';
	import MessageNavMap from './MessageNavMap.svelte';

	interface Props {
		session: Session;
		conversation: Conversation | null;
		onclose?: () => void;
		onsend?: (prompt: string) => void;
		onstop?: () => void;
		onopen?: () => void;
		onapprove?: () => void;
	}

	let { session, conversation, onclose, onsend, onstop, onopen, onapprove }: Props = $props();

	let uiState = $derived(getSessionUIState(session.id));
	let promptInput = $state('');
	let messagesContainer: HTMLDivElement;
	let isInitialLoad = $state(true);
	let hasScrolledToBottom = $state(false);

	onMount(() => {
		isInitialLoad = false;

		const handleKeydown = (e: KeyboardEvent) => {
			if (e.key === 'Escape') {
				handleClose();
			}
		};
		window.addEventListener('keydown', handleKeydown);
		return () => window.removeEventListener('keydown', handleKeydown);
	});

	function handleScroll() {
		// No longer persisting scroll position as we want to always show latest on open
	}

	$effect(() => {
		if (conversation && conversation.messages.length > 0 && messagesContainer) {
			if (!hasScrolledToBottom) {
				// Initial scroll to bottom when opening
				tick().then(() => {
					messagesContainer.scrollTop = messagesContainer.scrollHeight;
					hasScrolledToBottom = true;
				});
			} else {
				// Auto-scroll logic for new messages
				const isAtBottom =
					messagesContainer.scrollHeight - messagesContainer.scrollTop - messagesContainer.clientHeight < 150;
				if (isAtBottom) {
					tick().then(() => {
						messagesContainer.scrollTop = messagesContainer.scrollHeight;
					});
				}
			}
		}
	});

	$effect(() => {
		if (promptInput !== uiState.draftPrompt) {
			updateSessionUIState(session.id, { draftPrompt: promptInput });
		}
	});

	let isPermission = $derived(session.status === SessionStatus.NeedsPermission);
	let isWaitingInput = $derived(session.status === SessionStatus.WaitingForInput);
	let isWorking = $derived(session.status === SessionStatus.Working);

	function getStatusColor(): string {
		switch (session.status) {
			case SessionStatus.Working:
				return 'var(--status-working)';
			case SessionStatus.NeedsPermission:
				return 'var(--status-permission)';
			case SessionStatus.WaitingForInput:
				return 'var(--status-input)';
			case SessionStatus.Connecting:
				return 'var(--status-connecting)';
			default:
				return 'var(--status-connecting)';
		}
	}

	function getStatusLabel(): string {
		switch (session.status) {
			case SessionStatus.Working:
				return 'Working';
			case SessionStatus.NeedsPermission:
				return 'Approval Required';
			case SessionStatus.WaitingForInput:
				return 'Ready';
			case SessionStatus.Connecting:
				return 'Connecting';
			default:
				return 'Unknown';
		}
	}

	function handleClose() {
		onclose?.();
	}

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) {
			handleClose();
		}
	}

	function handleSend() {
		if (promptInput.trim()) {
			onsend?.(promptInput.trim());
			promptInput = '';
			updateSessionUIState(session.id, { draftPrompt: '' });
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
			handleSend();
		}
	}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div class="overlay-backdrop" onclick={handleBackdropClick} role="dialog" aria-modal="true" aria-labelledby="overlay-title" tabindex="-1">
		<div class="overlay-layout">
			<div class="overlay-card" class:permission={isPermission} class:waiting={isWaitingInput}>
				<!-- Header -->
				<header class="overlay-header" data-tauri-drag-region>
					<div class="header-left" data-tauri-drag-region>
						<div class="status-indicator" style="--status-color: {getStatusColor()}">
							<div class="status-dot"></div>
							{#if isPermission || isWaitingInput}
								<div class="status-ring"></div>
							{/if}
						</div>
						<div class="header-info">
							<div class="header-title">
								<h2 id="overlay-title" class="project-name">{session.projectName}</h2>
								{#if session.gitBranch}
									<span class="git-branch">
										<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
											<line x1="6" y1="3" x2="6" y2="15" />
											<circle cx="18" cy="6" r="3" />
											<circle cx="6" cy="18" r="3" />
											<path d="M18 9a9 9 0 0 1-9 9" />
										</svg>
										{session.gitBranch}
									</span>
								{/if}
							</div>
							<div class="header-meta">
								<span class="status-label" style="color: {getStatusColor()}">{getStatusLabel()}</span>
								<span class="separator">·</span>
								<span class="message-count">{conversation?.messages.length ?? 0} messages</span>
							</div>
						</div>
					</div>
					<div class="header-actions">
						<button type="button" class="header-button" onclick={() => onstop?.()} title="Stop Session">
							<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
								<rect x="6" y="6" width="12" height="12" rx="1" />
							</svg>
						</button>
						<button type="button" class="header-button" onclick={() => onopen?.()} title="Open in IDE">
							<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
								<path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
								<polyline points="15 3 21 3 21 9" />
								<line x1="10" y1="14" x2="21" y2="3" />
							</svg>
						</button>
						<div class="header-divider"></div>
						<button type="button" class="close-button" onclick={handleClose} aria-label="Close">
							<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
								<line x1="18" y1="6" x2="6" y2="18" />
								<line x1="6" y1="6" x2="18" y2="18" />
							</svg>
						</button>
					</div>
				</header>

				<!-- Conversation Area -->
				<div class="conversation-area" bind:this={messagesContainer} onscroll={handleScroll}>
					{#if !conversation}
						<div class="loading-state">
							<div class="loading-spinner"></div>
							<p>Loading conversation...</p>
						</div>
					{:else if conversation.messages.length === 0}
						<div class="empty-state">
							<div class="empty-icon">
								<svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
									<path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
								</svg>
							</div>
							<p>No messages yet</p>
							<p class="empty-hint">Send a message to start the conversation</p>
						</div>
					{:else}
						<div class="messages">
							{#each conversation.messages as message, index (index)}
								<MessageBubble {message} />
							{/each}
						</div>
					{/if}
				</div>

				<!-- Input Area -->
				<footer class="input-area">
					{#if isPermission}
						<button type="button" class="approve-button" onclick={() => onapprove?.()}>
							<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
								<polyline points="20 6 9 17 4 12" />
							</svg>
							Approve Permission Request
						</button>
					{:else}
						<div class="input-wrapper">
							<textarea
								class="prompt-textarea"
								placeholder={isWaitingInput ? 'Type your message...' : 'Session is working...'}
								bind:value={promptInput}
								onkeydown={handleKeydown}
								rows="1"
								disabled={!isWaitingInput}
							></textarea>
							<button
								type="button"
								class="send-button"
								onclick={handleSend}
								disabled={!promptInput.trim() || !isWaitingInput}
								aria-label="Send message"
							>
								<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
									<line x1="22" y1="2" x2="11" y2="13" />
									<polygon points="22 2 15 22 11 13 2 9 22 2" />
								</svg>
							</button>
						</div>
					{/if}
				</footer>
			</div>

			<div class="nav-map-side">
				<MessageNavMap {conversation} scrollContainer={messagesContainer} />
			</div>
		</div>
</div>

<style>
	.overlay-backdrop {
		position: fixed;
		inset: 0;
		background: var(--bg-overlay);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
		padding: var(--space-2xl);
	}

	.overlay-layout {
		display: flex;
		align-items: flex-start;
		gap: var(--space-xl);
		width: 100%;
		max-width: 1100px;
		height: 85vh;
		max-height: 900px;
		pointer-events: none; /* Allow clicks through empty layout area */
	}

	.overlay-card {
		position: relative;
		flex: 1; /* Take up remaining space */
		height: 100%;
		background: var(--bg-card);
		border: 1px solid var(--border-default);
		display: flex;
		flex-direction: column;
		overflow: hidden;
		pointer-events: auto; /* Enable clicks on the card */
	}

	.nav-map-side {
		flex-shrink: 0;
		height: 100%;
		display: flex;
		flex-direction: column;
		pointer-events: auto; /* Enable clicks on the nav map */
	}

	/* Header */
	.overlay-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: var(--space-lg) var(--space-xl);
		border-bottom: 1px solid var(--border-default);
	}

	.header-left {
		display: flex;
		align-items: center;
		gap: var(--space-md);
	}

	.status-indicator {
		position: relative;
		width: 8px;
		height: 8px;
		flex-shrink: 0;
	}

	.status-dot {
		width: 100%;
		height: 100%;
		background: var(--status-color);
	}

	.status-ring {
		display: none;
	}

	.header-info {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.header-title {
		display: flex;
		align-items: center;
		gap: var(--space-md);
	}

	.project-name {
		font-family: var(--font-pixel);
		font-size: 14px;
		font-weight: 500;
		color: var(--text-primary);
		margin: 0;
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.git-branch {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		font-family: var(--font-mono);
		font-size: 10px;
		color: var(--text-muted);
		text-transform: lowercase;
	}

	.header-meta {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		font-family: var(--font-mono);
		font-size: 10px;
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.status-label {
		font-weight: 500;
	}

	.separator {
		color: var(--text-muted);
	}

	.message-count {
		color: var(--text-muted);
	}

	.header-actions {
		display: flex;
		align-items: center;
		gap: var(--space-xs);
	}

	.header-button {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		color: var(--text-muted);
		transition: color var(--transition-fast);
	}

	.header-button:hover {
		color: var(--text-primary);
	}

	.header-divider {
		width: 1px;
		height: 16px;
		background: var(--border-default);
		margin: 0 var(--space-sm);
	}

	.close-button {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		color: var(--text-muted);
		transition: color var(--transition-fast);
	}

	.close-button:hover {
		color: var(--accent-red);
	}

	.conversation-area {
		flex: 1;
		overflow-y: auto;
		padding: var(--space-xl);
	}

	.messages {
		display: flex;
		flex-direction: column;
	}

	.loading-state,
	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100%;
		gap: var(--space-md);
		color: var(--text-muted);
	}

	.loading-spinner {
		width: 24px;
		height: 24px;
		border: 2px solid var(--border-default);
		border-top-color: var(--text-primary);
		animation: spin 1s linear infinite;
	}

	.empty-icon {
		opacity: 0.3;
		margin-bottom: var(--space-sm);
	}

	.empty-hint {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--text-muted);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	/* Input Area */
	.input-area {
		padding: var(--space-lg) var(--space-xl);
		border-top: 1px solid var(--border-default);
	}

	.input-wrapper {
		display: flex;
		gap: var(--space-sm);
		align-items: flex-end;
	}

	.prompt-textarea {
		flex: 1;
		padding: var(--space-md);
		background: var(--bg-card);
		border: 1px solid var(--border-default);
		color: var(--text-primary);
		font-family: var(--font-mono);
		font-size: 13px;
		resize: none;
		min-height: 44px;
		max-height: 140px;
		transition: border-color var(--transition-fast);
	}

	.prompt-textarea:focus {
		outline: none;
		border-color: var(--text-primary);
	}

	.prompt-textarea:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.prompt-textarea::placeholder {
		color: var(--text-muted);
	}

	.send-button {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 44px;
		height: 44px;
		background: var(--text-primary);
		color: var(--bg-base);
		border: 1px solid var(--text-primary);
		transition: all var(--transition-fast);
		flex-shrink: 0;
	}

	.send-button:hover:not(:disabled) {
		background: transparent;
		color: var(--text-primary);
	}

	.send-button:disabled {
		opacity: 0.3;
		cursor: not-allowed;
	}

	.approve-button {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: var(--space-sm);
		width: 100%;
		padding: var(--space-md) var(--space-xl);
		background: var(--status-permission);
		color: var(--bg-base);
		font-family: var(--font-mono);
		font-size: 12px;
		font-weight: 500;
		text-transform: uppercase;
		letter-spacing: 0.1em;
		border: 1px solid var(--status-permission);
		transition: all var(--transition-fast);
	}

	.approve-button:hover {
		background: transparent;
		color: var(--status-permission);
	}
</style>
