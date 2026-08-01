import { describe, it, expect, vi } from "vitest";
import { render, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import NoteEditor from "../NoteEditor";

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
