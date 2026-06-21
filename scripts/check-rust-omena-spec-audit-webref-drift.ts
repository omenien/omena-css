import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";

import {
  GENERATOR_TOOL,
  SPEC_SOURCES_JSON,
  WEBREF_GRAMMAR_SNAPSHOT,
  WEBREF_PACKAGE,
  WEBREF_PACKAGE_JSON,
  extractWebrefGrammarSnapshot,
  serializeWebrefGrammarSnapshot,
} from "./webref-grammar-extract";

// Drift fence: re-extract the webref grammar from the installed pinned package and
// assert it is byte-identical to the vendored snapshot, and that the snapshot's
// stamped provenance matches the spec-sources pin. A drift means the upstream pin
// (or the snapshot) changed: per the source-pin review policy that is a
// human-reviewed re-pin, never a silent regeneration.

interface SpecSourcePin {
  readonly package: string;
  readonly version: string;
  readonly gitHead: string;
}

const repoRoot = process.cwd();

const installed = JSON.parse(readFileSync(path.join(repoRoot, WEBREF_PACKAGE_JSON), "utf8")) as {
  version?: string;
};
const pins = JSON.parse(readFileSync(path.join(repoRoot, SPEC_SOURCES_JSON), "utf8")) as {
  sources?: readonly SpecSourcePin[];
  generatedDataReviewGate?: { humanReviewRequired?: boolean };
};
const pin = (pins.sources ?? []).find((source) => source.package === WEBREF_PACKAGE);
assert.ok(pin, `${SPEC_SOURCES_JSON} must pin ${WEBREF_PACKAGE}`);

const humanReviewRequired = pins.generatedDataReviewGate?.humanReviewRequired ?? true;
const repinInstruction =
  `Webref grammar drift against the pinned ${WEBREF_PACKAGE} ${pin.version}. This is a ` +
  `human-reviewed re-pin, not a silent regeneration: update ${SPEC_SOURCES_JSON}, regenerate ` +
  `the snapshot (\`node --import tsx ./${GENERATOR_TOOL}\`), and have a maintainer review the ` +
  `changed grammar${humanReviewRequired ? " (humanReviewRequired)" : ""}.`;

// Installed package must still match the pin before the bytes are even compared.
assert.equal(installed.version, pin.version, repinInstruction);

const vendored = readFileSync(path.join(repoRoot, WEBREF_GRAMMAR_SNAPSHOT), "utf8");
const snapshot = extractWebrefGrammarSnapshot(repoRoot);
const reextracted = serializeWebrefGrammarSnapshot(snapshot);

// The vendored snapshot's stamped provenance must equal the pin.
assert.equal(snapshot.source.version, pin.version, repinInstruction);
assert.equal(snapshot.source.gitHead, pin.gitHead, repinInstruction);

// The vendored bytes must equal a fresh extraction from the pinned package.
assert.equal(reextracted, vendored, repinInstruction);

process.stdout.write(
  `${JSON.stringify(
    {
      product: "omena-spec-audit.webref-grammar-drift",
      package: WEBREF_PACKAGE,
      version: pin.version,
      gitHead: pin.gitHead,
      entryCount: snapshot.entryCount,
      drift: false,
    },
    null,
    2,
  )}\n`,
);
