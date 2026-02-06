<script lang="ts">
	import type { ToolCall } from '$lib/types';

	interface Props {
		toolCall: ToolCall;
	}

	let { toolCall }: Props = $props();

	let inputExpanded = $state(false);
	let outputExpanded = $state(false);

	function formatJson(obj: unknown): string {
		return JSON.stringify(obj, null, 2);
	}
</script>

<div class="tool-call-block" class:completed={toolCall.completed}>
	<div class="tool-header">
		<span class="tool-icon">{toolCall.completed ? '✓' : '⋯'}</span>
		<span class="tool-name">{toolCall.name}</span>
		<span class="tool-status">{toolCall.completed ? 'completed' : 'pending'}</span>
	</div>

	<div class="tool-section">
		<button
			class="section-toggle"
			onclick={() => (inputExpanded = !inputExpanded)}
			type="button"
		>
			<span class="toggle-icon">{inputExpanded ? '▼' : '▶'}</span>
			Input
		</button>
		{#if inputExpanded}
			<pre class="tool-data">{formatJson(toolCall.input)}</pre>
		{/if}
	</div>

	{#if toolCall.output !== undefined}
		<div class="tool-section">
			<button
				class="section-toggle"
				onclick={() => (outputExpanded = !outputExpanded)}
				type="button"
			>
				<span class="toggle-icon">{outputExpanded ? '▼' : '▶'}</span>
				Output
			</button>
			{#if outputExpanded}
				<pre class="tool-data">{toolCall.output}</pre>
			{/if}
		</div>
	{/if}
</div>

<style>
	.tool-call-block {
		border: 1px solid #e5e7eb;
		border-radius: 6px;
		padding: 10px;
		margin: 8px 0;
		background: #f9fafb;
	}

	.tool-call-block.completed {
		background: #f0fdf4;
		border-color: #86efac;
	}

	.tool-header {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-bottom: 8px;
	}

	.tool-icon {
		font-size: 14px;
		width: 16px;
		text-align: center;
	}

	.tool-name {
		font-family: monospace;
		font-size: 13px;
		font-weight: 600;
		color: #374151;
	}

	.tool-status {
		margin-left: auto;
		font-size: 11px;
		color: #6b7280;
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}

	.tool-section {
		margin-top: 6px;
	}

	.section-toggle {
		background: none;
		border: none;
		padding: 4px 0;
		cursor: pointer;
		font-size: 12px;
		font-weight: 500;
		color: #4b5563;
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.section-toggle:hover {
		color: #111827;
	}

	.toggle-icon {
		font-size: 10px;
		width: 12px;
	}

	.tool-data {
		background: white;
		border: 1px solid #e5e7eb;
		border-radius: 4px;
		padding: 8px;
		margin-top: 4px;
		font-size: 11px;
		overflow-x: auto;
		color: #1f2937;
	}
</style>
