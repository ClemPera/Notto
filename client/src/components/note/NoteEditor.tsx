import { useEditor, EditorContent } from "@tiptap/react";
import StarterKit from "@tiptap/starter-kit";
import { Markdown } from "@tiptap/markdown";
import type { EditorView } from "@tiptap/pm/view";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef } from "react";
import { prepareImageForInsert, ImageInputError, mimeTypeForFilename, fileUriToPath } from "../../lib/image";
import { ResizableImage } from "../../lib/resizableImage";
import { useToasts } from "../../store/toasts";
import { handleCommandError } from "../../lib/errors";
import "./NoteEditor.css";

/** Inserts `file` as an image node at `pos` by dispatching directly on the view (used by
 * paste/drop handlers, which only have access to the view, not the editor instance). */
async function insertImageAtPos(view: EditorView, file: File, pos: number) {
  try {
    const src = await prepareImageForInsert(file);
    if (view.isDestroyed) return;
    const node = view.state.schema.nodes.image.create({ src, alt: file.name });
    view.dispatch(view.state.tr.insert(pos, node));
  } catch (err) {
    const message = err instanceof ImageInputError ? err.message : "Failed to insert image.";
    useToasts.getState().addToast({ kind: "invalid_input", message });
  }
}

/** Same as insertImageAtPos, but for OS file drops where WebKitGTK gives us only a
 * `file://` path in dataTransfer, not a File — the bytes are read via a Tauri command. */
async function insertImageFromPathAtPos(view: EditorView, path: string, pos: number) {
  const mimeType = mimeTypeForFilename(path);
  if (!mimeType) {
    useToasts.getState().addToast({
      kind: "invalid_input",
      message: "Unsupported image format. Use PNG, JPEG, WebP or GIF.",
    });
    return;
  }

  try {
    const bytes = await invoke<number[]>("read_dropped_image", { path });
    if (view.isDestroyed) return;
    const name = path.split(/[/\\]/).pop() ?? "image";
    const file = new File([new Uint8Array(bytes)], name, { type: mimeType });
    const src = await prepareImageForInsert(file);
    if (view.isDestroyed) return;
    const node = view.state.schema.nodes.image.create({ src, alt: name });
    view.dispatch(view.state.tr.insert(pos, node));
  } catch (err) {
    if (err instanceof ImageInputError) {
      useToasts.getState().addToast({ kind: "invalid_input", message: err.message });
    } else {
      handleCommandError(err);
    }
  }
}

type Props = {
  noteId: string;
  content: string;
  onChange: (content: string) => void;
  disabled: boolean;
};

type ToolbarButtonProps = {
  onClick: () => void;
  active?: boolean;
  disabled?: boolean;
  title: string;
  children: React.ReactNode;
};

function ToolbarButton({ onClick, active, disabled, title, children }: ToolbarButtonProps) {
  return (
    <button
      onMouseDown={(e) => {
        e.preventDefault();
        onClick();
      }}
      disabled={disabled}
      title={title}
      className={`px-2 py-1 rounded text-xs font-medium transition-colors select-none ${
        active
          ? "bg-slate-600 text-white"
          : "text-slate-400 hover:text-white hover:bg-slate-700"
      } disabled:opacity-30 disabled:cursor-not-allowed`}
    >
      {children}
    </button>
  );
}

function Divider() {
  return <div className="w-px h-4 bg-slate-700 mx-0.5 shrink-0" />;
}

export default function NoteEditor({ noteId, content, onChange, disabled }: Props) {
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const isSwitchingRef = useRef(false);
  const isMountedRef = useRef(false);
  const lastContentRef = useRef(content);

  const fileInputRef = useRef<HTMLInputElement>(null);

  const editor = useEditor({
    extensions: [
      StarterKit,
      Markdown,
      ResizableImage.configure({
        inline: false,
        allowBase64: true,
        resize: { enabled: true, alwaysPreserveAspectRatio: true, minWidth: 60, minHeight: 60 },
      }),
    ],
    content,
    contentType: "markdown",
    editable: !disabled,
    onUpdate: ({ editor }) => {
      if (isSwitchingRef.current) return;
      const markdown = editor.getMarkdown();
      if (markdown === lastContentRef.current) return;

      if (debounceRef.current) clearTimeout(debounceRef.current);
      debounceRef.current = setTimeout(() => {
        lastContentRef.current = markdown;
        onChange(markdown);
      }, 400);
    },
    editorProps: {
      handlePaste: (view, event) => {
        if (!view.editable) return false;
        const file = Array.from(event.clipboardData?.items ?? [])
          .find((item) => item.type.startsWith("image/"))
          ?.getAsFile();
        if (!file) return false;

        event.preventDefault();
        insertImageAtPos(view, file, view.state.selection.from);
        return true;
      },
      handleDrop: (view, event) => {
        if (!view.editable) return false;
        const pos =
          view.posAtCoords({ left: event.clientX, top: event.clientY })?.pos ??
          view.state.selection.from;

        const file = Array.from(event.dataTransfer?.files ?? []).find((f) =>
          f.type.startsWith("image/")
        );
        if (file) {
          event.preventDefault();
          insertImageAtPos(view, file, pos);
          return true;
        }

        // WebKitGTK doesn't populate dataTransfer.files for OS drops, only a URI list.
        const uriList =
          event.dataTransfer?.getData("text/uri-list") || event.dataTransfer?.getData("text/plain");
        const path = uriList ? fileUriToPath(uriList.split("\n")[0]?.trim() ?? "") : null;
        if (!path) return false;

        event.preventDefault();
        insertImageFromPathAtPos(view, path, pos);
        return true;
      },
    },
  });

  const handleInsertImageClick = () => fileInputRef.current?.click();

  const handleFileSelected = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    e.target.value = "";
    if (!file || !editor) return;

    try {
      const src = await prepareImageForInsert(file);
      editor.chain().focus().setImage({ src, alt: file.name }).run();
    } catch (err) {
      const message = err instanceof ImageInputError ? err.message : "Failed to insert image.";
      useToasts.getState().addToast({ kind: "invalid_input", message });
    }
  };

  // Reset content when switching to a different note, skip on initial mount
  useEffect(() => {
    if (!isMountedRef.current) {
      isMountedRef.current = true;
      return;
    }
    if (!editor || editor.isDestroyed) return;
    isSwitchingRef.current = true;
    lastContentRef.current = content;
    if (debounceRef.current) clearTimeout(debounceRef.current);
    editor.commands.setContent(content, { emitUpdate: false, contentType: "markdown" });
    // onUpdate fires asynchronously, reset the flag after the current microtask queue
    Promise.resolve().then(() => { isSwitchingRef.current = false; });
  }, [noteId]);

  // Apply content update from server (e.g. live sync from another device)
  useEffect(() => {
    if (!editor || editor.isDestroyed) return;
    if (content === lastContentRef.current) return;
    isSwitchingRef.current = true;
    lastContentRef.current = content;
    if (debounceRef.current) clearTimeout(debounceRef.current);
    editor.commands.setContent(content, { emitUpdate: false, contentType: "markdown" });
    Promise.resolve().then(() => { isSwitchingRef.current = false; });
  }, [content]);

  // Sync disabled state
  useEffect(() => {
    if (!editor) return;
    editor.setEditable(!disabled);
  }, [disabled, editor]);

  useEffect(() => {
    return () => {
      if (debounceRef.current) clearTimeout(debounceRef.current);
    };
  }, []);

  const isDisabled = disabled || !editor;

  return (
    <div className="flex flex-col h-full">
      {/* Toolbar */}
      <div className="flex items-center gap-0.5 px-3 py-1.5 border-b border-slate-700 flex-wrap shrink-0">
        <ToolbarButton
          onClick={() => editor?.chain().focus().toggleBold().run()}
          active={editor?.isActive("bold")}
          disabled={isDisabled}
          title="Bold (Ctrl+B)"
        >
          B
        </ToolbarButton>
        <ToolbarButton
          onClick={() => editor?.chain().focus().toggleItalic().run()}
          active={editor?.isActive("italic")}
          disabled={isDisabled}
          title="Italic (Ctrl+I)"
        >
          <em>I</em>
        </ToolbarButton>
        <ToolbarButton
          onClick={() => editor?.chain().focus().toggleStrike().run()}
          active={editor?.isActive("strike")}
          disabled={isDisabled}
          title="Strikethrough"
        >
          <s>S</s>
        </ToolbarButton>
        <ToolbarButton
          onClick={() => editor?.chain().focus().toggleCode().run()}
          active={editor?.isActive("code")}
          disabled={isDisabled}
          title="Inline code"
        >
          {"<>"}
        </ToolbarButton>

        <Divider />

        <ToolbarButton
          onClick={() => editor?.chain().focus().toggleHeading({ level: 1 }).run()}
          active={editor?.isActive("heading", { level: 1 })}
          disabled={isDisabled}
          title="Heading 1"
        >
          H1
        </ToolbarButton>
        <ToolbarButton
          onClick={() => editor?.chain().focus().toggleHeading({ level: 2 }).run()}
          active={editor?.isActive("heading", { level: 2 })}
          disabled={isDisabled}
          title="Heading 2"
        >
          H2
        </ToolbarButton>
        <ToolbarButton
          onClick={() => editor?.chain().focus().toggleHeading({ level: 3 }).run()}
          active={editor?.isActive("heading", { level: 3 })}
          disabled={isDisabled}
          title="Heading 3"
        >
          H3
        </ToolbarButton>

        <Divider />

        <ToolbarButton
          onClick={() => editor?.chain().focus().toggleBulletList().run()}
          active={editor?.isActive("bulletList")}
          disabled={isDisabled}
          title="Bullet list"
        >
          <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
          </svg>
        </ToolbarButton>
        <ToolbarButton
          onClick={() => editor?.chain().focus().toggleOrderedList().run()}
          active={editor?.isActive("orderedList")}
          disabled={isDisabled}
          title="Ordered list"
        >
          <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 10h16M4 14h16M4 18h16" />
          </svg>
        </ToolbarButton>

        <Divider />

        <ToolbarButton
          onClick={() => editor?.chain().focus().toggleBlockquote().run()}
          active={editor?.isActive("blockquote")}
          disabled={isDisabled}
          title="Blockquote"
        >
          <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
          </svg>
        </ToolbarButton>
        <ToolbarButton
          onClick={() => editor?.chain().focus().toggleCodeBlock().run()}
          active={editor?.isActive("codeBlock")}
          disabled={isDisabled}
          title="Code block"
        >
          <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />
          </svg>
        </ToolbarButton>

        <Divider />

        <ToolbarButton onClick={handleInsertImageClick} disabled={isDisabled} title="Insert image">
          <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <rect x="3" y="3" width="18" height="18" rx="2" strokeWidth={2} />
            <circle cx="8.5" cy="8.5" r="1.5" strokeWidth={2} />
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 15l-5-5L5 21" />
          </svg>
        </ToolbarButton>
        <input
          ref={fileInputRef}
          type="file"
          accept="image/png,image/jpeg,image/webp,image/gif"
          onChange={handleFileSelected}
          className="hidden"
        />

        <Divider />

        <ToolbarButton
          onClick={() => editor?.chain().focus().undo().run()}
          disabled={isDisabled || !editor?.can().undo()}
          title="Undo (Ctrl+Z)"
        >
          <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 10h10a8 8 0 018 8v2M3 10l6 6m-6-6l6-6" />
          </svg>
        </ToolbarButton>
        <ToolbarButton
          onClick={() => editor?.chain().focus().redo().run()}
          disabled={isDisabled || !editor?.can().redo()}
          title="Redo (Ctrl+Y)"
        >
          <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 10H11a8 8 0 00-8 8v2m18-10l-6 6m6-6l-6-6" />
          </svg>
        </ToolbarButton>
      </div>

      {/* Editor content */}
      <EditorContent
        editor={editor}
        className="note-editor flex-1 overflow-y-auto overflow-x-hidden min-w-0 px-6 py-5"
      />
    </div>
  );
}
