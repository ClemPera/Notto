import { invoke } from '@tauri-apps/api/tauri'

// Type definitions for all Tauri commands
export interface RegisterRequest {
  username: string
  password: string
}

export interface RegisterResponse {
  user_id: string
  recovery_phrase: string
}

export interface LoginRequest {
  username: string
  password: string
}

export interface LoginResponse {
  token: string
}

export interface SetupTotpRequest {
  token: string
}

export interface SetupTotpResponse {
  secret: string
  backup_codes: string[]
  qr_code_uri: string
}

export interface CreateNoteRequest {
  title: string
  content: string
  folder_id?: string
}

export interface CreateFolderRequest {
  name: string
  parent_id?: string
}

export interface CreateFolderResponse {
  folder_id: string
}

export interface ListFoldersResponse {
  folder_ids: string[]
}

export interface InitiateSyncRequest {
  token: string
  server_url: string
}

export interface SyncStatusResponse {
  status: string
  message?: string
}

// Auth commands
export const authCommands = {
  register: async (req: RegisterRequest): Promise<RegisterResponse> => {
    return invoke('register', { req })
  },

  login: async (req: LoginRequest): Promise<LoginResponse> => {
    return invoke('login', { req })
  },

  setupTotp: async (req: SetupTotpRequest): Promise<SetupTotpResponse> => {
    return invoke('setup_totp', { req })
  },

  verifyTotpSetup: async (token: string, secret: string, code: string): Promise<boolean> => {
    return invoke('verify_totp_setup', { token, secret, code })
  },

  verifySessionToken: async (token: string): Promise<string> => {
    return invoke('verify_session_token', { token })
  },

  logout: async (user_id: string): Promise<{ success: boolean }> => {
    return invoke('logout', { user_id })
  },
}

// Note commands
export const noteCommands = {
  create: async (req: CreateNoteRequest): Promise<string> => {
    return invoke('create_note', { req })
  },

  get: async (note_id: string): Promise<string> => {
    return invoke('get_note', { note_id })
  },

  update: async (note_id: string, title: string, content: string): Promise<void> => {
    return invoke('update_note', { note_id, title, content })
  },

  delete: async (note_id: string): Promise<void> => {
    return invoke('delete_note', { note_id })
  },

  list: async (folder_id?: string): Promise<string[]> => {
    return invoke('list_notes', { folder_id })
  },
}

// Folder commands
export const folderCommands = {
  create: async (req: CreateFolderRequest): Promise<CreateFolderResponse> => {
    return invoke('create_folder', { req })
  },

  list: async (): Promise<ListFoldersResponse> => {
    return invoke('list_folders')
  },
}

// Sync commands
export const syncCommands = {
  initialize: async (req: InitiateSyncRequest): Promise<string> => {
    return invoke('initialize_sync', { req })
  },

  start: async (): Promise<SyncStatusResponse> => {
    return invoke('start_sync')
  },

  getStatus: async (): Promise<SyncStatusResponse> => {
    return invoke('get_sync_status')
  },

  checkConnectivity: async (server_url: string): Promise<boolean> => {
    return invoke('check_connectivity', { server_url })
  },
}
