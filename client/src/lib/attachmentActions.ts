import { invoke } from "@tauri-apps/api/core";
import { openPath } from "@tauri-apps/plugin-opener";
import { dataUrlToBytes } from "./file";
import { useToasts } from "../store/toasts";

/**
 * Opens an embedded attachment with the OS's default app for its file type. The bytes are
 * written to a temp file first since `openPath` needs a filesystem path, not raw data; falls
 * back to a browser-style download if no app is registered for the file (or the write fails).
 */
export async function openOrDownloadAttachment(href: string, filename: string): Promise<void> {
  try {
    const bytes = dataUrlToBytes(href);
    const path = await invoke<string>("write_temp_attachment", {
      filename,
      data: Array.from(bytes),
    });
    await openPath(path);
  } catch {
    if (!downloadAttachment(href, filename)) {
      useToasts.getState().addToast({
        kind: "invalid_input",
        message: "Couldn't open or download the attachment.",
      });
    }
  }
}

function downloadAttachment(href: string, filename: string): boolean {
  try {
    const a = document.createElement("a");
    a.href = href;
    a.download = filename;
    a.click();
    return true;
  } catch {
    return false;
  }
}
