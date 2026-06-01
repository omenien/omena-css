import { describe, it, expect } from "vitest";
import type { Position, Range } from "@omena/shared";

describe("scaffolding smoke test", () => {
  it("imports shared types without runtime cost", () => {
    const pos: Position = { line: 0, character: 0 };
    const range: Range = { start: pos, end: pos };
    expect(range.start).toBe(pos);
  });

  it("confirms vitest discovery is wired", () => {
    expect(1 + 1).toBe(2);
  });
});
