import { describe, it, expect, vi, afterEach } from "vitest";
import { render, screen, waitFor, act } from "@testing-library/react";
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

    await waitFor(() => {
      expect(container.querySelector(".ProseMirror")).toBeTruthy();
    });
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
    act(() => {
      editable.dispatchEvent(pasteEvent);
    });

    await waitFor(() => {
      expect(editable.querySelector("strong")?.textContent).toBe("already bold");
    });
  });
});

describe("NoteEditor image insertion", () => {
  const originalImage = globalThis.Image;

  /** Routes the shared `invoke` mock by command name, since a single test can trigger both
   * `read_dropped_image` and `insert_image` (and the paste/toolbar paths only the latter). */
  function stubImageCommands(uuid = "fake-image-uuid") {
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "insert_image") return uuid;
      if (cmd === "read_dropped_image") return Array.from(pngBytes());
      throw new Error(`unexpected invoke: ${cmd}`);
    });
  }

  afterEach(() => {
    globalThis.Image = originalImage;
    vi.restoreAllMocks();
    vi.mocked(openFileDialog).mockReset();
    vi.mocked(readFile).mockReset();
    vi.mocked(invoke).mockReset();
  });

  it("inserts an image via the toolbar button (desktop path) and syncs it back as markdown", async () => {
    // @ts-expect-error test double for HTMLImageElement
    globalThis.Image = SmallFakeImage;
    const user = userEvent.setup();
    const onChange = vi.fn();
    stubImageCommands("uuid-desktop");
    vi.mocked(openFileDialog).mockResolvedValue("/home/clement/Pictures/photo.png");
    vi.mocked(readFile).mockResolvedValue(pngBytes());

    const { container } = render(
      <NoteEditor noteId="note-1" content="" onChange={onChange} disabled={false} />
    );

    await user.click(screen.getByTitle("Insert image"));

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("insert_image", {
        note_id: "note-1",
        bytes: expect.any(Array),
      });
    });

    await waitFor(() => {
      const img = container.querySelector(".ProseMirror img") as HTMLImageElement | null;
      expect(img?.getAttribute("src")).toMatch(/^blob:/);
    });
    const img = container.querySelector(".ProseMirror img") as HTMLImageElement;
    expect(img.getAttribute("alt")).toBe("photo.png");

    await waitFor(
      () => {
        expect(onChange).toHaveBeenCalled();
      },
      { timeout: 1000 }
    );
    const lastMarkdown = onChange.mock.calls[onChange.mock.calls.length - 1][0] as string;
    expect(lastMarkdown).toMatch(/!\[photo\.png\]\(nooto-image:uuid-desktop\)/);
  });

  it("inserts an image via the toolbar button when the dialog returns an Android content:// URI", async () => {
    // @ts-expect-error test double for HTMLImageElement
    globalThis.Image = SmallFakeImage;
    const user = userEvent.setup();
    stubImageCommands();
    vi.mocked(openFileDialog).mockResolvedValue(
      "content://com.android.providers.media.documents/document/image%3A12345"
    );
    vi.mocked(readFile).mockResolvedValue(pngBytes());

    const { container } = render(
      <NoteEditor noteId="note-android" content="" onChange={vi.fn()} disabled={false} />
    );

    await user.click(screen.getByTitle("Insert image"));

    await waitFor(() => {
      // No filename in the URI to key off, so mime type comes from sniffing the bytes.
      const img = container.querySelector(".ProseMirror img") as HTMLImageElement | null;
      expect(img?.getAttribute("src")).toMatch(/^blob:/);
    });
    expect(invoke).toHaveBeenCalledWith("insert_image", {
      note_id: "note-android",
      bytes: expect.any(Array),
    });
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
    stubImageCommands();

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
      expect(invoke).toHaveBeenCalledWith("insert_image", {
        note_id: "note-2",
        bytes: expect.any(Array),
      });
    });
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
    stubImageCommands();
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

  it("shows an error and does not insert when prepareImageForInsert rejects", async () => {
    // @ts-expect-error test double for HTMLImageElement
    globalThis.Image = SmallFakeImage;
    const user = userEvent.setup();
    vi.spyOn(imageLib, "prepareImageForInsert").mockRejectedValue(
      new imageLib.ImageInputError("Image is too large even after compression.")
    );
    vi.mocked(openFileDialog).mockResolvedValue("/home/clement/Pictures/a.png");
    vi.mocked(readFile).mockResolvedValue(pngBytes());

    useToasts.setState({ toasts: [] });
    const { container } = render(
      <NoteEditor noteId="note-too-large" content="" onChange={vi.fn()} disabled={false} />
    );

    await user.click(screen.getByTitle("Insert image"));

    await waitFor(() => {
      expect(useToasts.getState().toasts.some((t) => /too large/.test(t.message))).toBe(true);
    });
    expect(invoke).not.toHaveBeenCalledWith("insert_image", expect.anything());
    expect(container.querySelector(".ProseMirror img")).toBeNull();
  });

  it("shows an error and does not insert an image node when insert_image fails", async () => {
    // @ts-expect-error test double for HTMLImageElement
    globalThis.Image = SmallFakeImage;
    const user = userEvent.setup();
    vi.mocked(invoke).mockRejectedValue(new Error("No workspace is loaded"));
    vi.mocked(openFileDialog).mockResolvedValue("/home/clement/Pictures/a.png");
    vi.mocked(readFile).mockResolvedValue(pngBytes());

    useToasts.setState({ toasts: [] });
    const { container } = render(
      <NoteEditor noteId="note-insert-fails" content="" onChange={vi.fn()} disabled={false} />
    );

    await user.click(screen.getByTitle("Insert image"));

    await waitFor(() => {
      expect(useToasts.getState().toasts.some((t) => /Failed to insert image/.test(t.message))).toBe(
        true
      );
    });
    expect(container.querySelector(".ProseMirror img")).toBeNull();
  });

  it("reads the file from disk when a drop only carries a file:// path (WebKitGTK)", async () => {
    // @ts-expect-error test double for HTMLImageElement
    globalThis.Image = SmallFakeImage;
    stubImageCommands();
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
    expect(invoke).toHaveBeenCalledWith("insert_image", {
      note_id: "note-5",
      bytes: expect.any(Array),
    });
    const img = container.querySelector(".ProseMirror img") as HTMLImageElement;
    expect(img.getAttribute("alt")).toBe("3.png");
  });
});

describe("NoteEditor list indent/outdent buttons", () => {
  it("indent is disabled on the first item of a list, which has no sibling to nest under", () => {
    const onChange = vi.fn();
    const { getByTitle } = render(
      <NoteEditor noteId="note-3" content={"- first\n- second"} onChange={onChange} disabled={false} />
    );

    expect(getByTitle("Indent list item")).toBeDisabled();
  });

  it("outdent lifts a top-level item out of the list into a plain paragraph", async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();

    const { container, getByTitle } = render(
      <NoteEditor noteId="note-4" content={"- only item"} onChange={onChange} disabled={false} />
    );

    const editable = container.querySelector(".ProseMirror") as HTMLElement;
    expect(getByTitle("Outdent list item")).not.toBeDisabled();
    await user.click(getByTitle("Outdent list item"));

    await waitFor(() => {
      expect(editable.querySelector("li")).toBeNull();
      expect(editable.textContent).toContain("only item");
    });
  });
});

describe("NoteEditor attachments list", () => {
  afterEach(() => {
    vi.restoreAllMocks();
    vi.mocked(invoke).mockReset();
  });

  function stubImageCommands({
    uuid = "fake-image-uuid",
    deleteImpl,
  }: { uuid?: string; deleteImpl?: () => Promise<void> } = {}) {
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "insert_image") return uuid;
      if (cmd === "get_image") return Array.from(pngBytes());
      if (cmd === "delete_image") return deleteImpl ? deleteImpl() : undefined;
      throw new Error(`unexpected invoke: ${cmd}`);
    });
  }

  it("shows no attachments bar for a note with no images", async () => {
    stubImageCommands();
    render(<NoteEditor noteId="note-attach-empty" content="just text" onChange={vi.fn()} disabled={false} />);

    await waitFor(() => {
      expect(screen.queryByTitle("Remove attachment")).toBeNull();
    });
  });

  it("lists an image already present in the note's content on load", async () => {
    stubImageCommands({ uuid: "existing-uuid" });
    render(
      <NoteEditor
        noteId="note-attach-existing"
        content="![a photo](nooto-image:existing-uuid)"
        onChange={vi.fn()}
        disabled={false}
      />
    );

    await waitFor(() => {
      expect(screen.getByText("1 attachment")).toBeTruthy();
    });
  });

  it("adds a newly inserted image to the attachments list", async () => {
    // @ts-expect-error test double for HTMLImageElement
    globalThis.Image = SmallFakeImage;
    const user = userEvent.setup();
    stubImageCommands({ uuid: "new-uuid" });
    vi.mocked(openFileDialog).mockResolvedValue("/home/clement/Pictures/photo.png");
    vi.mocked(readFile).mockResolvedValue(pngBytes());

    render(<NoteEditor noteId="note-attach-add" content="" onChange={vi.fn()} disabled={false} />);

    await user.click(screen.getByTitle("Insert image"));

    await waitFor(() => {
      expect(screen.getByText("1 attachment")).toBeTruthy();
    });
  });

  it("updates the attachments list when switching to a different note", async () => {
    stubImageCommands({ uuid: "note-a-uuid" });
    const { rerender } = render(
      <NoteEditor
        noteId="note-a"
        content="![a](nooto-image:note-a-uuid)"
        onChange={vi.fn()}
        disabled={false}
      />
    );

    await waitFor(() => {
      expect(screen.getByText("1 attachment")).toBeTruthy();
    });

    rerender(<NoteEditor noteId="note-b" content="no images here" onChange={vi.fn()} disabled={false} />);

    await waitFor(() => {
      expect(screen.queryByTitle("Remove attachment")).toBeNull();
    });
  });

  it("deletes the stored image first, then removes it from the note and the list", async () => {
    // @ts-expect-error test double for HTMLImageElement
    globalThis.Image = SmallFakeImage;
    const user = userEvent.setup();
    const onChange = vi.fn();
    stubImageCommands({ uuid: "to-delete-uuid" });
    vi.mocked(openFileDialog).mockResolvedValue("/home/clement/Pictures/photo.png");
    vi.mocked(readFile).mockResolvedValue(pngBytes());

    const { container } = render(
      <NoteEditor noteId="note-attach-delete" content="" onChange={onChange} disabled={false} />
    );

    await user.click(screen.getByTitle("Insert image"));
    await waitFor(() => {
      expect(container.querySelector(".ProseMirror img")).toBeTruthy();
    });

    await user.click(screen.getByTitle("Remove attachment"));

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("delete_image", { uuid: "to-delete-uuid" });
    });
    await waitFor(() => {
      expect(container.querySelector(".ProseMirror img")).toBeNull();
      expect(screen.queryByTitle("Remove attachment")).toBeNull();
    });

    await waitFor(
      () => {
        const lastMarkdown = onChange.mock.calls[onChange.mock.calls.length - 1]?.[0] as
          | string
          | undefined;
        expect(lastMarkdown).not.toMatch(/nooto-image:/);
      },
      { timeout: 1000 }
    );
  });

  it("keeps the attachment listed and shows an error when delete_image fails", async () => {
    // @ts-expect-error test double for HTMLImageElement
    globalThis.Image = SmallFakeImage;
    const user = userEvent.setup();
    stubImageCommands({
      uuid: "stubborn-uuid",
      deleteImpl: () => Promise.reject(new Error("Could not reach the server")),
    });
    vi.mocked(openFileDialog).mockResolvedValue("/home/clement/Pictures/photo.png");
    vi.mocked(readFile).mockResolvedValue(pngBytes());

    const { container } = render(
      <NoteEditor noteId="note-attach-delete-fail" content="" onChange={vi.fn()} disabled={false} />
    );

    await user.click(screen.getByTitle("Insert image"));
    await waitFor(() => {
      expect(container.querySelector(".ProseMirror img")).toBeTruthy();
    });

    useToasts.setState({ toasts: [] });
    await user.click(screen.getByTitle("Remove attachment"));

    await waitFor(() => {
      expect(useToasts.getState().toasts.length).toBeGreaterThan(0);
    });
    // The image stays, both in the note and in the list, so the user can retry.
    expect(container.querySelector(".ProseMirror img")).toBeTruthy();
    expect(screen.getByTitle("Remove attachment")).toBeTruthy();
  });
});
