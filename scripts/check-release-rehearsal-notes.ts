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

const requiredSections = [
  "Platform surface",
  "Explicit opt-ins",
  "Behavior corrections",
  "Version scope",
  "Changes",
  "Distribution",
  "Release links",
] as const;

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
const persistedBody = normalize(latest.body ?? "");
assert.ok(persistedBody.length > 0, `${latest.tag_name} has no persisted body`);

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
  const renderedForHealth = process.argv.includes("--inject-missing-required-section")
    ? rendered.replace(/^## Release links$/mu, "## Release links omitted by test fixture")
    : rendered;
  assert.ok(renderedForHealth.length > 0, `${latest.tag_name} fresh render is empty`);
  for (const section of requiredSections) {
    assert.ok(
      new RegExp(`^## ${escapeRegExp(section)}$`, "mu").test(renderedForHealth),
      `${latest.tag_name} fresh render is missing required section: ${section}`,
    );
  }

  // A persisted release body may receive legitimate editorial changes after
  // publication. Keep that live comparison observable without making those
  // edits a rehearsal failure.
  const persistedBodyForComparison = process.argv.includes("--inject-edited-persisted-body")
    ? `${persistedBody}\n\nEditorial clarification from test fixture.`
    : persistedBody;
  const bodyMatchesFreshRender = rendered === persistedBodyForComparison;
  if (!bodyMatchesFreshRender) {
    process.stdout.write(
      `::notice::${latest.tag_name} persisted GitHub Release body differs from a fresh authenticated render; this difference is outside LANE-GREEN\n`,
    );
  }
  process.stdout.write(
    `${JSON.stringify(
      {
        schemaVersion: "0",
        product: "release.rehearsal.notes",
        tag: latest.tag_name,
        liveApiRead: true,
        liveApiCompared: true,
        persistedBodyBytes: Buffer.byteLength(persistedBodyForComparison),
        renderedBytes: Buffer.byteLength(rendered),
        requiredSections,
        bodyMatchesFreshRender,
        bodyComparisonAffectsLaneGreen: false,
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

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
}
