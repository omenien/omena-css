import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";

const testName =
  "runtime::lex_cache::tests::lex_splice_equivalence_property_covers_generated_edits";

const result = spawnSync(
  "cargo",
  [
    "test",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "omena-transform-passes",
    testName,
    "--",
    "--exact",
  ],
  {
    cwd: process.cwd(),
    encoding: "utf8",
    stdio: "pipe",
  },
);

if (result.stdout) process.stdout.write(result.stdout);
if (result.stderr) process.stderr.write(result.stderr);

assert.equal(result.status, 0, "lex splice equivalence property test must pass");
assert.ok(
  result.stdout.includes(`test ${testName} ... ok`),
  "lex splice equivalence property test must be executed exactly",
);

console.log(
  JSON.stringify({
    schemaVersion: "0",
    product: "rust.omena-transform-passes.lex-splice-equivalence",
    testName,
  }),
);
