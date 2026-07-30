import { Node, mergeAttributes } from "@tiptap/core";
import { escapeHtml } from "./file";
import { formatFileSize } from "./attachment";
import { openOrDownloadAttachment } from "./attachmentActions";

export type AttachmentAttrs = {
  href: string;
  filename: string;
  mimeType: string;
  size: number;
};

declare module "@tiptap/core" {
  interface Commands<ReturnType> {
    attachment: {
      setAttachment: (attrs: AttachmentAttrs) => ReturnType;
    };
  }
}

const FILE_ICON_SVG =
  '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" ' +
  'stroke-width="2" stroke-linecap="round" stroke-linejoin="round">' +
  '<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>' +
  '<path d="M14 2v6h6"/></svg>';

function buildChip(attrs: AttachmentAttrs): HTMLElement {
  const dom = document.createElement("span");
  dom.className = "note-attachment";
  dom.contentEditable = "false";
  dom.setAttribute("role", "button");
  dom.setAttribute("tabindex", "0");
  dom.setAttribute("title", `Open ${attrs.filename}`);

  const icon = document.createElement("span");
  icon.className = "note-attachment-icon";
  icon.innerHTML = FILE_ICON_SVG;

  const name = document.createElement("span");
  name.className = "note-attachment-name";
  name.textContent = attrs.filename;

  dom.append(icon, name);

  if (attrs.size) {
    const size = document.createElement("span");
    size.className = "note-attachment-size";
    size.textContent = formatFileSize(attrs.size);
    dom.append(size);
  }

  const open = () => void openOrDownloadAttachment(attrs.href, attrs.filename);
  dom.addEventListener("click", (event) => {
    event.preventDefault();
    open();
  });
  dom.addEventListener("keydown", (event) => {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      open();
    }
  });

  return dom;
}

/**
 * Embeds arbitrary files inline in a note as a clickable chip. Like images, the file is
 * stored as a base64 data URI in the markdown content, so it rides through the existing
 * note encryption and sync pipeline unchanged. Rendered as a raw `<a>` tag for markdown
 * round-tripping, but the node view below intercepts clicks instead of letting the webview
 * navigate to the data URI directly.
 */
export const Attachment = Node.create({
  name: "attachment",
  group: "inline",
  inline: true,
  atom: true,
  draggable: true,

  addAttributes() {
    return {
      href: { default: null },
      filename: { default: null, parseHTML: (el) => el.getAttribute("data-filename") },
      mimeType: { default: null, parseHTML: (el) => el.getAttribute("data-mime") },
      size: {
        default: 0,
        parseHTML: (el) => Number(el.getAttribute("data-size")) || 0,
      },
    };
  },

  parseHTML() {
    return [{ tag: "a[data-attachment]" }];
  },

  renderHTML({ HTMLAttributes, node }) {
    return [
      "a",
      mergeAttributes(HTMLAttributes, { "data-attachment": "" }),
      (node.attrs.filename as string) ?? "",
    ];
  },

  renderMarkdown(node) {
    const { href, filename, mimeType, size } = node.attrs as AttachmentAttrs;
    const name = escapeHtml(filename || "attachment");
    const attrs = [
      `href="${href}"`,
      `data-filename="${name}"`,
      mimeType ? `data-mime="${escapeHtml(mimeType)}"` : null,
      size ? `data-size="${size}"` : null,
    ]
      .filter(Boolean)
      .join(" ");
    return `<a data-attachment ${attrs}>${name}</a>`;
  },

  addNodeView() {
    return ({ node }) => ({ dom: buildChip(node.attrs as AttachmentAttrs) });
  },

  addCommands() {
    return {
      setAttachment:
        (attrs) =>
        ({ commands }) =>
          commands.insertContent({ type: this.name, attrs }),
    };
  },
});
