import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const readme = readFileSync(path.join(repoRoot, "README.md"), "utf8");
const source = requiredCapture(
  readme,
  /Create `path\/to\/file\.module\.css`[\s\S]*?```css\n([\s\S]*?)\n```/u,
  "README CSS lint fixture",
);
const documentedFinding = requiredCapture(
  readme,
  /The report includes this finding:[\s\S]*?```text\n([^\n]+)\n```/u,
  "README lint finding",
);

assert.equal(source.split("\n").length, 3, "README lint fixture must remain three lines");
assert.ok(!source.includes("@keyframes"), "README lint fixture must not declare its keyframes");
assert.ok(
  readme.includes("omena lint path/to/file.module.css"),
  "README must show the zero-config single-file lint command",
);

const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), "omena-readme-lint-"));
const fixturePath = path.join(fixtureRoot, "path", "to", "file.module.css");
mkdirSync(path.dirname(fixturePath), { recursive: true });
writeFileSync(fixturePath, `${source}\n`);

try {
  const result = spawnSync(
    "cargo",
    [
      "run",
      "--quiet",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-cli",
      "--bin",
      "omena",
      "--",
      "lint",
      fixturePath,
    ],
    { cwd: repoRoot, encoding: "utf8", maxBuffer: 16 * 1024 * 1024 },
  );
  assert.equal(result.status, 0, result.stderr);
  const capturedFinding = requiredCapture(
    result.stdout,
    /^\s+(\d+:\d+ missing-keyframes .+)$/mu,
    "executed lint finding",
  );
  assert.equal(
    documentedFinding,
    capturedFinding,
    "README lint finding must match the executed Rust CLI output",
  );
  process.stdout.write(
    `${JSON.stringify(
      {
        schemaVersion: "0",
        product: "docs.readme-example",
        command: "omena lint path/to/file.module.css",
        sourceLineCount: source.split("\n").length,
        finding: capturedFinding,
      },
      null,
      2,
    )}\n`,
  );
} finally {
  rmSync(fixtureRoot, { force: true, recursive: true });
}

function requiredCapture(sourceText: string, pattern: RegExp, label: string): string {
  const match = pattern.exec(sourceText)?.[1];
  assert.ok(match, `${label} is missing`);
  return match;
}
