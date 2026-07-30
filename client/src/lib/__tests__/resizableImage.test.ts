import { describe, it, expect } from "vitest";
import { Editor } from "@tiptap/core";
import StarterKit from "@tiptap/starter-kit";
import { Markdown } from "@tiptap/markdown";
import { ResizableImage } from "../resizableImage";

function makeEditor(content = "") {
  return new Editor({
    element: document.createElement("div"),
    extensions: [
      StarterKit,
      Markdown,
      ResizableImage.configure({ inline: false, allowBase64: true, resize: { enabled: true } }),
    ],
    content,
    contentType: "markdown",
  });
}

describe("ResizableImage markdown round-trip", () => {
  it("uses plain markdown syntax when no width/height is set", () => {
    const editor = makeEditor();
    editor.commands.insertContent({
      type: "image",
      attrs: { src: "data:image/png;base64,AAAA", alt: "plain" },
    });

    expect(editor.getMarkdown().trim()).toBe("![plain](data:image/png;base64,AAAA)");
  });

  it("serializes a resized image as a raw <img> tag with width/height", () => {
    const editor = makeEditor();
    editor.commands.insertContent({
      type: "image",
      attrs: { src: "data:image/png;base64,BBBB", alt: "resized", width: 400, height: 200 },
    });

    expect(editor.getMarkdown().trim()).toBe(
      '<img src="data:image/png;base64,BBBB" alt="resized" width="400" height="200" />'
    );
  });

  it("round-trips a resized image through markdown without losing its size", () => {
    const editor = makeEditor();
    editor.commands.insertContent({
      type: "image",
      attrs: { src: "data:image/png;base64,CCCC", alt: "note-image", width: 320, height: 180 },
    });

    const markdown = editor.getMarkdown();
    editor.commands.setContent(markdown, { contentType: "markdown", emitUpdate: false });

    const imageNode = editor.getJSON().content?.find((n) => n.type === "image");
    expect(imageNode?.attrs).toMatchObject({ width: 320, height: 180 });
    expect(editor.getMarkdown()).toBe(markdown);
  });
});
