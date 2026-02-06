<script lang="ts">
	import { selectedSessionId, sessions, currentConversation } from '$lib/stores/sessions';
	import { getConversation, sendPrompt, stopSession, openSession } from '$lib/api';
	import SessionList from '$lib/components/SessionList.svelte';
	import ConversationView from '$lib/components/ConversationView.svelte';
	import PromptInput from '$lib/components/PromptInput.svelte';

	let currentSessionId = $derived($selectedSessionId);
	let currentSessions = $derived($sessions);

	// Get selected session details
	let selectedSession = $derived(
		currentSessions.find((s) => s.id === currentSessionId) || null
	);

	// Fetch conversation when a session is selected
	$effect(() => {
		if (currentSessionId) {
			getConversation(currentSessionId)
				.then((conversation) => {
					currentConversation.set(conversation);
				})
				.catch((error) => {
					console.error('Failed to fetch conversation:', error);
					currentConversation.set(null);
				});
		} else {
			currentConversation.set(null);
		}
	});

	async function handleSendPrompt(prompt: string) {
		if (!currentSessionId) return;

		try {
			await sendPrompt(currentSessionId, prompt);
		} catch (error) {
			console.error('Failed to send prompt:', error);
		}
	}

	async function handleStop() {
		if (!currentSessionId) return;

		try {
			await stopSession(currentSessionId);
		} catch (error) {
			console.error('Failed to stop session:', error);
		}
	}

	async function handleOpen() {
		if (!currentSessionId) return;

		try {
			await openSession(currentSessionId);
		} catch (error) {
			console.error('Failed to open session:', error);
		}
	}
</script>

<div class="app-container">
	<!-- Left Panel: Session List -->
	<aside class="sidebar">
		<SessionList />
	</aside>

	<!-- Right Panel: Conversation and Input -->
	<main class="main-panel">
		<!-- Conversation Header -->
		{#if selectedSession}
			<div class="conversation-header">
				<div class="project-info">
					<h1 class="project-name">
						{selectedSession.projectName}
						{#if selectedSession.gitBranch}
							<span class="git-branch">({selectedSession.gitBranch})</span>
						{/if}
					</h1>
					<p class="project-path">{selectedSession.projectPath}</p>
				</div>
			</div>
		{/if}

		<!-- Conversation View -->
		<div class="conversation-container">
			<ConversationView />
		</div>

		<!-- Bottom Action Bar -->
		<div class="action-bar">
			<div class="action-buttons">
				<button
					class="action-button stop-button"
					onclick={handleStop}
					disabled={!currentSessionId}
					type="button"
				>
					<svg
						width="16"
						height="16"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
						stroke-linecap="round"
						stroke-linejoin="round"
					>
						<rect x="6" y="6" width="12" height="12"></rect>
					</svg>
					Stop
				</button>
				<button
					class="action-button open-button"
					onclick={handleOpen}
					disabled={!currentSessionId}
					type="button"
				>
					<svg
						width="16"
						height="16"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
						stroke-linecap="round"
						stroke-linejoin="round"
					>
						<path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path>
						<polyline points="15 3 21 3 21 9"></polyline>
						<line x1="10" y1="14" x2="21" y2="3"></line>
					</svg>
					Open
				</button>
			</div>

			<div class="prompt-container">
				<PromptInput onsend={handleSendPrompt} />
			</div>
		</div>
	</main>
</div>

<style>
	.app-container {
		display: flex;
		height: 100vh;
		width: 100vw;
		overflow: hidden;
		background: #fafafa;
	}

	.sidebar {
		width: 300px;
		min-width: 250px;
		max-width: 400px;
		border-right: 1px solid #e5e7eb;
		background: #fafafa;
		display: flex;
		flex-direction: column;
	}

	.main-panel {
		flex: 1;
		display: flex;
		flex-direction: column;
		min-width: 0;
		background: white;
	}

	.conversation-header {
		padding: 20px 24px;
		border-bottom: 1px solid #e5e7eb;
		background: white;
	}

	.project-info {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.project-name {
		margin: 0;
		font-size: 20px;
		font-weight: 600;
		color: #111827;
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.git-branch {
		font-size: 14px;
		font-weight: 500;
		color: #6b7280;
	}

	.project-path {
		margin: 0;
		font-size: 13px;
		color: #6b7280;
		font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New',
			monospace;
	}

	.conversation-container {
		flex: 1;
		min-height: 0;
		overflow: hidden;
	}

	.action-bar {
		display: flex;
		gap: 16px;
		padding: 0;
		border-top: 1px solid #e5e7eb;
		background: white;
	}

	.action-buttons {
		display: flex;
		gap: 8px;
		padding: 16px 20px;
		border-right: 1px solid #e5e7eb;
	}

	.action-button {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 16px;
		font-size: 14px;
		font-weight: 500;
		border: 1px solid #d1d5db;
		border-radius: 6px;
		background: white;
		color: #374151;
		cursor: pointer;
		transition: all 0.2s;
	}

	.action-button:hover:not(:disabled) {
		background: #f9fafb;
		border-color: #9ca3af;
	}

	.action-button:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.stop-button:hover:not(:disabled) {
		background: #fee2e2;
		border-color: #fca5a5;
		color: #dc2626;
	}

	.open-button:hover:not(:disabled) {
		background: #dbeafe;
		border-color: #93c5fd;
		color: #2563eb;
	}

	.prompt-container {
		flex: 1;
		min-width: 0;
	}
</style>
