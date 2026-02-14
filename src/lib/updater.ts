/**
 * Auto-updater — checks for new versions on app launch and installs them.
 * Only runs inside Tauri desktop; no-op in WebSocket/browser mode.
 */

import { isTauri } from './ws';

/**
 * Check for updates and prompt the user to install if available.
 * Call this once on app startup (e.g. in +layout.svelte onMount).
 */
export async function checkForUpdates() {
	if (!isTauri()) return;

	try {
		const { check } = await import('@tauri-apps/' + 'plugin-updater');
		const { relaunch } = await import('@tauri-apps/' + 'plugin-process');

		const update = await check();
		if (!update) return;

		console.log(`[updater] New version available: ${update.version}`);

		await update.downloadAndInstall();
		await relaunch();
	} catch (error) {
		// Updater failures should never break the app
		console.error('[updater] Update check failed:', error);
	}
}
