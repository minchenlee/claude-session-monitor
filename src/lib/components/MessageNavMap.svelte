<script lang="ts">
	import type { Conversation, Message } from '$lib/types';

	interface Props {
		conversation: Conversation | null;
		scrollContainer: HTMLDivElement | null;
	}

	let { conversation, scrollContainer }: Props = $props();

	// Filter for "milestone" messages - user messages, tool blocks, and thinking steps
	let items = $derived.by(() => {
		if (!conversation) return [];
		return conversation.messages
			.map((msg, index) => ({ msg, index }))
			.filter(({ msg }) => 
				msg.messageType === 'User' || 
				msg.messageType === 'Thinking' || 
				(msg.messageType === 'ToolUse' && msg.content?.length > 0)
			);
	});

	function getMessageIcon(message: Message): string {
		switch (message.messageType) {
			case 'User':
				return '→';
			case 'Thinking':
				return '◇';
			case 'ToolUse':
				return '⚙';
			default:
				return '•';
		}
	}

	function getMessageColor(message: Message): string {
		switch (message.messageType) {
			case 'User':
				return 'var(--text-primary)';
			case 'Thinking':
				return 'var(--status-permission)';
			case 'ToolUse':
				return 'var(--status-input)';
			default:
				return 'var(--text-muted)';
		}
	}

	async function scrollToMessage(index: number) {
		if (!scrollContainer) return;
		
		const messages = scrollContainer.querySelector('.messages');
		if (!messages) return;
		
		const target = messages.children[index] as HTMLElement;
		if (target) {
			target.scrollIntoView({ behavior: 'smooth', block: 'start' });
		}
	}

	function truncateContent(content: string | undefined): string {
		if (!content) return '...';
		const clean = content.replace(/[#*`]/g, '').trim();
		return clean.length > 40 ? clean.substring(0, 40) + '...' : clean;
	}
</script>

<div class="nav-map-floating" class:hidden={!items.length}>
	<div class="nav-header">
		<span class="nav-title">Navigation</span>
		<span class="nav-count">{items.length} items</span>
	</div>
	<div class="nav-list">
		{#each items as { msg, index }}
			<!-- svelte-ignore a11y_click_events_have_key_events -->
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div 
				class="nav-item-descriptive" 
				class:is-user={msg.messageType === 'User'}
				class:is-thinking={msg.messageType === 'Thinking'}
				style="--item-color: {getMessageColor(msg)}"
				onclick={() => scrollToMessage(index)}
			>
				<span class="nav-icon">{getMessageIcon(msg)}</span>
				<span class="nav-text">{truncateContent(msg.content)}</span>
				<div class="nav-indicator"></div>
			</div>
		{/each}
	</div>
</div>

<style>
	.nav-map-floating {
		width: 240px;
		height: fit-content;
		max-height: 70vh;
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
		padding: var(--space-lg);
		background: var(--bg-card);
		border: 1px solid var(--border-default);
		pointer-events: auto;
		animation: slide-in-right 0.3s ease-out;
	}

	.nav-map-floating.hidden {
		display: none;
	}

	.nav-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding-bottom: var(--space-sm);
		border-bottom: 1px solid var(--border-muted);
	}

	.nav-title {
		font-family: var(--font-pixel);
		font-size: 10px;
		text-transform: uppercase;
		letter-spacing: 0.1em;
		color: var(--text-muted);
	}

	.nav-count {
		font-family: var(--font-mono);
		font-size: 9px;
		color: var(--text-muted);
		opacity: 0.5;
	}

	.nav-list {
		display: flex;
		flex-direction: column;
		gap: 2px;
		overflow-y: auto;
		padding-right: 4px;
	}

	.nav-item-descriptive {
		position: relative;
		display: flex;
		align-items: flex-start;
		gap: var(--space-sm);
		padding: 6px 8px;
		cursor: pointer;
		transition: all var(--transition-fast);
		border: 1px solid transparent;
	}

	.nav-item-descriptive:hover {
		background: rgba(255, 255, 255, 0.03);
		border-color: var(--border-muted);
	}

	.nav-icon {
		font-family: var(--font-mono);
		font-size: 10px;
		color: var(--item-color);
		flex-shrink: 0;
		margin-top: 1px;
	}

	.nav-text {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--text-secondary);
		line-height: 1.4;
		word-break: break-all;
	}

	.nav-item-descriptive:hover .nav-text {
		color: var(--text-primary);
	}

	.nav-indicator {
		position: absolute;
		left: 0;
		top: 50%;
		transform: translateY(-50%);
		width: 2px;
		height: 0;
		background: var(--item-color);
		transition: height var(--transition-fast);
	}

	.nav-item-descriptive:hover .nav-indicator {
		height: 60%;
	}

	.is-user {
		margin-bottom: 4px;
	}

	.is-thinking {
		opacity: 0.8;
		padding-left: calc(var(--space-sm) + 8px);
	}

	.is-thinking .nav-text {
		font-style: italic;
	}

	@keyframes slide-in-right {
		from { opacity: 0; transform: translateX(20px); }
		to { opacity: 1; transform: translateX(0); }
	}

	/* Scrollbar for nav list */
	.nav-list::-webkit-scrollbar {
		width: 2px;
	}
	.nav-list::-webkit-scrollbar-thumb {
		background: var(--border-default);
	}
</style>
