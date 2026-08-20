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
      checkCommand(args);
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
  const rendered = renderRegisteredRelease(tag, {
    withGeneratedChangelog: args.includes("--changelog"),
  });
  if (output) {
    writeFileSync(path.resolve(repoRoot, output), rendered);
    process.stdout.write(`release notes: ${tag} -> ${output}\n`);
    return;
  }
  process.stdout.write(rendered);
}

function checkCommand(args: readonly string[]): void {
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
    assert.ok(
      notes.startsWith("---\n"),
      `${entry.notesFile} must carry docs-site frontmatter in the source page`,
    );
    for (const heading of [
      "## Platform surface",
      "## Explicit opt-ins",
      "## Behavior corrections",
      "## Version scope",
    ]) {
      assert.ok(notes.includes(heading), `${entry.notesFile} must include ${heading}`);
    }
    assert.ok(
      !entry.name.includes(entry.tag),
      `${entry.tag} release name must not repeat the tag string`,
    );
    const rendered = renderRegisteredRelease(entry.tag);
    assert.ok(rendered.length > notes.length - 200);
    assert.ok(
      !rendered.startsWith("---"),
      `${entry.tag} rendered body must not leak docs frontmatter`,
    );
    assert.ok(
      !rendered.includes("sourceOfTruth:"),
      `${entry.tag} rendered body must not leak docs metadata fields`,
    );
    assert.ok(rendered.includes("## Release links"), `${entry.tag} body must carry release links`);
    if (manifestPreviousTag(manifest, entry)) {
      assert.ok(
        rendered.includes("- Full diff:"),
        `${entry.tag} body must link the compare range to its predecessor`,
      );
    }
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
  ].map((relativePath) => [relativePath, workflowSourceForCheck(relativePath, args)] as const);
  for (const [relativePath, workflow] of workflows) {
    assert.ok(
      workflow.includes("release-notes.md"),
      `${relativePath} must render and publish release-notes.md`,
    );
    assert.ok(
      workflow.includes("--changelog"),
      `${relativePath} must render the generated changelog section`,
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
  const provenanceGuardReceipts: ProvenanceGuardReceipt[] = [];
  provenanceGuardReceipts.push(
    assertProvenanceSourceGuard({
      relativePath: ".github/workflows/publish-extension.yml",
      scope: "publish",
      source: extensionWorkflow,
      stepName: "Verify publication checkout matches provenance source",
      before: "- uses: ./.github/actions/setup-pnpm",
    }),
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

  const npmWorkflow = workflowSourceForCheck(".github/workflows/_publish-npm.yml", args);
  const npmIntegrityJob = npmWorkflow.slice(
    npmWorkflow.indexOf("  release-integrity:"),
    npmWorkflow.indexOf("  # --- 1."),
  );
  provenanceGuardReceipts.push(
    assertProvenanceSourceGuard({
      relativePath: ".github/workflows/_publish-npm.yml",
      scope: "release-integrity",
      source: npmIntegrityJob,
      stepName: "Verify publication checkout matches provenance source",
      before: "- uses: ./.github/actions/setup-pnpm",
    }),
  );
  provenanceGuardReceipts.push(
    assertProvenanceSourceGuard({
      relativePath: ".github/workflows/_publish-npm.yml",
      scope: "napi-binaries",
      source: yamlJobSource(npmWorkflow, "napi-binaries"),
      stepName: "Verify publication checkout matches provenance source",
      before: "- uses: ./.github/actions/setup-pnpm",
    }),
  );
  const npmPublishJob = yamlJobSource(npmWorkflow, "publish");
  provenanceGuardReceipts.push(
    assertProvenanceSourceGuard({
      relativePath: ".github/workflows/_publish-npm.yml",
      scope: "publish",
      source: npmPublishJob,
      stepName: "Verify publication checkout matches provenance source",
      before: "- uses: ./.github/actions/setup-pnpm",
    }),
  );
  const npmPluginPublishJob = yamlJobSource(npmWorkflow, "publish-plugins");
  provenanceGuardReceipts.push(
    assertProvenanceSourceGuard({
      relativePath: ".github/workflows/_publish-npm.yml",
      scope: "publish-plugins",
      source: npmPluginPublishJob,
      stepName: "Verify publication checkout matches provenance source",
      before: "- uses: ./.github/actions/setup-pnpm",
    }),
  );
  assert.ok(
    npmWorkflow.includes("needs: [release-integrity, napi-binaries]") &&
      npmWorkflow.includes("needs: release-integrity"),
    "every npm publication job must remain downstream of the provenance source guard",
  );

  const sifWorkflow = workflowSourceForCheck(".github/workflows/sif-keyless-attestation.yml", args);
  provenanceGuardReceipts.push(
    assertProvenanceSourceGuard({
      relativePath: ".github/workflows/sif-keyless-attestation.yml",
      scope: "generate-and-attest",
      source: yamlJobSource(sifWorkflow, "generate-and-attest"),
      stepName: "Verify attestation checkout matches provenance source",
      before: "- uses: ./.github/actions/setup-rust-pinned",
    }),
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
  const malformedCheckout = spawnSync(process.execPath, [provenanceChecker], {
    cwd: repoRoot,
    encoding: "utf8",
    env: { ...process.env, OMENA_PROVENANCE_SOURCE_SHA: "0".repeat(39) },
  });
  assert.notEqual(malformedCheckout.status, 0, "malformed publication source SHA must fail");
  assert.match(malformedCheckout.stderr, /requires a full OMENA_PROVENANCE_SOURCE_SHA/u);

  assert.ok(existsSync(path.join(repoRoot, ".github/release.yml")));
  const processDoc = readFileSync(path.join(repoRoot, "docs/releases/PROCESS.md"), "utf8");
  for (const heading of [
    "## Tag families",
    "## Release order",
    "## Provenance rule",
    "## Release notes pipeline",
    "## Historical releases",
  ]) {
    assert.ok(processDoc.includes(heading), `docs/releases/PROCESS.md must include ${heading}`);
  }
  assert.equal(stripDocsFrontmatter("---\ntitle: x\n---\nBody\n"), "Body\n");
  assert.equal(stripDocsFrontmatter("Body only"), "Body only");
  const latestExtension = manifest.releases.find((entry) => entry.tag === "vscode-v5.4.0");
  assert.ok(latestExtension);
  assert.equal(manifestPreviousTag(manifest, latestExtension), "vscode-v5.3.0");
  assert.equal(
    stripLegacyPreamble(`${legacyEraBanner}\n\n## [4.0.0]\n> quoted content stays`),
    "## [4.0.0]\n> quoted content stays",
    "legacy banner stripping must be idempotent and preserve body blockquotes",
  );
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
        provenanceSourceGuardChecks: provenanceGuardReceipts,
        provenanceRuntimeSelfTests: {
          matchingCheckoutExit: matchingCheckout.status,
          mismatchedCheckoutExit: mismatchedCheckout.status,
          malformedCheckoutExit: malformedCheckout.status,
        },
      },
      null,
      2,
    )}\n`,
  );
}

interface ProvenanceGuardReceipt {
  readonly workflow: string;
  readonly scope: string;
  readonly guardStep: string;
  readonly checkoutCount: number;
  readonly sourceShaValidation: "full-lowercase-hex-40";
  readonly mismatchExit: 1;
}

interface ProvenanceGuardCheckInput {
  readonly relativePath: string;
  readonly scope: string;
  readonly source: string;
  readonly stepName: string;
  readonly before: string;
}

const provenanceGuardRunBlock = [
  "set -euo pipefail",
  'if [[ ! "${OMENA_PROVENANCE_SOURCE_SHA}" =~ ^[0-9a-f]{40}$ ]]; then',
  'echo "publish provenance source check requires a full OMENA_PROVENANCE_SOURCE_SHA or GITHUB_SHA" >&2',
  "exit 1",
  "fi",
  'checked_out_sha="$(git rev-parse HEAD)"',
  'if [[ "${checked_out_sha}" != "${OMENA_PROVENANCE_SOURCE_SHA}" ]]; then',
  'echo "publish provenance source mismatch" >&2',
  'echo "workflow provenance source: ${OMENA_PROVENANCE_SOURCE_SHA}" >&2',
  'echo "checked-out publication source: ${checked_out_sha}" >&2',
  'echo "Dispatch the workflow from the same immutable ref that is being checked out." >&2',
  "exit 1",
  "fi",
  'echo "publish provenance source verified: ${checked_out_sha}"',
].join("\n");

function assertProvenanceSourceGuard(input: ProvenanceGuardCheckInput): ProvenanceGuardReceipt {
  const checkoutMarker = "- uses: actions/checkout@";
  const checkoutCount = occurrences(input.source, checkoutMarker);
  assert.equal(
    checkoutCount,
    1,
    `${input.relativePath} ${input.scope} must have exactly one checkout before its provenance guard`,
  );
  const checkoutIndex = input.source.indexOf(checkoutMarker);
  const step = namedWorkflowStep(input.source, input.stepName);
  const beforeIndex = input.source.indexOf(input.before);
  assert.ok(
    checkoutIndex < step.start,
    `${input.relativePath} ${input.scope} provenance guard must follow checkout`,
  );
  assert.ok(
    beforeIndex >= step.end,
    `${input.relativePath} ${input.scope} provenance guard must finish before ${input.before}`,
  );
  const normalizedStep = step.source
    .split("\n")
    .map((line) => line.trimStart())
    .join("\n");
  assert.ok(
    normalizedStep.includes("OMENA_PROVENANCE_SOURCE_SHA: ${{ github.sha }}"),
    `${input.relativePath} ${input.scope} must compare against the workflow provenance source`,
  );
  assert.ok(
    normalizedStep.includes(provenanceGuardRunBlock),
    `${input.relativePath} ${input.scope} must carry the canonical fail-closed provenance guard`,
  );
  assert.equal(
    normalizedStep.includes("exit 0"),
    false,
    `${input.relativePath} ${input.scope} provenance guard must not turn a refusal into success`,
  );
  return {
    workflow: input.relativePath,
    scope: input.scope,
    guardStep: input.stepName,
    checkoutCount,
    sourceShaValidation: "full-lowercase-hex-40",
    mismatchExit: 1,
  };
}

function workflowSourceForCheck(relativePath: string, args: readonly string[]): string {
  const source = readFileSync(path.join(repoRoot, relativePath), "utf8");
  if (relativePath !== ".github/workflows/publish-extension.yml") return source;
  const mutationFlags = [
    "--inject-provenance-guard-deletion",
    "--inject-provenance-guard-inversion",
    "--inject-provenance-guard-exit-zero",
  ].filter((flag) => args.includes(flag));
  assert.ok(mutationFlags.length <= 1, "select at most one provenance guard mutation");
  const [mutation] = mutationFlags;
  if (!mutation) return source;

  const stepName = "Verify publication checkout matches provenance source";
  const step = namedWorkflowStep(source, stepName);
  if (mutation === "--inject-provenance-guard-deletion") {
    return source.slice(0, step.start) + source.slice(step.end);
  }
  if (mutation === "--inject-provenance-guard-inversion") {
    const before = 'if [[ "${checked_out_sha}" != "${OMENA_PROVENANCE_SOURCE_SHA}" ]]; then';
    assert.ok(step.source.includes(before), "provenance inversion mutation target is absent");
    const mutated = step.source.replace(
      before,
      'if [[ "${checked_out_sha}" == "${OMENA_PROVENANCE_SOURCE_SHA}" ]]; then',
    );
    return source.slice(0, step.start) + mutated + source.slice(step.end);
  }

  const exitIndex = step.source.lastIndexOf("exit 1");
  assert.ok(exitIndex >= 0, "provenance exit mutation target is absent");
  const mutated = `${step.source.slice(0, exitIndex)}exit 0${step.source.slice(exitIndex + 6)}`;
  return source.slice(0, step.start) + mutated + source.slice(step.end);
}

function namedWorkflowStep(
  source: string,
  stepName: string,
): { readonly start: number; readonly end: number; readonly source: string } {
  const marker = `      - name: ${stepName}\n`;
  const start = source.indexOf(marker);
  assert.ok(start >= 0, `workflow step is missing: ${stepName}`);
  const nextStep = source.indexOf("\n      - ", start + marker.length);
  const end = nextStep < 0 ? source.length : nextStep + 1;
  return { start, end, source: source.slice(start, end) };
}

function yamlJobSource(source: string, jobName: string): string {
  const marker = `\n  ${jobName}:\n`;
  const markerIndex = source.indexOf(marker);
  assert.ok(markerIndex >= 0, `workflow job is missing: ${jobName}`);
  const start = markerIndex + 1;
  const remainder = source.slice(start + marker.length - 1);
  const nextJob = /^  [A-Za-z0-9_-]+:\n/gmu.exec(remainder);
  const end = nextJob ? start + marker.length - 1 + nextJob.index : source.length;
  return source.slice(start, end);
}

function occurrences(source: string, needle: string): number {
  return source.split(needle).length - 1;
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
  const existingTags = new Set(releases.map((release) => release.tag_name));
  const previousTags = previousTagByAxis(releases.map((release) => release.tag_name));
  let changed = 0;

  for (const release of releases.filter(
    (candidate) => !tagFilter || candidate.tag_name === tagFilter,
  )) {
    const body = buildHistoricalBody(release, previousTags.get(release.tag_name), existingTags);
    const registered = manifest.releases.find((entry) => entry.tag === release.tag_name);
    const desiredName = registered?.name ?? release.name ?? release.tag_name;
    const bodyChanged = normalizeMarkdown(release.body ?? "") !== normalizeMarkdown(body);
    const nameChanged = desiredName !== (release.name ?? "");
    if (!bodyChanged && !nameChanged) continue;
    changed += 1;
    if (printBody) {
      process.stdout.write(`${body}\n`);
      continue;
    }
    process.stdout.write(
      `${apply ? "update" : "would update"} ${release.tag_name}${nameChanged ? ` (name -> ${desiredName})` : ""}\n`,
    );
    if (apply) {
      githubJson([
        "api",
        "--method",
        "PATCH",
        `repos/${manifest.repository}/releases/${release.id}`,
        "--raw-field",
        `body=${body}`,
        "--raw-field",
        `name=${desiredName}`,
      ]);
    }
  }

  if (!printBody) {
    process.stdout.write(
      `${apply ? "updated" : "dry-run"} ${changed} of ${releases.length} GitHub Releases\n`,
    );
  }
}

interface RenderOptions {
  readonly withGeneratedChangelog?: boolean;
  readonly previousTagOverride?: string;
}

function renderRegisteredRelease(tag: string, options: RenderOptions = {}): string {
  const manifest = loadManifest();
  const stableTag = tag.replace(/-preview\.\d+$/, "");
  const entry = manifest.releases.find((candidate) => candidate.tag === stableTag);
  assert.ok(entry, `release tag ${tag} is not registered in docs/releases/manifest.json`);

  const sourcePath = path.join(repoRoot, entry.notesFile);
  const source = absolutizeRelativeLinks(
    stripDocsFrontmatter(readFileSync(sourcePath, "utf8")).trim(),
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
  const previousTag = options.previousTagOverride ?? manifestPreviousTag(manifest, entry);
  const changelog = options.withGeneratedChangelog
    ? `\n\n${renderGeneratedChangelog(entry, previousTag)}`
    : "";
  return `${preamble}${source}${changelog}\n\n${renderArtifactSection(entry)}\n\n${renderReleaseLinks(tag, previousTag)}\n`;
}

function stripDocsFrontmatter(source: string): string {
  // docs/releases/*.md pages carry docs-site YAML frontmatter; a GitHub Release
  // body must never expose that internal metadata block.
  return source.replace(/^---\n[\s\S]*?\n---\n/, "");
}

function manifestPreviousTag(
  manifest: ReleaseManifest,
  entry: ReleaseManifestEntry,
): string | undefined {
  const axisTags = manifest.releases
    .filter((candidate) => candidate.axis === entry.axis)
    .map((candidate) => candidate.tag)
    .toSorted(compareReleaseTags);
  return axisTags[axisTags.indexOf(entry.tag) - 1];
}

function renderGeneratedChangelog(
  entry: ReleaseManifestEntry,
  previousTag: string | undefined,
): string {
  const lines = ["## Changes", ""];
  if (entry.axis === "rust") {
    const declared = declaredBreakingSection(entry.version);
    if (declared.length > 0) lines.push(...declared, "");
  }
  if (!previousTag) {
    lines.push("- First registered release on this axis; no comparison baseline exists.");
    return lines.join("\n");
  }
  const repository = loadManifest().repository;
  const comparison = githubJson<{
    readonly commits: readonly { readonly sha: string; readonly commit: { message: string } }[];
    readonly total_commits: number;
  }>(["api", `repos/${repository}/compare/${previousTag}...${entry.tag}`]);
  const groups = new Map<string, string[]>([
    ["Features", []],
    ["Fixes", []],
    ["Performance", []],
    ["Other changes", []],
  ]);
  for (const commit of comparison.commits) {
    const subject = commit.commit.message.split("\n", 1)[0] ?? "";
    const kind = /^([a-z]+)(?:\([^)]*\))?!?:/.exec(subject)?.[1] ?? "";
    const group =
      kind === "feat"
        ? "Features"
        : kind === "fix"
          ? "Fixes"
          : kind === "perf"
            ? "Performance"
            : "Other changes";
    groups.get(group)?.push(`- ${commit.sha.slice(0, 7)} ${subject}`);
  }
  for (const [title, entries] of groups) {
    if (entries.length === 0) continue;
    if (title === "Other changes") {
      lines.push("<details>", `<summary>Other changes (${entries.length})</summary>`, "");
      lines.push(...entries, "", "</details>", "");
      continue;
    }
    lines.push(`### ${title}`, "", ...entries, "");
  }
  if (comparison.total_commits > comparison.commits.length) {
    lines.push(
      `- …and ${comparison.total_commits - comparison.commits.length} earlier commits (see the full diff below).`,
      "",
    );
  }
  while (lines.at(-1) === "") lines.pop();
  return lines.join("\n");
}

function declaredBreakingSection(releaseVersion: string): readonly string[] {
  const intentPath = path.join(repoRoot, "rust/omena-rust-semver-intent.json");
  if (!existsSync(intentPath)) return [];
  const intent = JSON.parse(readFileSync(intentPath, "utf8")) as {
    readonly baselineWorkspaceVersion?: string;
    readonly targetReleaseVersion?: string;
    readonly intents?: readonly {
      readonly crate?: string;
      readonly expectedFailures?: readonly unknown[];
      readonly expectedRuntimeValueChanges?: readonly unknown[];
    }[];
  };
  // The intent contract is the CURRENT train's declaration. Only render it when
  // it targets exactly this release; a HEAD-side backfill of an older tag must
  // not borrow the in-progress next-train declaration.
  if (intent.targetReleaseVersion !== releaseVersion) return [];
  const rows = intent.intents ?? [];
  if (rows.length === 0) return [];
  const lines = [
    `### Declared breaking changes (\`${intent.baselineWorkspaceVersion ?? "?"}\` → \`${intent.targetReleaseVersion ?? "?"}\`)`,
    "",
    "Machine-declared in the release-semver intent contract; every entry below was gated before publication.",
    "",
  ];
  for (const row of rows) {
    const failureCount = row.expectedFailures?.length ?? 0;
    const runtimeCount = row.expectedRuntimeValueChanges?.length ?? 0;
    lines.push(
      `- \`${row.crate ?? "?"}\` — ${failureCount} API change witness(es), ${runtimeCount} declared runtime value change(s)`,
    );
  }
  return lines;
}

function renderArtifactSection(entry: ReleaseManifestEntry): string {
  return ["## Distribution", "", ...entry.distribution.map((fact) => `- ${fact}`)].join("\n");
}

const legacyEraBanner = [
  "> [!NOTE]",
  "> Pre-rebrand release from the `css-module-explainer` era. Current releases use the",
  "> `vscode-v*` (editor extension) and `release-v*` (Rust crate train / CLI / npm) tag families.",
].join("\n");

function buildHistoricalBody(
  release: GitHubRelease,
  previousTag: string | undefined,
  existingTags: ReadonlySet<string> = new Set(),
): string {
  const registeredTag = release.tag_name.replace(/-preview\.\d+$/, "");
  if (loadManifest().releases.some((entry) => entry.tag === registeredTag)) {
    return renderRegisteredRelease(release.tag_name, {
      previousTagOverride: previousTag,
      withGeneratedChangelog: true,
    });
  }

  const existing = stripLegacyPreamble(stripReleaseLinks(release.body ?? "").trim());
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

  const isLegacyEra =
    !release.tag_name.startsWith("release-v") && !release.tag_name.startsWith("vscode-v");
  const preamble = isLegacyEra
    ? `${legacyEraBanner}\n${legacyDuplicateNotice(release, existingTags)}\n`
    : "";
  return `${preamble}${base.trim()}\n\n${renderReleaseLinks(release.tag_name, previousTag)}\n`;
}

function legacyDuplicateNotice(release: GitHubRelease, existingTags: ReadonlySet<string>): string {
  const counterpart = `vscode-${release.tag_name}`;
  if (!existingTags.has(counterpart)) return "";
  return `>\n> This tag duplicates [\`${counterpart}\`](https://github.com/${loadManifest().repository}/releases/tag/${counterpart}), which is the canonical release for this version.\n`;
}

function stripLegacyPreamble(body: string): string {
  // Idempotency: a rerun must not stack banners or duplicate notices. Only the
  // LEADING banner block is stripped; blockquotes inside historical bodies stay.
  if (!body.startsWith("> [!NOTE]\n> Pre-rebrand release")) return body;
  const lines = body.split("\n");
  let index = 0;
  while (index < lines.length && (lines[index] ?? "").startsWith(">")) index += 1;
  return lines.slice(index).join("\n").replace(/^\n+/, "");
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
