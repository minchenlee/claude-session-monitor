<script lang="ts">
	import { selectedSessionId } from '$lib/stores/sessions';

	interface Props {
		onsend?: (prompt: string) => void;
	}

	let { onsend }: Props = $props();

	let promptText = $state('');
	let textareaElement: HTMLTextAreaElement;
	let hasSelectedSession = $derived($selectedSessionId !== null);

	function handleSend() {
		const trimmedPrompt = promptText.trim();
		if (trimmedPrompt && onsend) {
			onsend(trimmedPrompt);
			promptText = '';
			// Reset textarea height
			if (textareaElement) {
				textareaElement.style.height = 'auto';
			}
		}
	}

	function handleKeyDown(event: KeyboardEvent) {
		// Send on Ctrl+Enter or Cmd+Enter
		if ((event.ctrlKey || event.metaKey) && event.key === 'Enter') {
			event.preventDefault();
			handleSend();
		}
	}

	function handleInput() {
		// Auto-resize textarea
		if (textareaElement) {
			textareaElement.style.height = 'auto';
			textareaElement.style.height = textareaElement.scrollHeight + 'px';
		}
	}
</script>

<div class="prompt-input">
	<div class="input-container">
		<textarea
			bind:this={textareaElement}
			bind:value={promptText}
			onkeydown={handleKeyDown}
			oninput={handleInput}
			placeholder={hasSelectedSession
				? 'Type your message... (Ctrl+Enter to send)'
				: 'Select a session to send a message'}
			disabled={!hasSelectedSession}
			rows="1"
		></textarea>
		<button
			class="send-button"
			onclick={handleSend}
			disabled={!hasSelectedSession || !promptText.trim()}
			type="button"
			aria-label="Send message"
		>
			<svg
				width="20"
				height="20"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<line x1="22" y1="2" x2="11" y2="13"></line>
				<polygon points="22 2 15 22 11 13 2 9 22 2"></polygon>
			</svg>
		</button>
	</div>
</div>

<style>
	.prompt-input {
		border-top: 1px solid #e5e7eb;
		background: white;
		padding: 16px 20px;
	}

	.input-container {
		display: flex;
		gap: 12px;
		align-items: flex-end;
	}

	textarea {
		flex: 1;
		padding: 12px 16px;
		border: 1px solid #d1d5db;
		border-radius: 8px;
		font-size: 14px;
		font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
		resize: none;
		max-height: 200px;
		overflow-y: auto;
		transition: border-color 0.2s;
	}

	textarea:focus {
		outline: none;
		border-color: #3b82f6;
	}

	textarea:disabled {
		background: #f9fafb;
		color: #9ca3af;
		cursor: not-allowed;
	}

	.send-button {
		padding: 12px 16px;
		background: #3b82f6;
		color: white;
		border: none;
		border-radius: 8px;
		cursor: pointer;
		transition: background-color 0.2s;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.send-button:hover:not(:disabled) {
		background: #2563eb;
	}

	.send-button:disabled {
		background: #d1d5db;
		cursor: not-allowed;
	}
</style>
