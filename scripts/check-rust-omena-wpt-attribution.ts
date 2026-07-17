import { strict as assert } from "node:assert";
import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import path from "node:path";

const repoRoot = process.cwd();
const corpusRoot = path.join(repoRoot, "rust/crates/omena-diff-test/wpt-corpus");
const diffTestRoot = path.join(repoRoot, "rust/crates/omena-diff-test");
const specDataRoot = path.join(repoRoot, "rust/crates/omena-spec-audit/data");
const requiredCopyright = "Copyright © web-platform-tests contributors";

const licensePath = path.join(corpusRoot, "LICENSE.md");
const noticePath = path.join(corpusRoot, "NOTICE.md");
const policyPath = path.join(corpusRoot, "VENDORING-POLICY.md");
const specNoticePath = path.join(specDataRoot, "NOTICE.md");

const license = readFileSync(licensePath, "utf8");
const notice = readFileSync(noticePath, "utf8");
const policy = readFileSync(policyPath, "utf8");
const specNotice = readFileSync(specNoticePath, "utf8");

assert.ok(license.startsWith("# The 3-Clause BSD License\n"));
assert.ok(license.includes(requiredCopyright));
assert.ok(license.includes("Redistribution and use in source and binary forms"));
assert.ok(
  license.includes('THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"'),
);
assert.ok(notice.includes(requiredCopyright));
assert.ok(notice.includes("web-platform-tests/wpt"));
assert.match(policy, /\|\s*Web Platform Tests\s*\|\s*BSD 3-Clause/u);
assert.match(policy, /\|\s*`@webref\/css` wrapper\s*\|\s*MIT/u);
assert.match(policy, /\|\s*WHATWG-derived Webref entries\s*\|\s*CC BY 4\.0/u);
assert.match(policy, /\|\s*MDN Browser Compatibility Data\s*\|\s*CC0 1\.0 Universal/u);
assert.ok(specNotice.includes("Webref CSS"));
assert.ok(specNotice.includes("MIT License"));
assert.ok(specNotice.includes("CC BY 4.0"));
assert.ok(specNotice.includes("W3C Document License"));
assert.ok(specNotice.includes("CC0 1.0 Universal"));

const citedArtifacts = collectFiles(diffTestRoot).filter((filePath) => {
  if (!/\.(?:json|toml)$/u.test(filePath)) return false;
  return /"wptPath"\s*:/u.test(readFileSync(filePath, "utf8"));
});

if (process.env.OMENA_WPT_ATTRIBUTION_TEST_INJECT_NOTICELESS_CITATION === "1") {
  citedArtifacts.push(path.join(diffTestRoot, "uncovered-wpt-citation.json"));
}

const uncoveredArtifacts = citedArtifacts.filter(
  (filePath) => !hasNoticeBoundary(path.dirname(filePath), diffTestRoot),
);
assert.deepEqual(
  uncoveredArtifacts.map((filePath) => path.relative(repoRoot, filePath)).sort(),
  [],
  "WPT-derived data must live below a directory containing LICENSE.md and NOTICE.md",
);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.omena-wpt-attribution",
      citedArtifactCount: citedArtifacts.length,
      uncoveredArtifactCount: uncoveredArtifacts.length,
      wptLicense: "BSD-3-Clause",
      webrefWrapperLicense: "MIT",
      whatwgDerivedDataLicense: "CC-BY-4.0",
      mdnBcdLicense: "CC0-1.0",
    },
    null,
    2,
  )}\n`,
);

function hasNoticeBoundary(startDirectory: string, stopDirectory: string): boolean {
  let current = startDirectory;
  while (current.startsWith(stopDirectory)) {
    if (
      existsSync(path.join(current, "LICENSE.md")) &&
      existsSync(path.join(current, "NOTICE.md"))
    ) {
      return true;
    }
    if (current === stopDirectory) break;
    current = path.dirname(current);
  }
  return false;
}

function collectFiles(root: string): string[] {
  const files: string[] = [];
  for (const entry of readdirSync(root)) {
    const filePath = path.join(root, entry);
    if (statSync(filePath).isDirectory()) {
      files.push(...collectFiles(filePath));
    } else {
      files.push(filePath);
    }
  }
  return files;
}
