<script lang="ts">
	import { onMount } from 'svelte';
	import { initializeSessionListeners, sessions } from '$lib/stores/sessions';
	import { getSessions } from '$lib/api';
	import { loadDemoDataIfActive } from '$lib/demo';
	import { checkForUpdates } from '$lib/updater';
	import '../app.css';

	onMount(async () => {
		// If demo mode was persisted, load demo data and skip real fetch
		const demoActive = loadDemoDataIfActive();

		// Initialize Tauri event listeners for real-time updates
		await initializeSessionListeners();

		if (!demoActive) {
			// Fetch initial session data and update store
			const initialSessions = await getSessions();
			sessions.set(initialSessions);
		}

		// Check for updates in the background (non-blocking)
		checkForUpdates();
	});
</script>

<slot />
