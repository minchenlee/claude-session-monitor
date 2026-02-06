<script lang="ts">
	import { currentConversation, selectedSessionId } from '$lib/stores/sessions';
	import MessageBubble from './MessageBubble.svelte';
	import { onMount, tick } from 'svelte';

	let conversation = $derived($currentConversation);
	let hasSelectedSession = $derived($selectedSessionId !== null);
	let messagesContainer: HTMLDivElement;

	// Auto-scroll to bottom when new messages arrive
	$effect(() => {
		if (conversation && conversation.messages.length > 0) {
			tick().then(() => {
				if (messagesContainer) {
					messagesContainer.scrollTop = messagesContainer.scrollHeight;
				}
			});
		}
	});
</script>

<div class="conversation-view">
	<div class="conversation-header">
		<h2>Conversation</h2>
		{#if conversation}
			<span class="message-count">{conversation.messages.length} messages</span>
		{/if}
	</div>

	<div class="conversation-content" bind:this={messagesContainer}>
		{#if !hasSelectedSession}
			<div class="empty-state">
				<p>No session selected</p>
				<p class="empty-hint">Select a session from the left panel to view its conversation</p>
			</div>
		{:else if !conversation}
			<div class="loading-state">
				<p>Loading conversation...</p>
			</div>
		{:else if conversation.messages.length === 0}
			<div class="empty-state">
				<p>No messages yet</p>
				<p class="empty-hint">This conversation is empty</p>
			</div>
		{:else}
			<div class="messages">
				{#each conversation.messages as message, index (index)}
					<MessageBubble {message} />
				{/each}
			</div>
		{/if}
	</div>
</div>

<style>
	.conversation-view {
		display: flex;
		flex-direction: column;
		height: 100%;
		background: white;
	}

	.conversation-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 16px 20px;
		border-bottom: 1px solid #e5e7eb;
		background: white;
	}

	.conversation-header h2 {
		margin: 0;
		font-size: 18px;
		font-weight: 600;
		color: #111827;
	}

	.message-count {
		background: #e5e7eb;
		color: #6b7280;
		font-size: 12px;
		font-weight: 600;
		padding: 2px 8px;
		border-radius: 12px;
	}

	.conversation-content {
		flex: 1;
		overflow-y: auto;
		padding: 20px;
	}

	.messages {
		display: flex;
		flex-direction: column;
	}

	.empty-state,
	.loading-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100%;
		text-align: center;
		padding: 40px 20px;
	}

	.empty-state p,
	.loading-state p {
		margin: 0;
		color: #6b7280;
		font-size: 14px;
	}

	.empty-hint {
		margin-top: 8px;
		font-size: 12px;
		color: #9ca3af;
	}
</style>
