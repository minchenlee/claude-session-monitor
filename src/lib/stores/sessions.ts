/**
 * Svelte stores for session state management
 */

import { writable, derived, get } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';
import type { Session, Conversation } from '../types';
import { SessionStatus } from '../types';
import { isDemoMode } from '../demo';

/**
 * Store containing all active sessions
 */
export const sessions = writable<Session[]>([]);

/**
 * Store containing the currently expanded session ID (for overlay)
 */
export const expandedSessionId = writable<string | null>(null);

/**
 * Store containing the conversation for the currently expanded session
 */
export const currentConversation = writable<Conversation | null>(null);

/**
 * Derived store: sessions sorted by attention priority
 * Priority: NeedsPermission > WaitingForInput > Working > Connecting
 */
export const sortedSessions = derived(sessions, ($sessions) => {
	const priorityOrder: Record<SessionStatus, number> = {
		[SessionStatus.NeedsPermission]: 0,
		[SessionStatus.WaitingForInput]: 1,
		[SessionStatus.Working]: 2,
		[SessionStatus.Connecting]: 3
	};

	return [...$sessions].sort((a, b) => {
		const priorityA = priorityOrder[a.status] ?? 4;
		const priorityB = priorityOrder[b.status] ?? 4;
		if (priorityA !== priorityB) {
			return priorityA - priorityB;
		}
		// Same priority: sort by most recent activity (newest at bottom)
		return new Date(a.modified).getTime() - new Date(b.modified).getTime();
	});
});

/**
 * Derived store: count of sessions needing attention
 */
export const attentionCount = derived(sessions, ($sessions) => {
	return $sessions.filter(
		(s) => s.status === SessionStatus.NeedsPermission || s.status === SessionStatus.WaitingForInput
	).length;
});

/**
 * Derived store: status summary for header
 */
export const statusSummary = derived(sessions, ($sessions) => {
	const working = $sessions.filter((s) => s.status === SessionStatus.Working || s.status === SessionStatus.Connecting).length;
	const permission = $sessions.filter((s) => s.status === SessionStatus.NeedsPermission).length;
	const input = $sessions.filter((s) => s.status === SessionStatus.WaitingForInput).length;

	return { working, permission, input };
});

/**
 * Initialize event listeners for backend updates
 * Call this once when the app starts
 */
export async function initializeSessionListeners() {
	// Listen for session updates from the backend polling loop
	await listen<Session[]>('sessions-updated', (event) => {
		if (!get(isDemoMode)) {
			sessions.set(event.payload);
		}
	});

	// Listen for conversation updates
	await listen<Conversation>('conversation-updated', (event) => {
		currentConversation.set(event.payload);
	});
}

// Legacy alias for backward compatibility
export const selectedSessionId = expandedSessionId;
