import { describe, it, expect, vi, afterEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import NoteEditor from "../NoteEditor";

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

describe("NoteEditor image insertion", () => {
  const originalImage = globalThis.Image;

  afterEach(() => {
    globalThis.Image = originalImage;
    vi.restoreAllMocks();
  });

  it("inserts an image via the toolbar button and syncs it back as markdown", async () => {
    // @ts-expect-error test double for HTMLImageElement
    globalThis.Image = SmallFakeImage;
    const user = userEvent.setup();
    const onChange = vi.fn();

    const { container } = render(
      <NoteEditor noteId="note-1" content="" onChange={onChange} disabled={false} />
    );

    const fileInput = container.querySelector('input[type="file"]') as HTMLInputElement;
    expect(fileInput).toBeTruthy();

    await user.upload(fileInput, makeImageFile("photo.png"));

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

    const { container } = render(
      <NoteEditor noteId="note-4" content="" onChange={vi.fn()} disabled={false} />
    );

    const fileInput = container.querySelector('input[type="file"]') as HTMLInputElement;
    await user.upload(fileInput, makeImageFile("photo.png"));

    await waitFor(() => {
      expect(container.querySelectorAll("[data-resize-handle]").length).toBeGreaterThan(0);
    });
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
