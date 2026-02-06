/**
 * TypeScript type definitions for Claude Session Monitor
 */

/**
 * Session status enumeration
 */
export enum SessionStatus {
  Working = 'Working',              // Executing tools/thinking
  NeedsPermission = 'NeedsPermission', // Waiting for user approval
  WaitingForInput = 'WaitingForInput', // Idle, ready for prompt
  Connecting = 'Connecting'            // Session starting up
}

/**
 * A Claude Code session
 */
export interface Session {
  /** Session UUID */
  id: string;

  /** Process ID of the running Claude instance */
  pid: number;

  /** Project directory name */
  projectName: string;

  /** Full path to project directory */
  projectPath: string;

  /** Git branch name (if available) */
  gitBranch: string | null;

  /** Summary of the first prompt (shown in list view) */
  firstPrompt: string;

  /** Total number of messages in the conversation */
  messageCount: number;

  /** Timestamp of last activity (ISO 8601 string) */
  modified: string;

  /** Current status of the session */
  status: SessionStatus;
}

/**
 * Message type in conversation
 */
export type MessageType = 'User' | 'Assistant' | 'Thinking' | 'ToolUse' | 'ToolResult';

/**
 * A message in a conversation
 */
export interface Message {
  /** Message timestamp (ISO 8601 string) */
  timestamp: string;

  /** Message type */
  messageType: MessageType;

  /** Message content text */
  content: string;
}

/**
 * A conversation containing all messages for a session
 */
export interface Conversation {
  /** Session ID this conversation belongs to */
  sessionId: string;

  /** Array of messages in chronological order */
  messages: Message[];
}
