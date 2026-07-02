/** Raised for any invalid image input; message is safe to show to the user. */
export class ImageInputError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "ImageInputError";
  }
}

const SUPPORTED_TYPES = new Set(["image/png", "image/jpeg", "image/webp", "image/gif"]);

// Notes are stored as a single encrypted blob (16MB server column limit), so embedded
// images need to stay well within budget alongside the note's text and other images.
const MAX_ORIGINAL_FILE_BYTES = 20 * 1024 * 1024;
const MAX_EMBEDDED_BYTES = 8 * 1024 * 1024;
const MAX_IMAGE_DIMENSION = 1600;
const JPEG_QUALITY = 0.85;

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
function decodedByteSize(dataUrl: string): number {
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

/** Validates, reads and downscales an image file, returning a data URL ready to embed in a note. */
export async function prepareImageForInsert(file: File): Promise<string> {
  if (!isSupportedImageType(file)) {
    throw new ImageInputError("Unsupported image format. Use PNG, JPEG, WebP or GIF.");
  }
  if (file.size > MAX_ORIGINAL_FILE_BYTES) {
    throw new ImageInputError("Image is too large (max 20 MB).");
  }

  const original = await readFileAsDataUrl(file);
  const resized = await resizeIfNeeded(original, file.type, MAX_IMAGE_DIMENSION);

  if (decodedByteSize(resized) > MAX_EMBEDDED_BYTES) {
    throw new ImageInputError("Image is too large even after compression. Try a smaller image.");
  }

  return resized;
}
