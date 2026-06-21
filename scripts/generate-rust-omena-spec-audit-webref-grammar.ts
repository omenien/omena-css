import { strict as assert } from "node:assert";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

import {
  GENERATOR_TOOL,
  WEBREF_GRAMMAR_SNAPSHOT,
  extractWebrefGrammarSnapshot,
  serializeWebrefGrammarSnapshot,
} from "./webref-grammar-extract";

// Vendor the `@webref/css` value-definition-syntax grammar snapshot from the
// installed pinned package. `--check` asserts the on-disk snapshot is current
// (idempotent regeneration); no flag rewrites it. Mirrors the WPT corpus
// generator discipline.

const repoRoot = process.cwd();
const checkOnly = process.argv.includes("--check");
const snapshotPath = path.join(repoRoot, WEBREF_GRAMMAR_SNAPSHOT);

const snapshot = extractWebrefGrammarSnapshot(repoRoot);
const snapshotSource = serializeWebrefGrammarSnapshot(snapshot);

if (checkOnly) {
  assert.equal(
    readFileSync(snapshotPath, "utf8"),
    snapshotSource,
    `${WEBREF_GRAMMAR_SNAPSHOT} is stale; run \`node --import tsx ./${GENERATOR_TOOL}\``,
  );
} else {
  writeFileSync(snapshotPath, snapshotSource);
}

process.stdout.write(
  `${JSON.stringify(
    {
      product: "omena-spec-audit.webref-grammar-generator",
      mode: checkOnly ? "check" : "write",
      version: snapshot.source.version,
      gitHead: snapshot.source.gitHead,
      entryCount: snapshot.entryCount,
      categoryCounts: Object.fromEntries(
        Object.entries(snapshot.categories).map(([category, entries]) => [
          category,
          entries.length,
        ]),
      ),
      generatedFiles: [WEBREF_GRAMMAR_SNAPSHOT],
    },
    null,
    2,
  )}\n`,
);
