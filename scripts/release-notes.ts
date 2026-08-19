import { strict as assert } from "node:assert";
import { execFileSync, spawnSync } from "node:child_process";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

type ReleaseAxis = "extension" | "rust";

interface ReleaseManifestEntry {
  readonly tag: string;
  readonly name: string;
  readonly axis: ReleaseAxis;
  readonly version: string;
  readonly notesFile: string;
  readonly distribution: readonly string[];
}

interface ReleaseManifest {
  readonly schemaVersion: string;
  readonly repository: string;
  readonly releases: readonly ReleaseManifestEntry[];
}

interface GitHubRelease {
  readonly id: number;
  readonly tag_name: string;
  readonly name: string | null;
  readonly body: string | null;
  readonly html_url: string;
  readonly prerelease: boolean;
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const manifestPath = path.join(repoRoot, "docs/releases/manifest.json");
const releaseLinksStart = "<!-- omena-release-links:start -->";
const releaseLinksEnd = "<!-- omena-release-links:end -->";

async function main(): Promise<void> {
  const rawArgs = process.argv.slice(2);
  const normalizedArgs = rawArgs[0] === "--" ? rawArgs.slice(1) : rawArgs;
  const [command = "render", ...args] = normalizedArgs;
  switch (command) {
    case "render":
      renderCommand(args);
      return;
    case "check":
      checkCommand();
      return;
    case "verify-github":
      verifyGitHubCommand(args);
      return;
    case "export-github":
      exportGitHubCommand(args);
      return;
    case "backfill":
      backfillCommand(args);
      return;
    default:
      // `pnpm release:notes -- --tag ...` remains the concise authoring form.
      renderCommand(normalizedArgs);
  }
}

function renderCommand(args: readonly string[]): void {
  const tag = requiredOption(args, "--tag");
  const output = option(args, "--output");
  const rendered = renderRegisteredRelease(tag);
  if (output) {
    writeFileSync(path.resolve(repoRoot, output), rendered);
    process.stdout.write(`release notes: ${tag} -> ${output}\n`);
    return;
  }
  process.stdout.write(rendered);
}

function checkCommand(): void {
  const manifest = loadManifest();
  assert.equal(manifest.schemaVersion, "1");
  assert.equal(manifest.repository, "omenien/omena-css");
  assert.ok(manifest.releases.length >= 2, "both release axes must be registered");

  const tags = new Set<string>();
  for (const entry of manifest.releases) {
    assert.ok(!tags.has(entry.tag), `duplicate release tag ${entry.tag}`);
    tags.add(entry.tag);
    assertTagMatchesEntry(entry);
    assert.ok(
      entry.distribution.length > 0,
      `${entry.tag} must freeze its channel-specific distribution facts`,
    );
    assert.equal(
      new Set(entry.distribution).size,
      entry.distribution.length,
      `${entry.tag} distribution facts must be unique`,
    );
    const notePath = path.join(repoRoot, entry.notesFile);
    assert.ok(existsSync(notePath), `${entry.tag} notes file does not exist: ${entry.notesFile}`);
    const notes = readFileSync(notePath, "utf8");
    for (const heading of [
      "## Platform surface",
      "## Explicit opt-ins",
      "## Behavior corrections",
      "## Version scope",
    ]) {
      assert.ok(notes.includes(heading), `${entry.notesFile} must include ${heading}`);
    }
    assert.ok(renderRegisteredRelease(entry.tag).length > notes.length);
  }

  const packageVersion = JSON.parse(readFileSync(path.join(repoRoot, "package.json"), "utf8"))
    .version as string;
  const rustVersion = readWorkspaceVersion();
  assert.ok(
    manifest.releases.some(
      (entry) => entry.axis === "extension" && entry.version === packageVersion,
    ),
    `manifest must register extension ${packageVersion}`,
  );
  assert.ok(
    manifest.releases.some((entry) => entry.axis === "rust" && entry.version === rustVersion),
    `manifest must register Rust train ${rustVersion}`,
  );

  const workflows = [
    ".github/workflows/publish-extension.yml",
    ".github/workflows/release-cli.yml",
    ".github/workflows/_publish-crate-train.yml",
  ].map(
    (relativePath) =>
      [relativePath, readFileSync(path.join(repoRoot, relativePath), "utf8")] as const,
  );
  for (const [relativePath, workflow] of workflows) {
    assert.ok(
      workflow.includes("release-notes.md"),
      `${relativePath} must render and publish release-notes.md`,
    );
    assert.ok(
      workflow.includes("body_path: release-notes.md"),
      `${relativePath} must use the canonical rendered body`,
    );
    assert.ok(
      workflow.includes("verify-github"),
      `${relativePath} must verify the persisted GitHub Release body`,
    );
    assert.ok(
      !workflow.includes("generate_release_notes: true"),
      `${relativePath} must not replace curated notes with generated notes`,
    );
  }
  const extensionWorkflow = workflows.find(
    ([relativePath]) => relativePath === ".github/workflows/publish-extension.yml",
  )?.[1];
  assert.ok(extensionWorkflow);
  const provenanceSourceGuard = 'checked_out_sha="$(git rev-parse HEAD)"';
  assert.ok(
    extensionWorkflow.indexOf(provenanceSourceGuard) <
      extensionWorkflow.indexOf("./scripts/publish-extension.sh"),
    "extension publication must reject a checkout/provenance mismatch before publishing",
  );
  assert.ok(
    extensionWorkflow.indexOf("Render canonical release notes") <
      extensionWorkflow.indexOf("./scripts/publish-extension.sh"),
    "extension notes must render before any marketplace publish",
  );
  const cliWorkflow = workflows.find(
    ([relativePath]) => relativePath === ".github/workflows/release-cli.yml",
  )?.[1];
  assert.ok(cliWorkflow);
  const cliBuildJob = cliWorkflow.slice(
    cliWorkflow.indexOf("  build:"),
    cliWorkflow.indexOf("  release:"),
  );
  const cliReleaseJob = cliWorkflow.slice(cliWorkflow.indexOf("  release:"));
  assert.ok(
    cliBuildJob.includes("ref: ${{ inputs.tag || github.ref }}"),
    "historical CLI binaries must be built from the requested immutable tag",
  );
  assert.ok(
    cliReleaseJob.includes("github.event.repository.default_branch") &&
      cliReleaseJob.includes("export-github"),
    "historical CLI rebuilds must use current tooling and preserve the existing release body",
  );
  const directPublisher = readFileSync(path.join(repoRoot, "scripts/publish-extension.sh"), "utf8");
  assert.ok(
    directPublisher.indexOf("release/check/release-notes") <
      directPublisher.indexOf("vsce publish"),
    "direct extension publishing must run the release-note gate before upload",
  );

  const npmWorkflow = readFileSync(
    path.join(repoRoot, ".github/workflows/_publish-npm.yml"),
    "utf8",
  );
  const npmIntegrityJob = npmWorkflow.slice(
    npmWorkflow.indexOf("  release-integrity:"),
    npmWorkflow.indexOf("  # --- 1."),
  );
  assert.ok(
    npmIntegrityJob.indexOf(provenanceSourceGuard) <
      npmIntegrityJob.indexOf("./.github/actions/setup-pnpm"),
    "npm publication must reject a checkout/provenance mismatch in its prerequisite gate",
  );
  assert.ok(
    npmWorkflow.includes("needs: [release-integrity, napi-binaries]") &&
      npmWorkflow.includes("needs: release-integrity"),
    "every npm publication job must remain downstream of the provenance source guard",
  );

  const provenanceChecker = path.join(repoRoot, "scripts/verify-publish-provenance-source.mjs");
  const headSha = execFileSync("git", ["rev-parse", "HEAD"], {
    cwd: repoRoot,
    encoding: "utf8",
  }).trim();
  const matchingCheckout = spawnSync(process.execPath, [provenanceChecker], {
    cwd: repoRoot,
    encoding: "utf8",
    env: { ...process.env, OMENA_PROVENANCE_SOURCE_SHA: headSha },
  });
  assert.equal(
    matchingCheckout.status,
    0,
    `matching publication checkout must pass:\n${matchingCheckout.stderr}`,
  );
  const mismatchedCheckout = spawnSync(process.execPath, [provenanceChecker], {
    cwd: repoRoot,
    encoding: "utf8",
    env: { ...process.env, OMENA_PROVENANCE_SOURCE_SHA: "0".repeat(40) },
  });
  assert.notEqual(mismatchedCheckout.status, 0, "mismatched publication checkout must fail");
  assert.match(mismatchedCheckout.stderr, /publish provenance source mismatch/u);

  assert.ok(existsSync(path.join(repoRoot, ".github/release.yml")));
  assert.ok(changelogSectionForTag("v5.2.0")?.includes("## [5.2.0]"));
  assert.equal(changelogSectionForTag("release-v0.2.0"), null);
  assert.ok(isCrateNameList("omena-parser\nomena-query\nomena-cli\nomena-wasm\nomena-sif"));
  assert.equal(
    stripReleaseLinks(`body\n\n${renderReleaseLinks("v1.0.0")}`),
    "body",
    "release-link replacement must be idempotent",
  );
  assert.deepEqual(
    [...previousTagByAxis(["v2.0.0", "v1.0.0", "release-v0.3.0", "release-v0.2.0"])],
    [
      ["v1.0.0", undefined],
      ["v2.0.0", "v1.0.0"],
      ["release-v0.2.0", undefined],
      ["release-v0.3.0", "release-v0.2.0"],
    ],
  );
  process.stdout.write(
    `${JSON.stringify(
      {
        schemaVersion: "1",
        product: "release.notes",
        registeredReleaseCount: manifest.releases.length,
        releaseAxes: [...new Set(manifest.releases.map((entry) => entry.axis))].toSorted(),
        workflowCount: workflows.length,
        persistedBodyVerificationRequired: true,
        provenanceSourceGuard: "green",
        provenanceMismatchSelfTest: "red",
      },
      null,
      2,
    )}\n`,
  );
}

function verifyGitHubCommand(args: readonly string[]): void {
  const tag = requiredOption(args, "--tag");
  const expectedPath = path.resolve(repoRoot, requiredOption(args, "--file"));
  const expected = normalizeMarkdown(readFileSync(expectedPath, "utf8"));
  const release = githubJson<GitHubRelease>([
    "api",
    `repos/${loadManifest().repository}/releases/tags/${tag}`,
  ]);
  const actual = normalizeMarkdown(release.body ?? "");
  assert.equal(actual, expected, `GitHub Release body for ${tag} differs from ${expectedPath}`);
  process.stdout.write(`GitHub Release body verified: ${tag}\n`);
}

function exportGitHubCommand(args: readonly string[]): void {
  const tag = requiredOption(args, "--tag");
  const output = path.resolve(repoRoot, requiredOption(args, "--output"));
  const release = githubJson<GitHubRelease>([
    "api",
    `repos/${loadManifest().repository}/releases/tags/${tag}`,
  ]);
  const body = normalizeMarkdown(release.body ?? "");
  assert.ok(body.length > 0, `GitHub Release ${tag} has no body to preserve`);
  writeFileSync(output, `${body}\n`);
  process.stdout.write(
    `GitHub Release body exported: ${tag} -> ${path.relative(repoRoot, output)}\n`,
  );
}

function backfillCommand(args: readonly string[]): void {
  const apply = args.includes("--apply");
  const printBody = args.includes("--print");
  const tagFilter = option(args, "--tag");
  const manifest = loadManifest();
  const releasePages = githubJson<GitHubRelease[][]>([
    "api",
    "--paginate",
    "--slurp",
    `repos/${manifest.repository}/releases?per_page=100`,
  ]);
  const releases = releasePages.flat();
  const previousTags = previousTagByAxis(releases.map((release) => release.tag_name));
  let changed = 0;

  for (const release of releases.filter(
    (candidate) => !tagFilter || candidate.tag_name === tagFilter,
  )) {
    const body = buildHistoricalBody(release, previousTags.get(release.tag_name));
    if (normalizeMarkdown(release.body ?? "") === normalizeMarkdown(body)) continue;
    changed += 1;
    if (printBody) {
      process.stdout.write(`${body}\n`);
      continue;
    }
    process.stdout.write(`${apply ? "update" : "would update"} ${release.tag_name}\n`);
    if (apply) {
      githubJson([
        "api",
        "--method",
        "PATCH",
        `repos/${manifest.repository}/releases/${release.id}`,
        "--raw-field",
        `body=${body}`,
      ]);
    }
  }

  if (!printBody) {
    process.stdout.write(
      `${apply ? "updated" : "dry-run"} ${changed} of ${releases.length} GitHub Releases\n`,
    );
  }
}

function renderRegisteredRelease(tag: string): string {
  const manifest = loadManifest();
  const stableTag = tag.replace(/-preview\.\d+$/, "");
  const entry = manifest.releases.find((candidate) => candidate.tag === stableTag);
  assert.ok(entry, `release tag ${tag} is not registered in docs/releases/manifest.json`);

  const sourcePath = path.join(repoRoot, entry.notesFile);
  const source = absolutizeRelativeLinks(
    readFileSync(sourcePath, "utf8").trim(),
    entry.notesFile,
    tag,
  );
  const preview = tag !== stableTag;
  const preamble = preview
    ? [
        "> [!WARNING]",
        "> This is a preview build. It may change before the stable release and should not be used as a production compatibility baseline.",
        "",
      ].join("\n")
    : "";
  return `${preamble}${source}\n\n${renderArtifactSection(entry)}\n\n${renderReleaseLinks(tag)}\n`;
}

function renderArtifactSection(entry: ReleaseManifestEntry): string {
  return ["## Distribution", "", ...entry.distribution.map((fact) => `- ${fact}`)].join("\n");
}

function buildHistoricalBody(release: GitHubRelease, previousTag: string | undefined): string {
  const registeredTag = release.tag_name.replace(/-preview\.\d+$/, "");
  if (loadManifest().releases.some((entry) => entry.tag === registeredTag)) {
    return renderRegisteredRelease(release.tag_name);
  }

  const existing = stripReleaseLinks(release.body ?? "").trim();
  let base = existing;
  if (base.length === 0) {
    base =
      changelogSectionForTag(release.tag_name) ?? generatedNotes(release.tag_name, previousTag);
  } else if (isCrateNameList(base)) {
    const crates = base
      .split(/\r?\n/)
      .map((line) => line.trim())
      .filter((line) => /^[a-z][a-z0-9-]+$/.test(line));
    base = [
      `# Omena Rust crate train ${versionFromTag(release.tag_name) ?? release.tag_name}`,
      "",
      `This coordinated release published ${crates.length} Rust crates from one tagged workspace revision.`,
      "",
      "<details>",
      "<summary>Published crates</summary>",
      "",
      ...crates.map((crate) => `- \`${crate}\``),
      "",
      "</details>",
    ].join("\n");
  }

  return `${base.trim()}\n\n${renderReleaseLinks(release.tag_name, previousTag)}\n`;
}

function generatedNotes(tag: string, previousTag: string | undefined): string {
  const args = [
    "api",
    "--method",
    "POST",
    `repos/${loadManifest().repository}/releases/generate-notes`,
    "--raw-field",
    `tag_name=${tag}`,
  ];
  if (previousTag) args.push("--raw-field", `previous_tag_name=${previousTag}`);
  const generated = githubJson<{ body: string }>(args).body.trim();
  const context = [
    `# Omena CSS Modules ${versionFromTag(tag) ?? tag}`,
    "",
    "This historical release predates the curated release-note registry, and no standalone changelog entry was committed for this tag. The comparison below is the authoritative change set.",
  ].join("\n");
  if (generated.length > 0) return `${context}\n\n${generated}`;
  return context;
}

function changelogSectionForTag(tag: string): string | null {
  if (tag.startsWith("release-v")) return null;
  const version = versionFromTag(tag);
  if (!version) return null;
  const changelog = readFileSync(path.join(repoRoot, "CHANGELOG.md"), "utf8");
  const heading = new RegExp(`^## (?:\\[)?${escapeRegExp(version)}(?:\\])?(?:\\s|$)`, "m");
  const match = heading.exec(changelog);
  if (!match) return null;
  const start = match.index;
  const rest = changelog.slice(start + match[0].length);
  const next = /^## /m.exec(rest);
  const section = changelog.slice(start, next ? start + match[0].length + next.index : undefined);
  return absolutizeRelativeLinks(section.trim(), "CHANGELOG.md", tag);
}

function renderReleaseLinks(tag: string, previousTag?: string): string {
  const repository = loadManifest().repository;
  const lines = [
    releaseLinksStart,
    "",
    "## Release links",
    "",
    `- Source: [\`${tag}\`](https://github.com/${repository}/tree/${tag})`,
    "- Changelog: [`CHANGELOG.md`](https://github.com/omenien/omena-css/blob/master/CHANGELOG.md)",
  ];
  if (previousTag) {
    lines.push(
      `- Full diff: [\`${previousTag}...${tag}\`](https://github.com/${repository}/compare/${previousTag}...${tag})`,
    );
  }
  lines.push("", releaseLinksEnd);
  return lines.join("\n");
}

function previousTagByAxis(tags: readonly string[]): Map<string, string | undefined> {
  const result = new Map<string, string | undefined>();
  const groups = new Map<string, string[]>();
  for (const tag of tags) {
    const axis = tag.startsWith("release-v")
      ? "rust"
      : tag.startsWith("vscode-v")
        ? "extension"
        : "legacy";
    const group = groups.get(axis) ?? [];
    group.push(tag);
    groups.set(axis, group);
  }
  for (const group of groups.values()) {
    group.sort(compareReleaseTags);
    group.forEach((tag, index) => result.set(tag, group[index - 1]));
  }
  return result;
}

function compareReleaseTags(left: string, right: string): number {
  const leftParts = versionFromTag(left)?.split(".").map(Number) ?? [0];
  const rightParts = versionFromTag(right)?.split(".").map(Number) ?? [0];
  for (let index = 0; index < Math.max(leftParts.length, rightParts.length); index += 1) {
    const difference = (leftParts[index] ?? 0) - (rightParts[index] ?? 0);
    if (difference !== 0) return difference;
  }
  return left.localeCompare(right);
}

function readWorkspaceVersion(): string {
  const cargo = readFileSync(path.join(repoRoot, "rust/Cargo.toml"), "utf8");
  const workspace = /\[workspace\.package\]([\s\S]*?)(?:\n\[|$)/.exec(cargo)?.[1] ?? "";
  const version = /^version\s*=\s*"([^"]+)"/m.exec(workspace)?.[1];
  assert.ok(version, "rust/Cargo.toml must declare workspace.package.version");
  return version;
}

function assertTagMatchesEntry(entry: ReleaseManifestEntry): void {
  const expected =
    entry.axis === "extension" ? `vscode-v${entry.version}` : `release-v${entry.version}`;
  assert.equal(entry.tag, expected, `${entry.axis} release tag must be ${expected}`);
  assert.ok(entry.name.includes(entry.version), `${entry.tag} name must include its version`);
}

function loadManifest(): ReleaseManifest {
  return JSON.parse(readFileSync(manifestPath, "utf8")) as ReleaseManifest;
}

function githubJson<T = unknown>(args: readonly string[]): T {
  const output = execFileSync("gh", args, {
    cwd: repoRoot,
    encoding: "utf8",
    maxBuffer: 20_000_000,
  });
  return JSON.parse(output) as T;
}

function absolutizeRelativeLinks(markdown: string, sourceFile: string, tag: string): string {
  const sourceDir = path.posix.dirname(sourceFile);
  return markdown.replace(/\]\((?!https?:|#|mailto:)([^)]+)\)/g, (_full, target: string) => {
    const resolved = path.posix.normalize(path.posix.join(sourceDir, target));
    return `](https://github.com/omenien/omena-css/blob/${tag}/${resolved})`;
  });
}

function stripReleaseLinks(body: string): string {
  const start = body.indexOf(releaseLinksStart);
  const end = body.indexOf(releaseLinksEnd);
  if (start < 0 || end < start) return body;
  return `${body.slice(0, start)}${body.slice(end + releaseLinksEnd.length)}`.trim();
}

function isCrateNameList(body: string): boolean {
  const lines = body
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);
  const crateLines = lines.filter((line) => /^[a-z][a-z0-9-]+$/.test(line));
  const otherLines = lines.filter((line) => !/^[a-z][a-z0-9-]+$/.test(line));
  return (
    crateLines.length >= 5 &&
    otherLines.every((line) => line.startsWith("**Full Changelog**: https://github.com/"))
  );
}

function versionFromTag(tag: string): string | null {
  return /(?:^|-)v(\d+\.\d+\.\d+)/.exec(tag)?.[1] ?? null;
}

function normalizeMarkdown(value: string): string {
  return value.replace(/\r\n/g, "\n").trimEnd();
}

function option(args: readonly string[], name: string): string | undefined {
  const index = args.indexOf(name);
  return index >= 0 ? args[index + 1] : undefined;
}

function requiredOption(args: readonly string[], name: string): string {
  const value = option(args, name);
  assert.ok(value, `${name} is required`);
  return value;
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

void main().catch((error: unknown) => {
  process.stderr.write(
    `${error instanceof Error ? (error.stack ?? error.message) : String(error)}\n`,
  );
  process.exitCode = 1;
});
