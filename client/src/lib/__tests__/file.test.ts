import { describe, it, expect } from "vitest";
import { fileUriToPath, decodedByteSize, dataUrlToBytes } from "../file";

describe("fileUriToPath", () => {
  it("returns null for non-file URIs", () => {
    expect(fileUriToPath("https://example.com/a.png")).toBe(null);
  });

  it("decodes a plain unix path", () => {
    expect(fileUriToPath("file:///home/clement/Downloads/3.png")).toBe(
      "/home/clement/Downloads/3.png"
    );
  });

  it("decodes percent-encoded characters", () => {
    expect(fileUriToPath("file:///home/clement/My%20Photos/3.png")).toBe(
      "/home/clement/My Photos/3.png"
    );
  });

  it("strips the extra leading slash on Windows drive paths", () => {
    expect(fileUriToPath("file:///C:/Users/clement/3.png")).toBe("C:/Users/clement/3.png");
  });
});

describe("decodedByteSize", () => {
  it("approximates decoded byte length from base64 length", () => {
    const dataUrl = `data:text/plain;base64,${"A".repeat(100)}`;
    expect(decodedByteSize(dataUrl)).toBe(75);
  });
});

describe("dataUrlToBytes", () => {
  it("decodes a base64 payload back to its original bytes", () => {
    const bytes = dataUrlToBytes("data:text/plain;base64,SGVsbG8=");
    expect(new TextDecoder().decode(bytes)).toBe("Hello");
  });
});
