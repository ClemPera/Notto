import { Note } from "../../types";
import { sortNotes } from "../../lib/noteSort";
import Icon from "../icons/Icon";

type Props = {
  folderId: string;
  notes: Note[];
  onSelect: (id: string) => void;
  onCreateNote: (parentId: string | null) => void;
  onCreateFolder: (parentId: string | null) => void;
};

export default function FolderView({ folderId, notes, onSelect, onCreateNote, onCreateFolder }: Props) {
  const children = sortNotes(notes.filter((n) => n.parent_id === folderId && !n.deleted));

  const toolbar = (
    <div className="flex gap-2 shrink-0">
      <button
        onClick={() => onCreateNote(folderId)}
        className="flex items-center gap-1.5 px-3 py-1.5 bg-blue-600 hover:bg-blue-700 rounded-lg text-sm text-white transition-colors"
        title="New Note"
      >
        <Icon name="plus" className="w-4 h-4" />
        New note
      </button>
      <button
        onClick={() => onCreateFolder(folderId)}
        className="flex items-center gap-1.5 px-3 py-1.5 bg-slate-700 hover:bg-slate-600 rounded-lg text-sm text-white transition-colors"
        title="New Folder"
      >
        <Icon name="folder" className="w-4 h-4" />
        New folder
      </button>
    </div>
  );

  if (children.length === 0) {
    return (
      <div className="flex flex-col h-full">
        <div className="p-3 md:p-6 pb-0 flex justify-end">{toolbar}</div>
        <div className="flex-1 flex flex-col items-center justify-center text-slate-500 opacity-40 select-none p-6">
          <div className="p-8 bg-slate-800/50 rounded-full mb-6">
            <Icon name="folder" className="w-16 h-16" strokeWidth={1} />
          </div>
          <p className="text-sm text-center max-w-xs">This folder is empty.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full overflow-y-auto">
      <div className="flex justify-end p-3 md:p-6 pb-0">{toolbar}</div>
      <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-3 content-start p-3 md:p-6">
        {children.map((child) => (
          <button
            key={child.id}
            onClick={() => onSelect(child.id)}
            className="flex flex-col items-start gap-2 p-3 bg-slate-800 hover:bg-slate-700 rounded-xl border border-slate-700 hover:border-slate-600 transition-all text-left group"
          >
            <Icon
              name={child.is_folder ? "folder" : "document"}
              className={`w-6 h-6 ${child.is_folder ? "text-blue-400" : "text-slate-400"}`}
              strokeWidth={1.5}
            />
            <span className="flex items-center gap-1.5 w-full min-w-0">
              {child.pinned && <Icon name="pin" className="w-3 h-3 shrink-0 text-amber-400" />}
              <span className="text-sm font-medium text-slate-300 group-hover:text-white transition-colors truncate">
                {child.title}
              </span>
            </span>
          </button>
        ))}
      </div>
    </div>
  );
}
