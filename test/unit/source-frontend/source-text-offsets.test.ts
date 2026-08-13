import { describe, expect, it } from "vitest";
import {
  utf16OffsetAtUtf8ByteOffset,
  utf8ByteOffsetAtUtf16Offset,
} from "../../../server/engine-core-ts/src/core/source-frontend/source-text-offsets";

describe("source text offset conversion", () => {
  const source = "A한😀Z";

  it("converts ASCII, BMP, and astral boundaries in both directions", () => {
    expect([0, 1, 2, 4, 5].map((offset) => utf8ByteOffsetAtUtf16Offset(source, offset))).toEqual([
      0, 1, 4, 8, 9,
    ]);
    expect([0, 1, 4, 8, 9].map((offset) => utf16OffsetAtUtf8ByteOffset(source, offset))).toEqual([
      0, 1, 2, 4, 5,
    ]);
  });

  it("floors offsets inside encoded code points to the preceding shared boundary", () => {
    expect(utf8ByteOffsetAtUtf16Offset(source, 3)).toBe(4);
    expect([2, 3].map((offset) => utf16OffsetAtUtf8ByteOffset(source, offset))).toEqual([1, 1]);
    expect([5, 6, 7].map((offset) => utf16OffsetAtUtf8ByteOffset(source, offset))).toEqual([
      2, 2, 2,
    ]);
  });

  it("round-trips every valid UTF-16 and UTF-8 boundary", () => {
    for (const utf16Offset of [0, 1, 2, 4, 5]) {
      expect(
        utf16OffsetAtUtf8ByteOffset(source, utf8ByteOffsetAtUtf16Offset(source, utf16Offset)),
      ).toBe(utf16Offset);
    }
    for (const utf8ByteOffset of [0, 1, 4, 8, 9]) {
      expect(
        utf8ByteOffsetAtUtf16Offset(source, utf16OffsetAtUtf8ByteOffset(source, utf8ByteOffset)),
      ).toBe(utf8ByteOffset);
    }
  });
});
