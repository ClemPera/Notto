import { describe, it, expect, vi, afterEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { openPath } from "@tauri-apps/plugin-opener";
import { openOrDownloadAttachment } from "../attachmentActions";

describe("openOrDownloadAttachment", () => {
  afterEach(() => {
    vi.restoreAllMocks();
    vi.clearAllMocks();
  });

  it("writes the decrypted bytes to a temp file and opens it", async () => {
    vi.mocked(invoke).mockResolvedValue("/tmp/nooto-attachments/abc/notes.pdf");

    await openOrDownloadAttachment("data:application/pdf;base64,SGVsbG8=", "notes.pdf");

    expect(invoke).toHaveBeenCalledWith("write_temp_attachment", {
      filename: "notes.pdf",
      data: Array.from(new TextEncoder().encode("Hello")),
    });
    expect(openPath).toHaveBeenCalledWith("/tmp/nooto-attachments/abc/notes.pdf");
  });

  it("falls back to a browser download when writing the temp file fails", async () => {
    vi.mocked(invoke).mockRejectedValue(new Error("write failed"));
    const clickSpy = vi.spyOn(HTMLAnchorElement.prototype, "click").mockImplementation(() => {});

    await openOrDownloadAttachment("data:application/pdf;base64,SGVsbG8=", "notes.pdf");

    expect(openPath).not.toHaveBeenCalled();
    expect(clickSpy).toHaveBeenCalledTimes(1);
  });

  it("falls back to a browser download when the OS can't open the file", async () => {
    vi.mocked(invoke).mockResolvedValue("/tmp/nooto-attachments/abc/notes.pdf");
    vi.mocked(openPath).mockRejectedValue(new Error("no app registered"));
    const clickSpy = vi.spyOn(HTMLAnchorElement.prototype, "click").mockImplementation(() => {});

    await openOrDownloadAttachment("data:application/pdf;base64,SGVsbG8=", "notes.pdf");

    expect(clickSpy).toHaveBeenCalledTimes(1);
  });

  it("shows a toast when both opening and downloading fail", async () => {
    vi.mocked(invoke).mockRejectedValue(new Error("write failed"));
    vi.spyOn(HTMLAnchorElement.prototype, "click").mockImplementation(() => {
      throw new Error("blocked");
    });
    const { useToasts } = await import("../../store/toasts");

    await openOrDownloadAttachment("data:application/pdf;base64,SGVsbG8=", "notes.pdf");

    const { toasts } = useToasts.getState();
    expect(toasts[toasts.length - 1]?.message).toBe("Couldn't open or download the attachment.");
  });
});
