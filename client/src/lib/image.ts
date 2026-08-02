/** Raised for any invalid image input; message is safe to show to the user. */
export class ImageInputError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "ImageInputError";
  }
}

const SUPPORTED_TYPES = new Set(["image/png", "image/jpeg", "image/webp", "image/gif"]);

const EXTENSION_MIME_TYPES: Record<string, string> = {
  png: "image/png",
  jpg: "image/jpeg",
  jpeg: "image/jpeg",
  webp: "image/webp",
  gif: "image/gif",
};

/** Best-effort mime type for a filename based on its extension, or null if unrecognized. */
export function mimeTypeForFilename(name: string): string | null {
  const ext = name.split(".").pop()?.toLowerCase();
  return ext ? (EXTENSION_MIME_TYPES[ext] ?? null) : null;
}

/** Identifies an image's mime type from its magic bytes. Needed for sources with no usable
 * filename, e.g. the content:// URIs Android's file picker returns. */
export function sniffImageMimeType(bytes: Uint8Array): string | null {
  if (bytes[0] === 0x89 && bytes[1] === 0x50 && bytes[2] === 0x4e && bytes[3] === 0x47) return "image/png";
  if (bytes[0] === 0xff && bytes[1] === 0xd8 && bytes[2] === 0xff) return "image/jpeg";
  if (bytes[0] === 0x47 && bytes[1] === 0x49 && bytes[2] === 0x46 && bytes[3] === 0x38) return "image/gif";
  if (
    bytes[0] === 0x52 && bytes[1] === 0x49 && bytes[2] === 0x46 && bytes[3] === 0x46 &&
    bytes[8] === 0x57 && bytes[9] === 0x45 && bytes[10] === 0x42 && bytes[11] === 0x50
  ) {
    return "image/webp";
  }
  return null;
}

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

// Notes are stored as a single encrypted blob (16MB server MEDIUMBLOB column limit), so the
// combined size of embedded images plus text needs to stay comfortably under that. A single
// image is capped well below the column limit so it can never dominate the budget on its own,
// and the note-wide budget is enforced separately since ciphertext size tracks plaintext size
// (AES-GCM adds only a 16-byte tag) with no extra base64 layer at rest.
const MAX_ORIGINAL_FILE_BYTES = 20 * 1024 * 1024;
const MAX_EMBEDDED_BYTES = 4 * 1024 * 1024;
const MAX_NOTE_IMAGE_BUDGET_BYTES = 12 * 1024 * 1024;
const MAX_IMAGE_DIMENSION = 1280;
const JPEG_QUALITY = 0.75;

export function isSupportedImageType(file: File): boolean {
  return SUPPORTED_TYPES.has(file.type);
}

export function readFileAsDataUrl(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result as string);
    reader.onerror = () => reject(new ImageInputError("Failed to read image file."));
    reader.readAsDataURL(file);
  });
}

function loadImageElement(dataUrl: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.onload = () => resolve(img);
    img.onerror = () => reject(new ImageInputError("Failed to decode image."));
    img.src = dataUrl;
  });
}

/** Returns the approximate decoded byte size of a base64 data URL. */
export function decodedByteSize(dataUrl: string): number {
  const base64 = dataUrl.slice(dataUrl.indexOf(",") + 1);
  return Math.floor((base64.length * 3) / 4);
}

/**
 * Downscales `dataUrl` through a canvas so its longest side is at most `maxDimension`.
 * GIFs are left untouched to preserve animation. Returns the original data URL unchanged
 * if it's already small enough or if canvas isn't available.
 */
async function resizeIfNeeded(dataUrl: string, mimeType: string, maxDimension: number): Promise<string> {
  const img = await loadImageElement(dataUrl);
  const { naturalWidth: width, naturalHeight: height } = img;

  if (mimeType === "image/gif" || (width <= maxDimension && height <= maxDimension)) {
    return dataUrl;
  }

  const scale = maxDimension / Math.max(width, height);
  const canvas = document.createElement("canvas");
  canvas.width = Math.round(width * scale);
  canvas.height = Math.round(height * scale);

  const ctx = canvas.getContext("2d");
  if (!ctx) return dataUrl;

  ctx.drawImage(img, 0, 0, canvas.width, canvas.height);

  const outputType = mimeType === "image/png" ? "image/png" : "image/jpeg";
  return canvas.toDataURL(outputType, JPEG_QUALITY);
}

/**
 * Validates, reads and downscales an image file, returning a data URL ready to embed in a note.
 * `existingImageBytes` is the combined decoded size of images already in the note; passing it
 * keeps the whole note within the server's size limit even when many images are added over time.
 */
export async function prepareImageForInsert(file: File, existingImageBytes = 0): Promise<string> {
  if (!isSupportedImageType(file)) {
    throw new ImageInputError("Unsupported image format. Use PNG, JPEG, WebP or GIF.");
  }
  if (file.size > MAX_ORIGINAL_FILE_BYTES) {
    throw new ImageInputError("Image is too large (max 20 MB).");
  }

  const original = await readFileAsDataUrl(file);
  const resized = await resizeIfNeeded(original, file.type, MAX_IMAGE_DIMENSION);
  const resizedBytes = decodedByteSize(resized);

  if (resizedBytes > MAX_EMBEDDED_BYTES) {
    throw new ImageInputError("Image is too large even after compression. Try a smaller image.");
  }
  if (existingImageBytes + resizedBytes > MAX_NOTE_IMAGE_BUDGET_BYTES) {
    throw new ImageInputError(
      "This note is close to its size limit. Remove or resize an existing image before adding more."
    );
  }

  return resized;
}
