import { useEditor, EditorContent } from "@tiptap/react";
import StarterKit from "@tiptap/starter-kit";
import { Markdown } from "@tiptap/markdown";
import type { EditorView } from "@tiptap/pm/view";
import type { Node as PMNode } from "@tiptap/pm/model";
import { invoke } from "@tauri-apps/api/core";
import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
import { readFile } from "@tauri-apps/plugin-fs";
import { useEffect, useRef } from "react";
import {
  prepareImageForInsert,
  ImageInputError,
  mimeTypeForFilename,
  fileUriToPath,
  sniffImageMimeType,
  decodedByteSize,
} from "../../lib/image";
import { ResizableImage } from "../../lib/resizableImage";
import { useToasts } from "../../store/toasts";
import { handleCommandError } from "../../lib/errors";
import "./NoteEditor.css";

/** Sums the decoded byte size of every embedded (base64) image already in the document. */
function totalEmbeddedImageBytes(doc: PMNode): number {
  let total = 0;
  doc.descendants((node) => {
    if (node.type.name === "image" && typeof node.attrs.src === "string" && node.attrs.src.startsWith("data:")) {
      total += decodedByteSize(node.attrs.src);
    }
  });
  return total;
}

/** Inserts `file` as an image node at `pos` by dispatching directly on the view (used by
 * paste/drop handlers, which only have access to the view, not the editor instance). */
async function insertImageAtPos(view: EditorView, file: File, pos: number) {
  try {
    const src = await prepareImageForInsert(file, totalEmbeddedImageBytes(view.state.doc));
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
    const src = await prepareImageForInsert(file, totalEmbeddedImageBytes(view.state.doc));
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

        const imageFile = Array.from(event.clipboardData?.items ?? [])
          .find((item) => item.type.startsWith("image/"))
          ?.getAsFile();
        if (imageFile) {
          event.preventDefault();
          insertImageAtPos(view, imageFile, view.state.selection.from);
          return true;
        }

        const clipboardData = event.clipboardData;
        if (!clipboardData || clipboardData.getData("text/html")) return false;
        const text = clipboardData.getData("text/plain");
        if (!text) return false;
        editor?.commands.insertContent(text, { contentType: "markdown" });
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

  // Uses the dialog + fs plugins (rather than a hidden <input type="file">) since the latter
  // doesn't reliably trigger Android's native picker in a Tauri webview. This also gives desktop
  // and Android a single code path: dialog returns a path on desktop and a content:// URI on
  // Android, and the fs plugin can read either.
  const handleInsertImageClick = async () => {
    if (!editor) return;
    const path = await openFileDialog({
      multiple: false,
      filters: [{ name: "Images", extensions: ["png", "jpg", "jpeg", "webp", "gif"] }],
    }).catch(() => null);
    if (!path || typeof path !== "string") return;

    try {
      const bytes = await readFile(path);
      const mimeType = sniffImageMimeType(bytes) ?? mimeTypeForFilename(path);
      if (!mimeType) {
        useToasts.getState().addToast({
          kind: "invalid_input",
          message: "Unsupported image format. Use PNG, JPEG, WebP or GIF.",
        });
        return;
      }
      const name = decodeURIComponent(path.split(/[/\\]/).pop() ?? "image");
      const file = new File([bytes], name, { type: mimeType });
      const src = await prepareImageForInsert(file, totalEmbeddedImageBytes(editor.state.doc));
      editor.chain().focus().setImage({ src, alt: name }).run();
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

  // @tiptap/core's ResizableNodeView only commits a resize on 'mouseup', never 'touchend', so a
  // touch-drag resize never persists on Android. Nudge it via the mouseup it already listens for,
  // but only while a resize is in progress so this can't affect unrelated touches.
  useEffect(() => {
    const commitTouchResize = () => {
      if (document.querySelector('[data-resize-state="true"]')) {
        document.dispatchEvent(new MouseEvent("mouseup"));
      }
    };
    document.addEventListener("touchend", commitTouchResize);
    document.addEventListener("touchcancel", commitTouchResize);
    return () => {
      document.removeEventListener("touchend", commitTouchResize);
      document.removeEventListener("touchcancel", commitTouchResize);
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
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8.25 6.75H20.25M8.25 12H20.25M8.25 17.25H20.25M3.75 6.75H3.7575V6.7575H3.75V6.75ZM4.125 6.75C4.125 6.95711 3.95711 7.125 3.75 7.125C3.54289 7.125 3.375 6.95711 3.375 6.75C3.375 6.54289 3.54289 6.375 3.75 6.375C3.95711 6.375 4.125 6.54289 4.125 6.75ZM3.75 12H3.7575V12.0075H3.75V12ZM4.125 12C4.125 12.2071 3.95711 12.375 3.75 12.375C3.54289 12.375 3.375 12.2071 3.375 12C3.375 11.7929 3.54289 11.625 3.75 11.625C3.95711 11.625 4.125 11.7929 4.125 12ZM3.75 17.25H3.7575V17.2575H3.75V17.25ZM4.125 17.25C4.125 17.4571 3.95711 17.625 3.75 17.625C3.54289 17.625 3.375 17.4571 3.375 17.25C3.375 17.0429 3.54289 16.875 3.75 16.875C3.95711 16.875 4.125 17.0429 4.125 17.25Z" />
          </svg>
        </ToolbarButton>
        <ToolbarButton
          onClick={() => editor?.chain().focus().toggleOrderedList().run()}
          active={editor?.isActive("orderedList")}
          disabled={isDisabled}
          title="Ordered list"
        >
          <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8.24185 5.99179H20.2416M8.24118 11.9945H20.2409M8.24185 17.9936H20.2416M4.1157 7.49548V3.74512H2.99072M4.1157 7.49548H2.99072M4.1157 7.49548H5.24068M3.32128 10.0715C3.76061 9.63214 4.4729 9.63214 4.91223 10.0715C5.35157 10.5109 5.35157 11.2233 4.91223 11.6627L3.08285 13.4923L5.24182 13.4925M2.99072 15.7446H4.1156C4.73696 15.7446 5.24068 16.2484 5.24068 16.8697C5.24068 17.4911 4.73696 17.9949 4.1156 17.9949H3.74071M3.74071 17.9928H4.1156C4.73696 17.9928 5.24068 18.4966 5.24068 19.1179C5.24068 19.7393 4.73696 20.243 4.1156 20.243H2.99072" />
          </svg>
        </ToolbarButton>
        <ToolbarButton
          onClick={() => editor?.chain().focus().liftListItem("listItem").run()}
          disabled={isDisabled || !editor?.can().liftListItem("listItem")}
          title="Outdent list item"
        >
          <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M20 6l-7 0" />
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M20 12l-9 0" />
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M20 18l-7 0" />
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 8l-4 4l4 4" />
          </svg>
        </ToolbarButton>
        <ToolbarButton
          onClick={() => editor?.chain().focus().sinkListItem("listItem").run()}
          disabled={isDisabled || !editor?.can().sinkListItem("listItem")}
          title="Indent list item"
        >
          <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M20 6l-11 0" />
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M20 12l-7 0" />
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M20 18l-11 0" />
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 8l4 4l-4 4" />
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
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M16 3a2 2 0 0 0-2 2v6a2 2 0 0 0 2 2 1 1 0 0 1 1 1v1a2 2 0 0 1-2 2 1 1 0 0 0-1 1v2a1 1 0 0 0 1 1 6 6 0 0 0 6-6V5a2 2 0 0 0-2-2z" />
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 3a2 2 0 0 0-2 2v6a2 2 0 0 0 2 2 1 1 0 0 1 1 1v1a2 2 0 0 1-2 2 1 1 0 0 0-1 1v2a1 1 0 0 0 1 1 6 6 0 0 0 6-6V5a2 2 0 0 0-2-2z" />
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
