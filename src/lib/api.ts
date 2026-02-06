/**
 * API wrapper for Tauri backend commands
 */

import { invoke } from '@tauri-apps/api/core';
import type { Session, Conversation } from './types';

/**
 * Get all active Claude Code sessions
 * @returns Promise resolving to array of sessions
 */
export async function getSessions(): Promise<Session[]> {
  return await invoke<Session[]>('get_sessions');
}

/**
 * Get the full conversation history for a specific session
 * @param sessionId - The session UUID
 * @returns Promise resolving to the conversation
 */
export async function getConversation(sessionId: string): Promise<Conversation> {
  return await invoke<Conversation>('get_conversation', { sessionId });
}

/**
 * Send a prompt to a specific session
 * @param sessionId - The session UUID
 * @param prompt - The prompt text to send
 * @returns Promise resolving when the prompt has been sent
 */
export async function sendPrompt(sessionId: string, prompt: string): Promise<void> {
  await invoke<void>('send_prompt', { sessionId, prompt });
}

/**
 * Stop a running session by sending SIGINT
 * @param sessionId - The session UUID
 * @returns Promise resolving when the stop signal has been sent
 */
export async function stopSession(sessionId: string): Promise<void> {
  await invoke<void>('stop_session', { sessionId });
}

/**
 * Open the terminal or IDE window for a session
 * @param sessionId - The session UUID
 * @returns Promise resolving when the window has been opened/focused
 */
export async function openSession(sessionId: string): Promise<void> {
  await invoke<void>('open_session', { sessionId });
}
