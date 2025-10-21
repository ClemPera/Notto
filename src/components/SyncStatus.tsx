import React, { useEffect } from 'react'
import { useAppStore } from '../store/appStore'
import { syncCommands } from '../utils/tauri'

export const SyncStatus: React.FC = () => {
  const { sync_status, sync_message, last_sync_time, setSyncStatus } = useAppStore()

  // Poll sync status every 3 seconds
  useEffect(() => {
    const pollStatus = async () => {
      try {
        const response = await syncCommands.getStatus()
        setSyncStatus(
          response.status as 'idle' | 'syncing' | 'success' | 'error' | 'offline',
          response.message
        )
      } catch (error) {
        console.error('Failed to get sync status:', error)
      }
    }

    const interval = setInterval(pollStatus, 3000)
    return () => clearInterval(interval)
  }, [setSyncStatus])

  const getStatusIcon = () => {
    switch (sync_status) {
      case 'syncing':
        return '🔄'
      case 'success':
        return '✓'
      case 'error':
        return '✗'
      case 'offline':
        return '⚠️'
      default:
        return '•'
    }
  }

  const getStatusColor = () => {
    switch (sync_status) {
      case 'syncing':
        return 'bg-blue-100 dark:bg-blue-900/30 text-blue-800 dark:text-blue-200'
      case 'success':
        return 'bg-green-100 dark:bg-green-900/30 text-green-800 dark:text-green-200'
      case 'error':
        return 'bg-red-100 dark:bg-red-900/30 text-red-800 dark:text-red-200'
      case 'offline':
        return 'bg-yellow-100 dark:bg-yellow-900/30 text-yellow-800 dark:text-yellow-200'
      default:
        return 'bg-gray-100 dark:bg-gray-800 text-gray-800 dark:text-gray-200'
    }
  }

  const formatLastSync = () => {
    if (!last_sync_time) return 'Never'

    const seconds = Math.floor((Date.now() - last_sync_time.getTime()) / 1000)

    if (seconds < 60) return 'Just now'
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`
    if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`
    return `${Math.floor(seconds / 86400)}d ago`
  }

  return (
    <div className={`flex items-center gap-2 px-3 py-2 rounded-lg text-sm ${getStatusColor()}`}>
      <span className="text-lg">{getStatusIcon()}</span>
      <div className="flex flex-col">
        <span className="font-semibold capitalize">{sync_status}</span>
        {sync_message && <span className="text-xs">{sync_message}</span>}
      </div>
      <span className="ml-auto text-xs opacity-75">Last: {formatLastSync()}</span>
    </div>
  )
}
