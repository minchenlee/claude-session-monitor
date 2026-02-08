/**
 * Auto-updater — checks for new versions on app launch and installs them.
 */

import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

/**
 * Check for updates and prompt the user to install if available.
 * Call this once on app startup (e.g. in +layout.svelte onMount).
 */
export async function checkForUpdates() {
	try {
		const update = await check();
		if (!update) return;

		console.log(`[updater] New version available: ${update.version}`);

		// Download and install
		await update.downloadAndInstall();

		// Relaunch the app to apply
		await relaunch();
	} catch (error) {
		// Updater failures should never break the app
		console.error('[updater] Update check failed:', error);
	}
}
