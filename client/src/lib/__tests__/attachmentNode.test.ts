import { describe, it, expect, vi } from "vitest";
import { Editor } from "@tiptap/core";
import StarterKit from "@tiptap/starter-kit";
import { Markdown } from "@tiptap/markdown";
import { Attachment } from "../attachmentNode";
import { openOrDownloadAttachment } from "../attachmentActions";

vi.mock("../attachmentActions", () => ({ openOrDownloadAttachment: vi.fn() }));

function makeEditor(content = "") {
  return new Editor({
    element: document.createElement("div"),
    extensions: [StarterKit, Markdown, Attachment],
    content,
    contentType: "markdown",
  });
}

describe("Attachment node markdown round-trip", () => {
  it("serializes to a raw HTML anchor and parses back to the same node", () => {
    const editor = makeEditor();
    editor.commands.setAttachment({
      href: "data:application/pdf;base64,AAAA",
      filename: "report.pdf",
      mimeType: "application/pdf",
      size: 2048,
    });

    const markdown = editor.getMarkdown();
    editor.commands.setContent(markdown, { contentType: "markdown", emitUpdate: false });

    const node = editor.state.doc.content.firstChild?.firstChild;
    expect(node?.type.name).toBe("attachment");
    expect(node?.attrs).toEqual({
      href: "data:application/pdf;base64,AAAA",
      filename: "report.pdf",
      mimeType: "application/pdf",
      size: 2048,
    });
    expect(editor.getMarkdown()).toBe(markdown);
  });

  it("escapes filenames with quotes and angle brackets", () => {
    const editor = makeEditor();
    editor.commands.setAttachment({
      href: "data:text/plain;base64,AAAA",
      filename: 'weird "name" <here>.txt',
      mimeType: "text/plain",
      size: 5,
    });

    const markdown = editor.getMarkdown();
    editor.commands.setContent(markdown, { contentType: "markdown", emitUpdate: false });

    const node = editor.state.doc.content.firstChild?.firstChild;
    expect(node?.attrs.filename).toBe('weird "name" <here>.txt');
    expect(editor.getMarkdown()).toBe(markdown);
  });

  it("does not collide with the Link mark, which rejects data: URIs", () => {
    const editor = makeEditor();
    editor.commands.setAttachment({
      href: "data:application/zip;base64,AAAA",
      filename: "archive.zip",
      mimeType: "application/zip",
      size: 100,
    });
    const markdown = editor.getMarkdown();
    editor.commands.setContent(markdown, { contentType: "markdown", emitUpdate: false });

    expect(editor.isActive("link")).toBe(false);
    expect(editor.state.doc.content.firstChild?.firstChild?.type.name).toBe("attachment");
  });
});

describe("Attachment node view", () => {
  it("renders a chip with the filename and size, and opens on click", () => {
    const editor = makeEditor();
    editor.commands.setAttachment({
      href: "data:application/pdf;base64,AAAA",
      filename: "report.pdf",
      mimeType: "application/pdf",
      size: 2048,
    });

    const chip = editor.view.dom.querySelector(".note-attachment") as HTMLElement;
    expect(chip).toBeTruthy();
    expect(chip.querySelector(".note-attachment-name")?.textContent).toBe("report.pdf");
    expect(chip.querySelector(".note-attachment-size")?.textContent).toBe("2.0 KB");

    chip.click();
    expect(openOrDownloadAttachment).toHaveBeenCalledWith(
      "data:application/pdf;base64,AAAA",
      "report.pdf"
    );
  });

  it("omits the size badge when size is unknown", () => {
    const editor = makeEditor();
    editor.commands.setAttachment({
      href: "data:application/pdf;base64,AAAA",
      filename: "report.pdf",
      mimeType: "application/pdf",
      size: 0,
    });

    const chip = editor.view.dom.querySelector(".note-attachment") as HTMLElement;
    expect(chip.querySelector(".note-attachment-size")).toBeNull();
  });
});
