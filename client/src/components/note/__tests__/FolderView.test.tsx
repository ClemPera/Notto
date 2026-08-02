import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import FolderView from "../FolderView";
import { Note } from "../../../types";

function makeNote(overrides: Partial<Note> = {}): Note {
  return {
    id: "note-1",
    title: "Note",
    parent_id: "folder-1",
    is_folder: false,
    deleted: false,
    pinned: false,
    folder_open: false,
    updated_at: new Date(),
    ...overrides,
  };
}

describe("FolderView create buttons", () => {
  it("creates a note scoped to the open folder, including when empty", async () => {
    const user = userEvent.setup();
    const onCreateNote = vi.fn();
    const onCreateFolder = vi.fn();

    render(
      <FolderView
        folderId="folder-1"
        notes={[]}
        onSelect={vi.fn()}
        onCreateNote={onCreateNote}
        onCreateFolder={onCreateFolder}
      />
    );

    await user.click(screen.getByTitle("New Note"));
    expect(onCreateNote).toHaveBeenCalledWith("folder-1");

    await user.click(screen.getByTitle("New Folder"));
    expect(onCreateFolder).toHaveBeenCalledWith("folder-1");
  });

  it("still shows the create buttons when the folder has children", async () => {
    const user = userEvent.setup();
    const onCreateNote = vi.fn();

    render(
      <FolderView
        folderId="folder-1"
        notes={[makeNote()]}
        onSelect={vi.fn()}
        onCreateNote={onCreateNote}
        onCreateFolder={vi.fn()}
      />
    );

    expect(screen.getByText("Note")).toBeInTheDocument();
    await user.click(screen.getByTitle("New Note"));
    expect(onCreateNote).toHaveBeenCalledWith("folder-1");
  });
});
