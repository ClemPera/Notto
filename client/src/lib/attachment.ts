import { readFileAsDataUrl } from "./file";

/** Raised for any invalid attachment input; message is safe to show to the user. */
export class AttachmentInputError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "AttachmentInputError";
  }
}

export type PreparedAttachment = {
  href: string;
  filename: string;
  mimeType: string;
  size: number;
};

// Same budget as embedded images: notes are stored as a single encrypted blob (16MB
// server column limit), and attachments can't be compressed like images can.
const MAX_ATTACHMENT_BYTES = 8 * 1024 * 1024;

/** Validates and reads an arbitrary file, returning a data URL ready to embed in a note. */
export async function prepareAttachmentForInsert(file: File): Promise<PreparedAttachment> {
  if (file.size > MAX_ATTACHMENT_BYTES) {
    throw new AttachmentInputError("File is too large (max 8 MB).");
  }

  let href: string;
  try {
    href = await readFileAsDataUrl(file);
  } catch {
    throw new AttachmentInputError("Failed to read file.");
  }

  return {
    href,
    filename: file.name,
    mimeType: file.type || "application/octet-stream",
    size: file.size,
  };
}

/** Formats a byte count as a short human-readable size, e.g. "1.4 MB". */
export function formatFileSize(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes < 0) return "";
  if (bytes < 1024) return `${bytes} B`;
  const units = ["KB", "MB", "GB"];
  let value = bytes / 1024;
  let unitIndex = 0;
  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex += 1;
  }
  return `${value.toFixed(1)} ${units[unitIndex]}`;
}
