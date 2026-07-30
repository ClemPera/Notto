/**
 * Extracts a local filesystem path from a `file://` URI, or null if `uri` isn't one.
 * Handles the Windows case where `file:///C:/...` parses with a leading slash before the drive letter.
 */
export function fileUriToPath(uri: string): string | null {
  if (!uri.startsWith("file://")) return null;
  try {
    const decoded = decodeURIComponent(new URL(uri).pathname);
    return /^\/[A-Za-z]:\//.test(decoded) ? decoded.slice(1) : decoded;
  } catch {
    return null;
  }
}

/** Escapes text for safe use inside an HTML attribute or as element content. */
export function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

export function readFileAsDataUrl(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result as string);
    reader.onerror = () => reject(new Error(`Failed to read '${file.name}'.`));
    reader.readAsDataURL(file);
  });
}

/** Returns the approximate decoded byte size of a base64 data URL. */
export function decodedByteSize(dataUrl: string): number {
  const base64 = dataUrl.slice(dataUrl.indexOf(",") + 1);
  return Math.floor((base64.length * 3) / 4);
}

/** Strips the `data:<mime>;base64,` prefix and decodes the payload into raw bytes. */
export function dataUrlToBytes(dataUrl: string): Uint8Array {
  const base64 = dataUrl.slice(dataUrl.indexOf(",") + 1);
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
  return bytes;
}
