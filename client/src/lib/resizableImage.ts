import Image from "@tiptap/extension-image";

/**
 * Plain markdown image syntax (`![alt](src)`) has no width/height field, so the stock
 * renderMarkdown drops any resize the user did. Here it falls back to a raw `<img>` tag
 * when width or height is set; parseMarkdown already handles that via generic HTML
 * parsing, so this stays a lossless round-trip.
 */
export const ResizableImage = Image.extend({
  renderMarkdown(node) {
    const { src, alt, title, width, height } = node.attrs as {
      src: string;
      alt: string | null;
      title: string | null;
      width: number | null;
      height: number | null;
    };

    if (!width && !height) {
      return title ? `![${alt ?? ""}](${src} "${title}")` : `![${alt ?? ""}](${src})`;
    }

    const attrs = [
      `src="${src}"`,
      alt ? `alt="${alt}"` : null,
      title ? `title="${title}"` : null,
      width ? `width="${width}"` : null,
      height ? `height="${height}"` : null,
    ]
      .filter(Boolean)
      .join(" ");
    return `<img ${attrs} />`;
  },
});
