import { describe, expect, it } from "vitest";

import {
  assertRequiredJobResults,
  summarizeRequiredJobResults,
} from "../../../scripts/check-ci-required-results.mjs";

describe("required CI result aggregation", () => {
  it("accepts only an all-success result map", () => {
    expect(
      assertRequiredJobResults(
        JSON.stringify({
          verify: { result: "success" },
          package: { result: "success" },
        }),
      ),
    ).toEqual({
      total: 2,
      failed: [],
    });
  });

  it.each(["failure", "cancelled", "skipped"])("rejects a %s required result", (result) => {
    expect(() =>
      assertRequiredJobResults(
        JSON.stringify({
          verify: { result: "success" },
          package: { result },
        }),
      ),
    ).toThrow(`package=${result}`);
  });

  it("reports every non-success result", () => {
    expect(
      summarizeRequiredJobResults({
        verify: { result: "failure" },
        package: { result: "cancelled" },
      }),
    ).toEqual({
      total: 2,
      failed: [
        { jobId: "verify", result: "failure" },
        { jobId: "package", result: "cancelled" },
      ],
    });
  });
});
