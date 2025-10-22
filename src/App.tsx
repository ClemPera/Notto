import React, { useState, useEffect } from 'react'
import { useAppStore } from './store/appStore'
import { LoginForm } from './components/LoginForm'
import { RegisterForm } from './components/RegisterForm'
import { MarkdownEditor } from './components/MarkdownEditor'
import { NoteList } from './components/NoteList'
import { SyncStatus } from './components/SyncStatus'
import { authCommands } from './utils/tauri'

type AuthView = 'login' | 'register'

export const App: React.FC = () => {
  const [authView, setAuthView] = useState<AuthView>('login')
  const { isAuthenticated, token, toggleTheme, theme, sidebar_open, toggleSidebar } = useAppStore()

  // Check if user is already authenticated on mount
  useEffect(() => {
    const checkAuth = async () => {
      const savedToken = localStorage.getItem('auth_token')
      if (savedToken) {
        try {
          const user_id = await authCommands.verifySessionToken(savedToken)
          if (user_id) {
            useAppStore.setState({
              token: savedToken,
              user_id,
              isAuthenticated: true,
            })
          }
        } catch (error) {
          localStorage.removeItem('auth_token')
        }
      }
    }

    checkAuth()
  }, [])

  // Save token to localStorage when authenticated
  useEffect(() => {
    if (token && isAuthenticated) {
      localStorage.setItem('auth_token', token)
    }
  }, [token, isAuthenticated])

  const handleAuthSuccess = () => {
    // User is already logged in via setAuthToken in store
  }

  const handleLogout = async () => {
    const { user_id, clearAuth } = useAppStore.getState()
    if (user_id) {
      try {
        await authCommands.logout(user_id)
      } catch (error) {
        console.error('Logout error:', error)
      }
    }
    clearAuth()
    localStorage.removeItem('auth_token')
    setAuthView('login')
  }

  if (!isAuthenticated) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-blue-50 to-blue-100 dark:from-gray-900 dark:to-gray-800 flex items-center justify-center p-4">
        <div className="w-full">
          {authView === 'login' ? (
            <LoginForm
              onSuccess={handleAuthSuccess}
              onSwitchToRegister={() => setAuthView('register')}
            />
          ) : (
            <RegisterForm
              onSuccess={handleAuthSuccess}
              onSwitchToLogin={() => setAuthView('login')}
            />
          )}
        </div>
      </div>
    )
  }

  return (
    <div className={theme === 'dark' ? 'dark' : ''}>
      <div className="h-screen flex flex-col bg-white dark:bg-gray-900 text-gray-900 dark:text-white">
        {/* Header */}
        <header className="bg-white dark:bg-gray-900 border-b border-gray-200 dark:border-gray-700 px-6 py-4 flex justify-between items-center">
          <div className="flex items-center gap-4">
            <button
              onClick={toggleSidebar}
              className="p-2 hover:bg-gray-200 dark:hover:bg-gray-800 rounded-lg"
            >
              ☰
            </button>
            <h1 className="text-2xl font-bold">Notto</h1>
          </div>

          <div className="flex items-center gap-4">
            <SyncStatus />
            <button
              onClick={toggleTheme}
              className="p-2 hover:bg-gray-200 dark:hover:bg-gray-800 rounded-lg"
            >
              {theme === 'light' ? '🌙' : '☀️'}
            </button>
            <button
              onClick={handleLogout}
              className="px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg font-semibold"
            >
              Logout
            </button>
          </div>
        </header>

        {/* Main content */}
        <div className="flex-1 flex overflow-hidden">
          {/* Sidebar */}
          {sidebar_open && <NoteList />}

          {/* Editor */}
          <MarkdownEditor />
        </div>
      </div>
    </div>
  )
}
