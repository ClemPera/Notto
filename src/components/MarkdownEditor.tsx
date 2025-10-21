import React, { useEffect } from 'react'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import { useAppStore } from '../store/appStore'
import { noteCommands } from '../utils/tauri'

export const MarkdownEditor: React.FC = () => {
  const {
    current_note_id,
    current_note_title,
    current_note_content,
    has_unsaved_changes,
    show_preview,
    updateNoteTitle,
    updateNoteContent,
    markSaved,
  } = useAppStore()

  const handleSave = async () => {
    if (!current_note_id || !has_unsaved_changes) return

    try {
      await noteCommands.update(current_note_id, current_note_title, current_note_content)
      markSaved()
    } catch (error) {
      console.error('Failed to save note:', error)
    }
  }

  // Auto-save when changes stop for 2 seconds
  useEffect(() => {
    const timeout = setTimeout(() => {
      if (has_unsaved_changes) {
        handleSave()
      }
    }, 2000)

    return () => clearTimeout(timeout)
  }, [has_unsaved_changes, current_note_content, current_note_title])

  if (!current_note_id) {
    return (
      <div className="flex-1 flex items-center justify-center bg-gray-50 dark:bg-gray-800">
        <div className="text-center">
          <p className="text-gray-500 dark:text-gray-400 text-lg">
            Select a note to edit
          </p>
        </div>
      </div>
    )
  }

  return (
    <div className="flex-1 flex flex-col bg-white dark:bg-gray-900 h-full">
      {/* Header */}
      <div className="border-b border-gray-200 dark:border-gray-700 p-4 flex justify-between items-center">
        <input
          type="text"
          value={current_note_title}
          onChange={(e) => updateNoteTitle(e.target.value)}
          className="text-2xl font-bold bg-transparent text-gray-900 dark:text-white focus:outline-none flex-1"
          placeholder="Note title"
        />
        <div className="flex items-center gap-2">
          {has_unsaved_changes && (
            <span className="text-sm text-orange-600 dark:text-orange-400">
              Unsaved changes
            </span>
          )}
          <button
            onClick={() => useAppStore.setState({ show_preview: !show_preview })}
            className="px-3 py-1 bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 text-gray-900 dark:text-white rounded text-sm"
          >
            {show_preview ? 'Hide Preview' : 'Show Preview'}
          </button>
        </div>
      </div>

      {/* Editor and Preview */}
      <div className="flex-1 flex overflow-hidden">
        {/* Editor */}
        <div className="flex-1 flex flex-col border-r border-gray-200 dark:border-gray-700 overflow-hidden">
          <textarea
            value={current_note_content}
            onChange={(e) => updateNoteContent(e.target.value)}
            className="flex-1 p-4 bg-white dark:bg-gray-900 text-gray-900 dark:text-white resize-none focus:outline-none font-mono text-sm"
            placeholder="Write your note in Markdown..."
            spellCheck="true"
          />
        </div>

        {/* Preview */}
        {show_preview && (
          <div className="flex-1 overflow-auto border-l border-gray-200 dark:border-gray-700 p-4 bg-gray-50 dark:bg-gray-800">
            <div className="prose prose-sm dark:prose-invert max-w-none">
              <ReactMarkdown remarkPlugins={[remarkGfm]}>
                {current_note_content}
              </ReactMarkdown>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}
