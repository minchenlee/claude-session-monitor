/**
 * Auto-updater — checks for new versions on app launch.
 * Downloads in background, then prompts user to restart.
 * Only runs inside Tauri desktop; no-op in WebSocket/browser mode.
 */

import { writable } from 'svelte/store';
import { isTauri } from './ws';

/** Exposed so the UI can show an update banner */
export const pendingUpdate = writable<{ version: string } | null>(null);

/**
 * Check for updates, download in background, and notify the UI.
 * Call this once on app startup (e.g. in +layout.svelte onMount).
 */
export async function checkForUpdates() {
	if (!isTauri()) return;

	try {
		const { check } = await import('@tauri-apps/' + 'plugin-updater');

		const update = await check();
		if (!update) return;

		console.log(`[updater] New version available: ${update.version}`);

		await update.downloadAndInstall();

		console.log(`[updater] Update ${update.version} ready — waiting for user to restart`);
		pendingUpdate.set({ version: update.version });
	} catch (error) {
		// Updater failures should never break the app
		console.error('[updater] Update check failed:', error);
	}
}

/** Called when user clicks "Restart Now" */
export async function restartToUpdate() {
	try {
		const { relaunch } = await import('@tauri-apps/' + 'plugin-process');
		await relaunch();
	} catch (error) {
		console.error('[updater] Relaunch failed:', error);
	}
}
