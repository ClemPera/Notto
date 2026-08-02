import { describe, it, expect, vi, afterEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
import { readFile } from "@tauri-apps/plugin-fs";
import { useToasts } from "../../../store/toasts";
import * as imageLib from "../../../lib/image";
import NoteEditor from "../NoteEditor";

/** Minimal PNG magic-byte prefix, enough for sniffImageMimeType to recognize it. */
function pngBytes(payloadSize = 40): Uint8Array {
  const bytes = new Uint8Array(8 + payloadSize);
  bytes.set([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);
  return bytes;
}

/** Stand-in for HTMLImageElement: jsdom doesn't decode images, so tests control dimensions directly. */
class SmallFakeImage {
  onload: (() => void) | null = null;
  onerror: (() => void) | null = null;
  naturalWidth = 200;
  naturalHeight = 150;
  private _src = "";

  set src(value: string) {
    this._src = value;
    queueMicrotask(() => this.onload?.());
  }

  get src() {
    return this._src;
  }
}

function makeImageFile(name = "photo.png") {
  return new File([new Uint8Array(50)], name, { type: "image/png" });
}

describe("NoteEditor paste handling", () => {
  it("parses plain-text clipboard content as markdown", async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();

    const { container } = render(
      <NoteEditor noteId="note-1" content="" onChange={onChange} disabled={false} />
    );

    const editable = container.querySelector(".ProseMirror") as HTMLElement;
    editable.focus();
    await user.paste("**bold** and _italic_");

    await waitFor(() => {
      expect(editable.querySelector("strong")?.textContent).toBe("bold");
    });
    expect(editable.querySelector("em")?.textContent).toBe("italic");
    expect(editable.textContent).not.toContain("**");
  });

  it("leaves rich HTML clipboard content to the default paste handling", async () => {
    const onChange = vi.fn();

    const { container } = render(
      <NoteEditor noteId="note-2" content="" onChange={onChange} disabled={false} />
    );

    const editable = container.querySelector(".ProseMirror") as HTMLElement;
    editable.focus();
    // jsdom has no DataTransfer constructor, stand in a minimal clipboardData for the paste event.
    const pasteEvent = new Event("paste", { bubbles: true, cancelable: true });
    Object.defineProperty(pasteEvent, "clipboardData", {
      value: {
        getData: (type: string) => {
          if (type === "text/html") return "<strong>already bold</strong>";
          if (type === "text/plain") return "already bold";
          return "";
        },
      },
    });
    editable.dispatchEvent(pasteEvent);

    await waitFor(() => {
      expect(editable.querySelector("strong")?.textContent).toBe("already bold");
    });
  });
});

describe("NoteEditor image insertion", () => {
  const originalImage = globalThis.Image;

  afterEach(() => {
    globalThis.Image = originalImage;
    vi.restoreAllMocks();
    vi.mocked(openFileDialog).mockReset();
    vi.mocked(readFile).mockReset();
  });

  it("inserts an image via the toolbar button (desktop path) and syncs it back as markdown", async () => {
    // @ts-expect-error test double for HTMLImageElement
    globalThis.Image = SmallFakeImage;
    const user = userEvent.setup();
    const onChange = vi.fn();
    vi.mocked(openFileDialog).mockResolvedValue("/home/clement/Pictures/photo.png");
    vi.mocked(readFile).mockResolvedValue(pngBytes());

    const { container } = render(
      <NoteEditor noteId="note-1" content="" onChange={onChange} disabled={false} />
    );

    await user.click(screen.getByTitle("Insert image"));

    await waitFor(() => {
      expect(container.querySelector(".ProseMirror img")).toBeTruthy();
    });

    const img = container.querySelector(".ProseMirror img") as HTMLImageElement;
    expect(img.getAttribute("src")).toMatch(/^data:image\/png;base64,/);
    expect(img.getAttribute("alt")).toBe("photo.png");

    await waitFor(
      () => {
        expect(onChange).toHaveBeenCalled();
      },
      { timeout: 1000 }
    );
    const lastMarkdown = onChange.mock.calls[onChange.mock.calls.length - 1][0] as string;
    expect(lastMarkdown).toMatch(/!\[photo\.png\]\(data:image\/png;base64,[^)]+\)/);
  });

  it("inserts an image via the toolbar button when the dialog returns an Android content:// URI", async () => {
    // @ts-expect-error test double for HTMLImageElement
    globalThis.Image = SmallFakeImage;
    const user = userEvent.setup();
    vi.mocked(openFileDialog).mockResolvedValue(
      "content://com.android.providers.media.documents/document/image%3A12345"
    );
    vi.mocked(readFile).mockResolvedValue(pngBytes());

    const { container } = render(
      <NoteEditor noteId="note-android" content="" onChange={vi.fn()} disabled={false} />
    );

    await user.click(screen.getByTitle("Insert image"));

    await waitFor(() => {
      expect(container.querySelector(".ProseMirror img")).toBeTruthy();
    });
    // No filename in the URI to key off, so mime type comes from sniffing the bytes.
    const img = container.querySelector(".ProseMirror img") as HTMLImageElement;
    expect(img.getAttribute("src")).toMatch(/^data:image\/png;base64,/);
  });

  it("does nothing when the file dialog is cancelled", async () => {
    const user = userEvent.setup();
    vi.mocked(openFileDialog).mockResolvedValue(null);

    const { container } = render(
      <NoteEditor noteId="note-cancel" content="" onChange={vi.fn()} disabled={false} />
    );

    await user.click(screen.getByTitle("Insert image"));

    expect(readFile).not.toHaveBeenCalled();
    expect(container.querySelector(".ProseMirror img")).toBeNull();
  });

  it("inserts an image at the cursor position on paste", async () => {
    // @ts-expect-error test double for HTMLImageElement
    globalThis.Image = SmallFakeImage;

    const onChange = vi.fn();
    const { container } = render(
      <NoteEditor noteId="note-2" content="hello" onChange={onChange} disabled={false} />
    );

    await waitFor(() => {
      expect(container.querySelector(".ProseMirror")).toBeTruthy();
    });
    const pm = container.querySelector(".ProseMirror") as HTMLElement;

    const file = makeImageFile("pasted.png");
    const clipboardData = {
      items: [{ type: "image/png", getAsFile: () => file }],
      files: [file],
      types: ["Files"],
      getData: () => "",
    };
    const pasteEvent = new Event("paste", { bubbles: true, cancelable: true });
    Object.defineProperty(pasteEvent, "clipboardData", { value: clipboardData });
    pm.dispatchEvent(pasteEvent);

    await waitFor(() => {
      expect(container.querySelector(".ProseMirror img")).toBeTruthy();
    });
    const img = container.querySelector(".ProseMirror img") as HTMLImageElement;
    expect(img.getAttribute("alt")).toBe("pasted.png");
  });

  it("disables the insert-image button when the editor is disabled", () => {
    render(<NoteEditor noteId="note-3" content="" onChange={vi.fn()} disabled={true} />);
    expect(screen.getByTitle("Insert image")).toBeDisabled();
  });

  it("renders resize handles on an inserted image", async () => {
    // @ts-expect-error test double for HTMLImageElement
    globalThis.Image = SmallFakeImage;
    const user = userEvent.setup();
    vi.mocked(openFileDialog).mockResolvedValue("/home/clement/Pictures/photo.png");
    vi.mocked(readFile).mockResolvedValue(pngBytes());

    const { container } = render(
      <NoteEditor noteId="note-4" content="" onChange={vi.fn()} disabled={false} />
    );

    await user.click(screen.getByTitle("Insert image"));

    await waitFor(() => {
      expect(container.querySelectorAll("[data-resize-handle]").length).toBeGreaterThan(0);
    });
  });

  it("commits an in-progress resize on touchend, since tiptap only listens for mouseup", async () => {
    render(<NoteEditor noteId="note-touch-resize" content="" onChange={vi.fn()} disabled={false} />);

    const marker = document.createElement("div");
    marker.setAttribute("data-resize-state", "true");
    document.body.appendChild(marker);

    const mouseupSpy = vi.fn();
    document.addEventListener("mouseup", mouseupSpy);
    document.dispatchEvent(new Event("touchend", { bubbles: true }));
    document.removeEventListener("mouseup", mouseupSpy);
    marker.remove();

    expect(mouseupSpy).toHaveBeenCalledTimes(1);
  });

  it("does not synthesize a mouseup on touchend when no resize is in progress", async () => {
    render(<NoteEditor noteId="note-no-resize" content="" onChange={vi.fn()} disabled={false} />);

    const mouseupSpy = vi.fn();
    document.addEventListener("mouseup", mouseupSpy);
    document.dispatchEvent(new Event("touchend", { bubbles: true }));
    document.removeEventListener("mouseup", mouseupSpy);

    expect(mouseupSpy).not.toHaveBeenCalled();
  });

  it("passes the note's existing embedded image size into prepareImageForInsert", async () => {
    // @ts-expect-error test double for HTMLImageElement
    globalThis.Image = SmallFakeImage;
    const user = userEvent.setup();
    const prepareSpy = vi.spyOn(imageLib, "prepareImageForInsert");
    vi.mocked(openFileDialog).mockResolvedValue("/home/clement/Pictures/a.png");
    vi.mocked(readFile).mockResolvedValue(pngBytes());

    // A note that already has one ~1KB embedded image.
    const existingImageMarkdown = `![a](data:image/png;base64,${"A".repeat(1200)})`;
    render(
      <NoteEditor noteId="note-budget" content={existingImageMarkdown} onChange={vi.fn()} disabled={false} />
    );

    await user.click(screen.getByTitle("Insert image"));

    await waitFor(() => expect(prepareSpy).toHaveBeenCalled());
    const [, existingImageBytes] = prepareSpy.mock.calls[prepareSpy.mock.calls.length - 1];
    expect(existingImageBytes).toBeGreaterThan(0);
  });

  it("shows an error and does not insert when the note is over its size budget", async () => {
    // @ts-expect-error test double for HTMLImageElement
    globalThis.Image = SmallFakeImage;
    const user = userEvent.setup();
    vi.spyOn(imageLib, "prepareImageForInsert").mockRejectedValue(
      new imageLib.ImageInputError("This note is close to its size limit.")
    );
    vi.mocked(openFileDialog).mockResolvedValue("/home/clement/Pictures/a.png");
    vi.mocked(readFile).mockResolvedValue(pngBytes());

    useToasts.setState({ toasts: [] });
    const { container } = render(
      <NoteEditor noteId="note-over-budget" content="" onChange={vi.fn()} disabled={false} />
    );

    await user.click(screen.getByTitle("Insert image"));

    await waitFor(() => {
      expect(useToasts.getState().toasts.some((t) => /size limit/.test(t.message))).toBe(true);
    });
    expect(container.querySelector(".ProseMirror img")).toBeNull();
  });

  it("reads the file from disk when a drop only carries a file:// path (WebKitGTK)", async () => {
    // @ts-expect-error test double for HTMLImageElement
    globalThis.Image = SmallFakeImage;
    vi.mocked(invoke).mockResolvedValue(Array.from(new Uint8Array([1, 2, 3, 4])));
    document.elementFromPoint = vi.fn(() => null);

    const { container } = render(
      <NoteEditor noteId="note-5" content="" onChange={vi.fn()} disabled={false} />
    );

    await waitFor(() => {
      expect(container.querySelector(".ProseMirror")).toBeTruthy();
    });
    const pm = container.querySelector(".ProseMirror") as HTMLElement;

    const dataTransfer = {
      files: [],
      getData: (type: string) =>
        type === "text/uri-list" ? "file:///home/clement/Downloads/3.png" : "",
    };
    const dropEvent = new Event("drop", { bubbles: true, cancelable: true });
    Object.defineProperty(dropEvent, "dataTransfer", { value: dataTransfer });
    Object.defineProperty(dropEvent, "clientX", { value: 0 });
    Object.defineProperty(dropEvent, "clientY", { value: 0 });
    pm.dispatchEvent(dropEvent);

    await waitFor(() => {
      expect(container.querySelector(".ProseMirror img")).toBeTruthy();
    });

    expect(invoke).toHaveBeenCalledWith("read_dropped_image", {
      path: "/home/clement/Downloads/3.png",
    });
    const img = container.querySelector(".ProseMirror img") as HTMLImageElement;
    expect(img.getAttribute("alt")).toBe("3.png");
  });
});
