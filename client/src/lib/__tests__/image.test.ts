import { describe, it, expect, vi, afterEach } from "vitest";
import { isSupportedImageType, prepareImageForInsert, ImageInputError } from "../image";

function makeFile(name: string, type: string, sizeBytes = 100): File {
  return new File([new Uint8Array(sizeBytes)], name, { type });
}

/** Stand-in for HTMLImageElement: jsdom doesn't decode images, so tests control dimensions directly. */
class FakeImage {
  onload: (() => void) | null = null;
  onerror: (() => void) | null = null;
  naturalWidth = 0;
  naturalHeight = 0;
  private _src = "";

  set src(value: string) {
    this._src = value;
    queueMicrotask(() => this.onload?.());
  }

  get src() {
    return this._src;
  }
}

describe("isSupportedImageType", () => {
  it("accepts common image formats", () => {
    expect(isSupportedImageType(makeFile("a.png", "image/png"))).toBe(true);
    expect(isSupportedImageType(makeFile("a.jpg", "image/jpeg"))).toBe(true);
    expect(isSupportedImageType(makeFile("a.webp", "image/webp"))).toBe(true);
    expect(isSupportedImageType(makeFile("a.gif", "image/gif"))).toBe(true);
  });

  it("rejects non-image or unsupported formats", () => {
    expect(isSupportedImageType(makeFile("a.pdf", "application/pdf"))).toBe(false);
    expect(isSupportedImageType(makeFile("a.svg", "image/svg+xml"))).toBe(false);
    expect(isSupportedImageType(makeFile("a.txt", ""))).toBe(false);
  });
});

describe("prepareImageForInsert", () => {
  const originalImage = globalThis.Image;

  afterEach(() => {
    globalThis.Image = originalImage;
    vi.restoreAllMocks();
  });

  it("rejects unsupported file types before reading", async () => {
    await expect(prepareImageForInsert(makeFile("a.pdf", "application/pdf"))).rejects.toThrow(
      ImageInputError
    );
  });

  it("rejects files above the raw size limit", async () => {
    const file = makeFile("a.png", "image/png");
    Object.defineProperty(file, "size", { value: 21 * 1024 * 1024 });
    await expect(prepareImageForInsert(file)).rejects.toThrow(/too large/);
  });

  it("returns the original data URL unchanged when dimensions are within bounds", async () => {
    class SmallImage extends FakeImage {
      naturalWidth = 400;
      naturalHeight = 300;
    }
    // @ts-expect-error test double for HTMLImageElement
    globalThis.Image = SmallImage;
    const getContextSpy = vi.spyOn(HTMLCanvasElement.prototype, "getContext");

    const result = await prepareImageForInsert(makeFile("small.png", "image/png"));

    expect(result.startsWith("data:image/png;base64,")).toBe(true);
    expect(getContextSpy).not.toHaveBeenCalled();
  });

  it("downscales through canvas when dimensions exceed the max", async () => {
    class LargeImage extends FakeImage {
      naturalWidth = 4000;
      naturalHeight = 2000;
    }
    // @ts-expect-error test double for HTMLImageElement
    globalThis.Image = LargeImage;

    const drawImage = vi.fn();
    const toDataURL = vi.fn(() => "data:image/jpeg;base64,cmVzaXplZA==");
    vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockReturnValue(
      { drawImage } as unknown as CanvasRenderingContext2D
    );
    vi.spyOn(HTMLCanvasElement.prototype, "toDataURL").mockImplementation(toDataURL);

    const result = await prepareImageForInsert(makeFile("large.jpg", "image/jpeg"));

    expect(drawImage).toHaveBeenCalledWith(expect.anything(), 0, 0, 1600, 800);
    expect(toDataURL).toHaveBeenCalledWith("image/jpeg", 0.85);
    expect(result).toBe("data:image/jpeg;base64,cmVzaXplZA==");
  });

  it("never resizes GIFs, to preserve animation", async () => {
    class LargeImage extends FakeImage {
      naturalWidth = 4000;
      naturalHeight = 4000;
    }
    // @ts-expect-error test double for HTMLImageElement
    globalThis.Image = LargeImage;
    const getContextSpy = vi.spyOn(HTMLCanvasElement.prototype, "getContext");

    const result = await prepareImageForInsert(makeFile("anim.gif", "image/gif"));

    expect(getContextSpy).not.toHaveBeenCalled();
    expect(result.startsWith("data:image/gif;base64,")).toBe(true);
  });

  it("rejects when the compressed result is still too large", async () => {
    class LargeImage extends FakeImage {
      naturalWidth = 4000;
      naturalHeight = 4000;
    }
    // @ts-expect-error test double for HTMLImageElement
    globalThis.Image = LargeImage;

    const hugeBase64 = "A".repeat(12 * 1024 * 1024); // decodes to ~9MB, above the 8MB cap
    vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockReturnValue(
      { drawImage: vi.fn() } as unknown as CanvasRenderingContext2D
    );
    vi.spyOn(HTMLCanvasElement.prototype, "toDataURL").mockReturnValue(
      `data:image/jpeg;base64,${hugeBase64}`
    );

    await expect(prepareImageForInsert(makeFile("large.png", "image/png"))).rejects.toThrow(
      /too large even after compression/
    );
  });
});
