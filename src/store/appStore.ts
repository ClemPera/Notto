import { create } from 'zustand'
import { authCommands } from '../utils/tauri'

export interface AuthState {
  token: string | null
  user_id: string | null
  username: string | null
  isAuthenticated: boolean
  is_2fa_setup: boolean
}

export interface EditorState {
  current_note_id: string | null
  current_note_title: string
  current_note_content: string
  is_editing: boolean
  has_unsaved_changes: boolean
}

export interface SyncState {
  sync_status: 'idle' | 'syncing' | 'success' | 'error' | 'offline'
  sync_message: string | null
  last_sync_time: Date | null
  server_url: string | null
}

export interface UIState {
  theme: 'light' | 'dark'
  sidebar_open: boolean
  show_preview: boolean
}

interface AppStore extends AuthState, EditorState, SyncState, UIState {
  // Auth actions
  setAuthToken: (token: string, user_id: string, username: string) => void
  clearAuth: () => void
  set2FASetup: (setup: boolean) => void

  // Editor actions
  setCurrentNote: (note_id: string, title: string, content: string) => void
  updateNoteContent: (content: string) => void
  updateNoteTitle: (title: string) => void
  markSaved: () => void
  clearCurrentNote: () => void

  // Sync actions
  setSyncStatus: (status: 'idle' | 'syncing' | 'success' | 'error' | 'offline', message?: string) => void
  setServerUrl: (url: string) => void
  updateLastSyncTime: () => void

  // UI actions
  toggleTheme: () => void
  toggleSidebar: () => void
  togglePreview: () => void
}

export const useAppStore = create<AppStore>((set) => ({
  // Auth state
  token: null,
  user_id: null,
  username: null,
  isAuthenticated: false,
  is_2fa_setup: false,

  // Editor state
  current_note_id: null,
  current_note_title: '',
  current_note_content: '',
  is_editing: false,
  has_unsaved_changes: false,

  // Sync state
  sync_status: 'idle',
  sync_message: null,
  last_sync_time: null,
  server_url: null,

  // UI state
  theme: 'light',
  sidebar_open: true,
  show_preview: true,

  // Auth actions
  setAuthToken: (token: string, user_id: string, username: string) =>
    set({
      token,
      user_id,
      username,
      isAuthenticated: true,
    }),

  clearAuth: () =>
    set({
      token: null,
      user_id: null,
      username: null,
      isAuthenticated: false,
      is_2fa_setup: false,
    }),

  set2FASetup: (setup: boolean) =>
    set({ is_2fa_setup: setup }),

  // Editor actions
  setCurrentNote: (note_id: string, title: string, content: string) =>
    set({
      current_note_id: note_id,
      current_note_title: title,
      current_note_content: content,
      is_editing: true,
      has_unsaved_changes: false,
    }),

  updateNoteContent: (content: string) =>
    set({
      current_note_content: content,
      has_unsaved_changes: true,
    }),

  updateNoteTitle: (title: string) =>
    set({
      current_note_title: title,
      has_unsaved_changes: true,
    }),

  markSaved: () =>
    set({ has_unsaved_changes: false }),

  clearCurrentNote: () =>
    set({
      current_note_id: null,
      current_note_title: '',
      current_note_content: '',
      is_editing: false,
      has_unsaved_changes: false,
    }),

  // Sync actions
  setSyncStatus: (status: 'idle' | 'syncing' | 'success' | 'error' | 'offline', message?: string) =>
    set({
      sync_status: status,
      sync_message: message || null,
    }),

  setServerUrl: (url: string) =>
    set({ server_url: url }),

  updateLastSyncTime: () =>
    set({ last_sync_time: new Date() }),

  // UI actions
  toggleTheme: () =>
    set((state) => ({
      theme: state.theme === 'light' ? 'dark' : 'light',
    })),

  toggleSidebar: () =>
    set((state) => ({
      sidebar_open: !state.sidebar_open,
    })),

  togglePreview: () =>
    set((state) => ({
      show_preview: !state.show_preview,
    })),
}))
