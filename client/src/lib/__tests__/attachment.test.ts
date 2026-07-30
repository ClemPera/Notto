import { describe, it, expect } from "vitest";
import { prepareAttachmentForInsert, AttachmentInputError, formatFileSize } from "../attachment";

function makeFile(name: string, type: string, sizeBytes = 100): File {
  return new File([new Uint8Array(sizeBytes)], name, { type });
}

describe("prepareAttachmentForInsert", () => {
  it("rejects files above the size limit", async () => {
    const file = makeFile("archive.zip", "application/zip");
    Object.defineProperty(file, "size", { value: 9 * 1024 * 1024 });
    await expect(prepareAttachmentForInsert(file)).rejects.toThrow(AttachmentInputError);
  });

  it("reads the file into a data URL and preserves its name, type and size", async () => {
    const file = makeFile("notes.pdf", "application/pdf", 42);
    const result = await prepareAttachmentForInsert(file);

    expect(result.href.startsWith("data:application/pdf;base64,")).toBe(true);
    expect(result.filename).toBe("notes.pdf");
    expect(result.mimeType).toBe("application/pdf");
    expect(result.size).toBe(42);
  });

  it("falls back to a generic mime type when the browser doesn't provide one", async () => {
    const file = makeFile("data.bin", "", 10);
    const result = await prepareAttachmentForInsert(file);
    expect(result.mimeType).toBe("application/octet-stream");
  });
});

describe("formatFileSize", () => {
  it("formats bytes, kilobytes, megabytes and gigabytes", () => {
    expect(formatFileSize(500)).toBe("500 B");
    expect(formatFileSize(2048)).toBe("2.0 KB");
    expect(formatFileSize(5 * 1024 * 1024)).toBe("5.0 MB");
    expect(formatFileSize(1.5 * 1024 * 1024 * 1024)).toBe("1.5 GB");
  });
});
