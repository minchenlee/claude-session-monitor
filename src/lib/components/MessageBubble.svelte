<script lang="ts">
	import type { Message } from '$lib/types';

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

	// Determine if this is a user message
	let isUser = $derived(message.messageType === 'User');
	let isAssistant = $derived(message.messageType === 'Assistant');
	let isThinking = $derived(message.messageType === 'Thinking');
	let isToolUse = $derived(message.messageType === 'ToolUse');
	let isToolResult = $derived(message.messageType === 'ToolResult');

	// Get display label for message type
	let roleLabel = $derived.by(() => {
		switch (message.messageType) {
			case 'User':
				return 'You';
			case 'Assistant':
				return 'Claude';
			case 'Thinking':
				return 'Claude (Thinking)';
			case 'ToolUse':
				return 'Tool Use';
			case 'ToolResult':
				return 'Tool Result';
			default:
				return 'Unknown';
		}
	});
</script>

<div
	class="message-bubble"
	class:user={isUser}
	class:assistant={isAssistant}
	class:thinking={isThinking}
	class:tool={isToolUse || isToolResult}
>
	<div class="message-header">
		<span class="message-role">{roleLabel}</span>
		<span class="message-time">{formatTime(message.timestamp)}</span>
	</div>

	{#if message.content}
		<div class="message-content">{message.content}</div>
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

	.message-bubble.thinking {
		background: #fef3c7;
		border: 1px solid #fde68a;
		margin-right: auto;
	}

	.message-bubble.tool {
		background: #f0fdf4;
		border: 1px solid #d1fae5;
		margin-right: auto;
		font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New',
			monospace;
		font-size: 13px;
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
