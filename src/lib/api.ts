/**
 * API wrapper for Tauri backend commands
 */

import { invoke } from '@tauri-apps/api/core';
import { get } from 'svelte/store';
import type { Session, Conversation } from './types';
import { isDemoMode } from './demo';
import { getDemoSessions, demoConversations } from './demo/data';

/**
 * Get all active Claude Code sessions
 * @returns Promise resolving to array of sessions
 */
export async function getSessions(): Promise<Session[]> {
  if (get(isDemoMode)) {
    return getDemoSessions();
  }
  return await invoke<Session[]>('get_sessions');
}

/**
 * Get the full conversation history for a specific session
 * @param sessionId - The session UUID
 * @returns Promise resolving to the conversation
 */
export async function getConversation(sessionId: string): Promise<Conversation> {
  if (get(isDemoMode)) {
    return demoConversations[sessionId] ?? { sessionId, messages: [] };
  }
  return await invoke<Conversation>('get_conversation', { sessionId });
}

/**
 * Stop a running session by terminating the process
 * On Unix: Sends SIGTERM for graceful termination
 * On Windows: Uses taskkill for process termination
 * @param pid - The process ID of the Claude session
 * @returns Promise resolving when the stop signal has been sent
 */
export async function stopSession(pid: number): Promise<void> {
  if (get(isDemoMode)) return;
  await invoke<void>('stop_session', { pid });
}

/**
 * Open the terminal or IDE window for a session
 * @param pid - The process ID of the Claude session
 * @param projectPath - The project path for window matching
 * @returns Promise resolving when the window has been opened/focused
 */
export async function openSession(pid: number, projectPath: string): Promise<void> {
  if (get(isDemoMode)) return;
  await invoke<void>('open_session', { pid, projectPath });
}

