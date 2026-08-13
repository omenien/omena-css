import path from "node:path";
import { pathToFileURL } from "node:url";
import { describe, expect, it } from "vitest";

import { normalizeContractParitySnapshot } from "../../../scripts/contract-parity-runtime";

describe("contract parity snapshot path normalization", () => {
  it("normalizes exact and embedded workspace references without matching root prefixes", () => {
    const workspaceRoot = path.resolve("test/_fixtures/contract-parity");

    expect(
      normalizeContractParitySnapshot(
        {
          exactRoot: workspaceRoot,
          embeddedPath: `expression:id:${workspaceRoot}${path.sep}Fixture.module.scss`,
          embeddedFileUrl: `styleImport:id:${
            pathToFileURL(path.join(workspaceRoot, "Fixture.module.scss")).href
          }`,
          rootPrefix: `${workspaceRoot}-copy${path.sep}Fixture.module.scss`,
        },
        workspaceRoot,
      ),
    ).toEqual({
      exactRoot: "<workspace>",
      embeddedPath: "expression:id:<workspace>/Fixture.module.scss",
      embeddedFileUrl: "styleImport:id:<workspace>/Fixture.module.scss",
      rootPrefix: `${workspaceRoot}-copy${path.sep}Fixture.module.scss`,
    });
  });
});
