import { expect } from "vitest";
import type { Range } from "@omena/shared";

/**
 * Custom matchers shared across Tier 1 and Tier 2.
 *
 * `toMatchLspRange(line, character, length)` — replaces the
 * verbose `{ start: { line, character }, end: { line, character: char + len } }`
 * literal that recurs in every provider test's assertion.
 *
 * Wired via `vitest.config.ts` `setupFiles` so all tests have
 * the matcher without importing anything.
 */

interface LspRangeMatchers<R = unknown> {
  toMatchLspRange(line: number, character: number, length: number): R;
}

declare module "vitest" {
  // eslint-disable-next-line @typescript-eslint/consistent-type-definitions
  interface Assertion<T = unknown> extends LspRangeMatchers<T> {}
  // eslint-disable-next-line @typescript-eslint/consistent-type-definitions
  interface AsymmetricMatchersContaining extends LspRangeMatchers {}
}

expect.extend({
  toMatchLspRange(
    received: Range,
    line: number,
    character: number,
    length: number,
  ): { pass: boolean; message: () => string } {
    const expected: Range = {
      start: { line, character },
      end: { line, character: character + length },
    };
    const pass =
      received.start.line === expected.start.line &&
      received.start.character === expected.start.character &&
      received.end.line === expected.end.line &&
      received.end.character === expected.end.character;
    return {
      pass,
      message: () =>
        pass
          ? `expected ${JSON.stringify(received)} NOT to match LSP range ${JSON.stringify(expected)}`
          : `expected ${JSON.stringify(received)} to match LSP range ${JSON.stringify(expected)}`,
    };
  },
});
