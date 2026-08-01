import { describe, it, expect, vi } from "vitest";
import { render, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import NoteEditor from "../NoteEditor";

describe("NoteEditor list indent/outdent buttons", () => {
  it("indent is disabled on the first item of a list, which has no sibling to nest under", () => {
    const onChange = vi.fn();
    const { getByTitle } = render(
      <NoteEditor noteId="note-1" content={"- first\n- second"} onChange={onChange} disabled={false} />
    );

    expect(getByTitle("Indent list item")).toBeDisabled();
  });

  it("outdent lifts a top-level item out of the list into a plain paragraph", async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();

    const { container, getByTitle } = render(
      <NoteEditor noteId="note-2" content={"- only item"} onChange={onChange} disabled={false} />
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
