import { describe, it, expect } from "vitest";
import { sortNotes } from "../noteSort";
import { Note } from "../../types";

function makeNote(overrides: Partial<Note>): Note {
  return {
    id: "id",
    title: "title",
    parent_id: null,
    is_folder: false,
    folder_open: false,
    pinned: false,
    updated_at: new Date(0),
    deleted: false,
    ...overrides,
  };
}

describe("sortNotes", () => {
  it("orders unpinned items by most recently updated first", () => {
    const older = makeNote({ id: "older", updated_at: new Date(1000) });
    const newer = makeNote({ id: "newer", updated_at: new Date(2000) });

    const sorted = sortNotes([older, newer]);

    expect(sorted.map((n) => n.id)).toEqual(["newer", "older"]);
  });

  it("puts pinned items before unpinned items regardless of date", () => {
    const pinnedOld = makeNote({ id: "pinned-old", pinned: true, updated_at: new Date(500) });
    const unpinnedNew = makeNote({ id: "unpinned-new", pinned: false, updated_at: new Date(5000) });

    const sorted = sortNotes([unpinnedNew, pinnedOld]);

    expect(sorted.map((n) => n.id)).toEqual(["pinned-old", "unpinned-new"]);
  });

  it("mixes folders and notes together by date instead of grouping folders first", () => {
    const folder = makeNote({ id: "folder", is_folder: true, updated_at: new Date(1000) });
    const note = makeNote({ id: "note", is_folder: false, updated_at: new Date(2000) });

    const sorted = sortNotes([folder, note]);

    expect(sorted.map((n) => n.id)).toEqual(["note", "folder"]);
  });

  it("sorts pinned items among themselves by date, most recent first", () => {
    const pinnedOlder = makeNote({ id: "pinned-older", pinned: true, updated_at: new Date(1000) });
    const pinnedNewer = makeNote({ id: "pinned-newer", pinned: true, updated_at: new Date(2000) });

    const sorted = sortNotes([pinnedOlder, pinnedNewer]);

    expect(sorted.map((n) => n.id)).toEqual(["pinned-newer", "pinned-older"]);
  });

  it("does not mutate the input array", () => {
    const notes = [makeNote({ id: "a", updated_at: new Date(1000) }), makeNote({ id: "b", updated_at: new Date(2000) })];
    const original = [...notes];

    sortNotes(notes);

    expect(notes).toEqual(original);
  });
});
