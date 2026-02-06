<script lang="ts">
	import type { Message } from '$lib/types';
	import ToolCallBlock from './ToolCallBlock.svelte';

	interface Props {
		message: Message;
	}

	let { message }: Props = $props();

	// Format timestamp to readable time
	function formatTime(isoTimestamp: string): string {
		const date = new Date(isoTimestamp);
		return date.toLocaleTimeString('en-US', {
			hour: '2-digit',
			minute: '2-digit'
		});
	}
</script>

<div class="message-bubble" class:user={message.role === 'user'} class:assistant={message.role === 'assistant'}>
	<div class="message-header">
		<span class="message-role">{message.role === 'user' ? 'You' : 'Claude'}</span>
		<span class="message-time">{formatTime(message.timestamp)}</span>
	</div>

	{#if message.content}
		<div class="message-content">{message.content}</div>
	{/if}

	{#if message.toolCalls && message.toolCalls.length > 0}
		<div class="tool-calls">
			{#each message.toolCalls as toolCall}
				<ToolCallBlock {toolCall} />
			{/each}
		</div>
	{/if}
</div>

<style>
	.message-bubble {
		margin: 12px 0;
		padding: 12px 16px;
		border-radius: 8px;
		max-width: 85%;
	}

	.message-bubble.user {
		background: #eff6ff;
		border: 1px solid #bfdbfe;
		margin-left: auto;
	}

	.message-bubble.assistant {
		background: #f3f4f6;
		border: 1px solid #e5e7eb;
		margin-right: auto;
	}

	.message-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 6px;
		gap: 12px;
	}

	.message-role {
		font-weight: 600;
		font-size: 13px;
		color: #374151;
	}

	.message-bubble.user .message-role {
		color: #1e40af;
	}

	.message-time {
		font-size: 11px;
		color: #9ca3af;
	}

	.message-content {
		color: #1f2937;
		font-size: 14px;
		line-height: 1.6;
		white-space: pre-wrap;
		word-wrap: break-word;
	}

	.tool-calls {
		margin-top: 8px;
	}
</style>
