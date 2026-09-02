import { strict as assert } from "node:assert";
import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import os from "node:os";
import path from "node:path";

interface GitHubRelease {
  readonly tag_name: string;
  readonly body: string | null;
  readonly published_at: string | null;
}

const repoRoot = process.cwd();
assert.ok(
  process.env.GH_TOKEN || process.env.GITHUB_TOKEN,
  "authenticated GitHub API access is required",
);
const repository = process.env.GITHUB_REPOSITORY ?? "omenien/omena-css";
const releases = JSON.parse(
  execFileSync("gh", ["api", `repos/${repository}/releases?per_page=100`], {
    cwd: repoRoot,
    encoding: "utf8",
  }),
) as GitHubRelease[];
const latest = releases
  .filter((release) => /^release-v\d+\.\d+\.\d+$/u.test(release.tag_name) && release.published_at)
  .toSorted((left, right) => right.published_at!.localeCompare(left.published_at!))[0];
assert(latest, "no published release-v* GitHub Release exists");
assert.ok(normalize(latest.body ?? "").length > 0, `${latest.tag_name} has no persisted body`);

const tempDir = mkdtempSync(path.join(os.tmpdir(), "omena-release-notes-rehearsal-"));
try {
  const outputPath = path.join(tempDir, "release-notes.md");
  execFileSync(
    process.execPath,
    [
      "--import",
      "tsx",
      "./scripts/release-notes.ts",
      "render",
      "--tag",
      latest.tag_name,
      "--changelog",
      "--output",
      outputPath,
    ],
    { cwd: repoRoot, stdio: "inherit", env: process.env },
  );
  const rendered = normalize(readFileSync(outputPath, "utf8"));
  assert.equal(
    rendered,
    normalize(latest.body ?? ""),
    `${latest.tag_name} live body differs from a fresh authenticated render`,
  );
  process.stdout.write(
    `${JSON.stringify(
      {
        schemaVersion: "0",
        product: "release.rehearsal.notes",
        tag: latest.tag_name,
        liveApiCompared: true,
        renderedBytes: Buffer.byteLength(rendered),
      },
      null,
      2,
    )}\n`,
  );
} finally {
  rmSync(tempDir, { recursive: true, force: true });
}

function normalize(value: string): string {
  return value.replace(/\r\n/gu, "\n").trimEnd();
}
