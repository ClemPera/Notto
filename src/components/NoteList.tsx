import React, { useState, useEffect } from 'react'
import { useAppStore } from '../store/appStore'
import { noteCommands, folderCommands } from '../utils/tauri'

export const NoteList: React.FC = () => {
  const [notes, setNotes] = useState<Array<{ id: string; title: string }>>([])
  const [folders, setFolders] = useState<Array<{ id: string; name: string }>>([])
  const [loading, setLoading] = useState(false)
  const [newNoteTitle, setNewNoteTitle] = useState('')
  const [newFolderName, setNewFolderName] = useState('')

  const { current_note_id, setCurrentNote } = useAppStore()

  // Load initial notes and folders
  useEffect(() => {
    loadNotes()
    loadFolders()
  }, [])

  const loadNotes = async () => {
    try {
      setLoading(true)
      const noteIds = await noteCommands.list()
      // For demo, we'll just use the IDs as titles
      setNotes(noteIds.map((id) => ({ id, title: `Note: ${id.slice(0, 8)}` })))
    } catch (error) {
      console.error('Failed to load notes:', error)
    } finally {
      setLoading(false)
    }
  }

  const loadFolders = async () => {
    try {
      const response = await folderCommands.list()
      setFolders(response.folder_ids.map((id) => ({ id, name: `Folder: ${id.slice(0, 8)}` })))
    } catch (error) {
      console.error('Failed to load folders:', error)
    }
  }

  const handleCreateNote = async () => {
    if (!newNoteTitle.trim()) return

    try {
      const noteId = await noteCommands.create({
        title: newNoteTitle,
        content: '',
      })
      setNewNoteTitle('')
      setCurrentNote(noteId, newNoteTitle, '')
      await loadNotes()
    } catch (error) {
      console.error('Failed to create note:', error)
    }
  }

  const handleCreateFolder = async () => {
    if (!newFolderName.trim()) return

    try {
      const response = await folderCommands.create({
        name: newFolderName,
      })
      setNewFolderName('')
      await loadFolders()
    } catch (error) {
      console.error('Failed to create folder:', error)
    }
  }

  const handleSelectNote = async (noteId: string) => {
    try {
      const content = await noteCommands.get(noteId)
      setCurrentNote(noteId, `Note: ${noteId.slice(0, 8)}`, content)
    } catch (error) {
      console.error('Failed to load note:', error)
    }
  }

  return (
    <div className="w-64 bg-gray-100 dark:bg-gray-900 border-r border-gray-300 dark:border-gray-700 flex flex-col h-full">
      {/* Create note/folder section */}
      <div className="p-4 border-b border-gray-300 dark:border-gray-700 space-y-2">
        <div className="flex gap-2">
          <input
            type="text"
            value={newNoteTitle}
            onChange={(e) => setNewNoteTitle(e.target.value)}
            onKeyPress={(e) => e.key === 'Enter' && handleCreateNote()}
            placeholder="New note..."
            className="flex-1 px-2 py-1 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded text-gray-900 dark:text-white text-sm focus:outline-none"
          />
          <button
            onClick={handleCreateNote}
            className="px-3 py-1 bg-blue-600 hover:bg-blue-700 text-white rounded text-sm font-semibold"
          >
            +
          </button>
        </div>

        <div className="flex gap-2">
          <input
            type="text"
            value={newFolderName}
            onChange={(e) => setNewFolderName(e.target.value)}
            onKeyPress={(e) => e.key === 'Enter' && handleCreateFolder()}
            placeholder="New folder..."
            className="flex-1 px-2 py-1 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded text-gray-900 dark:text-white text-sm focus:outline-none"
          />
          <button
            onClick={handleCreateFolder}
            className="px-3 py-1 bg-gray-400 hover:bg-gray-500 text-white rounded text-sm font-semibold"
          >
            Folder
          </button>
        </div>
      </div>

      {/* Notes and Folders list */}
      <div className="flex-1 overflow-auto p-4 space-y-4">
        {/* Folders section */}
        {folders.length > 0 && (
          <div>
            <h3 className="text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase mb-2">
              Folders
            </h3>
            <div className="space-y-1">
              {folders.map((folder) => (
                <div
                  key={folder.id}
                  className="flex items-center gap-2 px-2 py-1 rounded hover:bg-gray-200 dark:hover:bg-gray-800 cursor-pointer text-gray-700 dark:text-gray-300"
                >
                  <span className="text-lg">📁</span>
                  <span className="text-sm truncate">{folder.name}</span>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Notes section */}
        {notes.length > 0 && (
          <div>
            <h3 className="text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase mb-2">
              Notes
            </h3>
            <div className="space-y-1">
              {notes.map((note) => (
                <div
                  key={note.id}
                  onClick={() => handleSelectNote(note.id)}
                  className={`px-2 py-2 rounded cursor-pointer text-sm truncate transition-colors ${
                    current_note_id === note.id
                      ? 'bg-blue-600 text-white'
                      : 'text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-800'
                  }`}
                >
                  {note.title}
                </div>
              ))}
            </div>
          </div>
        )}

        {loading && (
          <p className="text-center text-gray-500 dark:text-gray-400 text-sm">Loading...</p>
        )}

        {!loading && notes.length === 0 && folders.length === 0 && (
          <p className="text-center text-gray-500 dark:text-gray-400 text-sm">
            No notes yet. Create one to get started!
          </p>
        )}
      </div>
    </div>
  )
}
