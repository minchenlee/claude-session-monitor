<script lang="ts">
	import { pendingUpdate, restartToUpdate } from '$lib/updater';
</script>

{#if $pendingUpdate}
	<div class="update-banner" role="alert">
		<span class="update-text">
			Update v{$pendingUpdate.version} is ready
		</span>
		<button class="update-btn" onclick={restartToUpdate}>
			Restart Now
		</button>
	</div>
{/if}

<style>
	.update-banner {
		position: fixed;
		bottom: 0;
		left: 0;
		right: 0;
		z-index: 10000;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 12px;
		padding: 8px 16px;
		background: var(--bg-elevated, #1a1a2e);
		border-top: 1px solid var(--accent-green, #00ff88);
	}

	.update-text {
		font-family: var(--font-mono, monospace);
		font-size: 12px;
		color: var(--text-secondary, #aaa);
	}

	.update-btn {
		font-family: var(--font-pixel, monospace);
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		padding: 4px 12px;
		background: var(--accent-green, #00ff88);
		color: var(--bg-primary, #000);
		border: none;
		cursor: pointer;
	}

	.update-btn:hover {
		opacity: 0.85;
	}
</style>
