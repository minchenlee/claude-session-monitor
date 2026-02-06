/**
 * Svelte stores for session state management
 */

import { writable } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';
import type { Session, Conversation } from '../types';

/**
 * Store containing all active sessions
 */
export const sessions = writable<Session[]>([]);

/**
 * Store containing the currently selected session ID
 */
export const selectedSessionId = writable<string | null>(null);

/**
 * Store containing the conversation for the currently selected session
 */
export const currentConversation = writable<Conversation | null>(null);

/**
 * Initialize event listeners for backend updates
 * Call this once when the app starts
 */
export async function initializeSessionListeners() {
  // Listen for session updates from the backend polling loop
  await listen<Session[]>('sessions-updated', (event) => {
    sessions.set(event.payload);
  });

  // Listen for conversation updates
  await listen<Conversation>('conversation-updated', (event) => {
    currentConversation.set(event.payload);
  });
}
