import { strict as assert } from "node:assert";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

interface CssModuleTokenRotationContractV1 {
  readonly schemaVersion: "1";
  readonly product: "omena.css-modules-token-rotation";
  readonly previousRotationIdentity: null;
  readonly rotationIdentity: string;
  readonly coordinatedRelease: {
    readonly extension: "5.4.0";
    readonly rustCrateTrain: "0.4.0";
  };
  readonly breakClasses: ReadonlyArray<{
    readonly id: string;
    readonly warningChannel: "none" | "build-time";
  }>;
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const docsRoot = path.join(repoRoot, "docs");
const requiredFields = [
  "title",
  "description",
  "kind",
  "status",
  "products",
  "owner",
  "sourceOfTruth",
] as const;
const validKinds = new Set(["tutorial", "how-to", "reference", "explanation"]);
const validStatuses = new Set(["stable", "preview", "experimental", "deprecated"]);
const validSourceKinds = new Set(["authored", "generated", "hybrid"]);
const forbiddenPublicTerms = [
  /\bpre-g\d+\b/iu,
  /\bgoal-g\d+\b/iu,
  /\bstage-\d+\b/iu,
  /\bslice-[a-z0-9]+\b/iu,
];
const tokenRotationIdentity = "omena.css-modules.module-and-class-hash.v1";
const tokenRotationBreakClassIds = [
  "persisted-emitted-token",
  "global-to-scoped-dependency-class",
  "handwritten-emitted-token-selector",
  "stale-interface-manifest",
  "mixed-token-identity-versions",
  "collision-retained-declaration-removal",
] as const;

const pagePaths = collectPages(docsRoot);
assert.ok(pagePaths.length >= 35, "the public documentation surface unexpectedly shrank");
const metaPaths = collectNamedFiles(docsRoot, "meta.json");
assert.ok(metaPaths.length > 0, "the documentation navigation metadata is missing");

let generatedPageCount = 0;
let hybridPageCount = 0;
let linkCount = 0;
const kindCounts = new Map<string, number>();

for (const absolutePath of pagePaths) {
  const relativePath = path.relative(repoRoot, absolutePath).replaceAll(path.sep, "/");
  const source = readFileSync(absolutePath, "utf8");
  const { frontmatter, body } = parseFrontmatter(source, relativePath);

  for (const field of requiredFields) {
    assert.ok(frontmatter.has(field), `${relativePath} is missing frontmatter field ${field}`);
  }

  const kind = requiredScalar(frontmatter, "kind", relativePath);
  const status = requiredScalar(frontmatter, "status", relativePath);
  const sourceKind = requiredScalar(frontmatter, "sourceOfTruth", relativePath);
  assert.ok(validKinds.has(kind), `${relativePath} has unsupported kind ${kind}`);
  assert.ok(validStatuses.has(status), `${relativePath} has unsupported status ${status}`);
  assert.ok(
    validSourceKinds.has(sourceKind),
    `${relativePath} has unsupported sourceOfTruth ${sourceKind}`,
  );
  assert.ok(
    parseInlineList(frontmatter.get("products") ?? "").length > 0,
    `${relativePath} must name at least one product`,
  );
  assert.ok(
    requiredScalar(frontmatter, "description", relativePath).length >= 24,
    `${relativePath} description is too short to explain the reader outcome`,
  );

  kindCounts.set(kind, (kindCounts.get(kind) ?? 0) + 1);
  if (sourceKind === "generated") {
    generatedPageCount += 1;
    assert.ok(
      body.includes("Generated from product code. Do not edit by hand."),
      `${relativePath} must carry the generated-file notice`,
    );
  }
  if (sourceKind === "hybrid") hybridPageCount += 1;

  for (const pattern of forbiddenPublicTerms) {
    assert.ok(!pattern.test(source), `${relativePath} contains private planning shorthand`);
  }

  for (const target of markdownLinkTargets(body)) {
    linkCount += 1;
    verifyLocalLink(absolutePath, target, relativePath);
  }
}

for (const kind of validKinds) {
  assert.ok((kindCounts.get(kind) ?? 0) > 0, `documentation kind ${kind} has no pages`);
}

for (const metaPath of metaPaths) {
  verifyNavigationMetadata(metaPath);
}

assert.ok(
  existsSync(path.join(repoRoot, "apps/docs/source.config.ts")),
  "the documentation schema must remain in the site application",
);
assert.ok(
  existsSync(path.join(repoRoot, "apps/docs/src/routes/docs/$.tsx")),
  "the TanStack Start documentation route must remain wired",
);
assert.ok(
  readFileSync(path.join(repoRoot, "apps/docs/src/styles/app.css"), "utf8").includes(
    "fumadocs-ui/css/solar.css",
  ),
  "the documentation site must keep the Solar theme",
);
assert.ok(
  existsSync(path.join(repoRoot, "docs/playground.mdx")),
  "the browser playground guide must remain discoverable",
);
assertFileIncludes(
  "apps/docs/src/routes/__root.tsx",
  'className="docs-skip-link"',
  "the site must expose skip navigation",
);
assertFileIncludes(
  "apps/docs/src/components/documentation-page.tsx",
  'id="main-content"',
  "documentation pages must expose a main landmark",
);
const staticOutputPreparation = readFile("apps/docs/scripts/prepare-static-output.mjs");
assert.ok(
  !staticOutputPreparation.includes(
    'copyFileSync(shellPath, path.join(publicOutput, "index.html"))',
  ),
  "the post-build step must not replace the prerendered root document with the SPA shell",
);
assertFileIncludes(
  "apps/docs/vite.config.ts",
  'maskPath: `${withDeploymentBase("/")}?__spa_shell=1`',
  "the SPA shell mask must not shadow the prerendered root documentation route",
);
assertFileIncludes(
  "apps/docs/vite.config.ts",
  "crawlLinks: false",
  "the explicit static route inventory must not crawl unbased routes during Pages builds",
);
assertFileIncludes(
  "apps/docs/vite.config.ts",
  "autoStaticPathsDiscovery: false",
  "the explicit static route inventory must not discover unbased routes during Pages builds",
);

assertFileIncludes(
  "docs/sdk.md",
  "checkStyleSourceJson",
  "the published NAPI example must use an API available in 0.2.1",
);
assertFileIncludes(
  "docs/sdk.md",
  'import { Workspace } from "@omena/wasm"',
  "the WASM example must use the bundler-target package entrypoint",
);
assert.ok(
  !readFile("docs/sdk.md").includes("import init"),
  "the WASM SDK guide must not require a default initializer absent from the npm package",
);
assertFileIncludes(
  "docs/releases/5.3.0.md",
  "50 of 51 publishable crates",
  "release notes must disclose the partial Rust publication",
);
assert.match(
  readFile("docs/releases/5.3.0.md"),
  /\|\s*NAPI binding\s*\|\s*`0\.2\.1`\s*\|/u,
  "release notes must report the registry NAPI version",
);
assertFileIncludes(
  "docs/reference/cli.md",
  "Cargo feature `mdl`",
  "the CLI reference must expose the compress feature gate",
);
for (const cssModulesConsumerGuide of ["docs/getting-started.md", "docs/sdk.md"]) {
  assert.match(
    readFile(cssModulesConsumerGuide),
    /For CSS Modules, the emitted token is not a contract; `classMap`, `namedExports`,\s+and\s+the generated `\.d\.ts` are\. Hand-writing an emitted token into markup, tests,\s+or\s+CSS is unsupported\./u,
    `${cssModulesConsumerGuide} must declare emitted CSS Modules tokens unsupported as a consumer contract`,
  );
}
const tokenRotation = readCssModuleTokenRotationContract();
// FALSIFIER: delete or rename the identity, reuse a release version, or give the
// legacy unnamed format the same value. The authored release block can emit each
// state, so these are contract checks rather than structural declarations.
assert.equal(tokenRotation.schemaVersion, "1");
assert.equal(tokenRotation.product, "omena.css-modules-token-rotation");
assert.equal(tokenRotation.previousRotationIdentity, null);
assert.equal(
  process.argv.includes("--inject-missing-token-rotation-identity")
    ? undefined
    : tokenRotation.rotationIdentity,
  tokenRotationIdentity,
  "the CSS Modules token rotation must have a stable identity independent of release versions",
);
assert.notEqual(
  tokenRotation.rotationIdentity,
  tokenRotation.previousRotationIdentity,
  "the CSS Modules token rotation identity must differ from the previous release",
);
assert.doesNotMatch(
  tokenRotation.rotationIdentity,
  /\b\d+\.\d+\.\d+\b/u,
  "the CSS Modules token rotation identity must not embed a release version",
);
assert.ok(
  !Object.values(tokenRotation.coordinatedRelease).includes(tokenRotation.rotationIdentity),
  "the CSS Modules token rotation identity must not be derived from a release version",
);
assert.deepEqual(tokenRotation.coordinatedRelease, {
  extension: "5.4.0",
  rustCrateTrain: "0.4.0",
});
const observedTokenRotationBreakClassIds = tokenRotation.breakClasses
  .map((entry) => entry.id)
  .filter(
    (id) =>
      !(
        process.argv.includes("--inject-missing-token-rotation-break-class") &&
        id === tokenRotationBreakClassIds.at(-1)
      ),
  )
  .toSorted();
// FALSIFIER: remove, rename, duplicate, or substitute any class. The authored
// release block is the producer and can emit each malformed membership set.
assert.deepEqual(
  observedTokenRotationBreakClassIds,
  [...tokenRotationBreakClassIds].toSorted(),
  "the CSS Modules token rotation must enumerate the exact silent-break class membership",
);
assert.equal(
  tokenRotation.breakClasses.length,
  tokenRotationBreakClassIds.length,
  "the CSS Modules token rotation break-class count is a redundant membership cross-check",
);
assert.equal(
  tokenRotation.breakClasses.find((entry) => entry.id === "persisted-emitted-token")
    ?.warningChannel,
  "none",
  "persisted emitted tokens have no warning channel",
);
assertFileIncludes(
  "docs/reference/cli.md",
  "Cargo feature `zk-audit`",
  "the CLI reference must expose the audit feature gate",
);
assertFileIncludes(
  "docs/reference/editor-settings.md",
  "Legacy compatibility",
  "the editor settings reference must distinguish compatibility-only keys",
);
assertFileIncludes(
  "docs/reference/crates.md",
  "The catalog classifies every workspace crate",
  "the generated crate catalog must retain its product-path authority",
);
for (const materialPath of [
  "SECURITY.md",
  "SUPPORT.md",
  ".github/ISSUE_TEMPLATE/config.yml",
  ".github/ISSUE_TEMPLATE/bug_report.yml",
  ".github/ISSUE_TEMPLATE/feature_request.yml",
]) {
  assert.ok(existsSync(path.join(repoRoot, materialPath)), `${materialPath} is missing`);
}

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "docs.site-contracts",
      pageCount: pagePaths.length,
      navigationMetadataCount: metaPaths.length,
      generatedPageCount,
      hybridPageCount,
      kindCounts: Object.fromEntries(
        [...kindCounts].toSorted(([left], [right]) => left.localeCompare(right)),
      ),
      linkCount,
      missingMetadataCount: 0,
      brokenLocalLinkCount: 0,
      unlistedNavigationPageCount: 0,
      stalePublicContractCount: 0,
      forbiddenPlanningTermCount: 0,
      cssModuleTokenRotationIdentity: tokenRotation.rotationIdentity,
      cssModuleTokenRotationPreviousIdentity: tokenRotation.previousRotationIdentity,
      cssModuleTokenRotationBreakClassCount: tokenRotation.breakClasses.length,
      cssModuleTokenRotationBreakClassIds: tokenRotation.breakClasses
        .map((entry) => entry.id)
        .toSorted(),
      cssModuleTokenRotationRelease: tokenRotation.coordinatedRelease,
    },
    null,
    2,
  )}\n`,
);

function collectPages(directory: string): string[] {
  return readdirSync(directory, { withFileTypes: true })
    .flatMap((entry) => {
      const absolutePath = path.join(directory, entry.name);
      if (entry.isDirectory()) return collectPages(absolutePath);
      return /\.(?:md|mdx)$/u.test(entry.name) ? [absolutePath] : [];
    })
    .toSorted();
}

function collectNamedFiles(directory: string, filename: string): string[] {
  return readdirSync(directory, { withFileTypes: true })
    .flatMap((entry) => {
      const absolutePath = path.join(directory, entry.name);
      if (entry.isDirectory()) return collectNamedFiles(absolutePath, filename);
      return entry.name === filename ? [absolutePath] : [];
    })
    .toSorted();
}

function parseFrontmatter(source: string, relativePath: string) {
  assert.ok(source.startsWith("---\n"), `${relativePath} must start with YAML frontmatter`);
  const end = source.indexOf("\n---\n", 4);
  assert.notEqual(end, -1, `${relativePath} frontmatter is not closed`);
  const frontmatter = new Map<string, string>();
  for (const line of source.slice(4, end).split("\n")) {
    const separator = line.indexOf(":");
    assert.ok(separator > 0, `${relativePath} has unsupported multiline frontmatter`);
    frontmatter.set(line.slice(0, separator).trim(), line.slice(separator + 1).trim());
  }
  return { frontmatter, body: source.slice(end + 5) };
}

function requiredScalar(
  frontmatter: ReadonlyMap<string, string>,
  key: string,
  relativePath: string,
) {
  const value = frontmatter.get(key)?.trim();
  assert.ok(value, `${relativePath} has an empty ${key}`);
  return value;
}

function parseInlineList(value: string): string[] {
  assert.match(value, /^\[[^\]]+\]$/u, "frontmatter arrays must use inline YAML syntax");
  return value
    .slice(1, -1)
    .split(",")
    .map((part) => part.trim())
    .filter(Boolean);
}

function markdownLinkTargets(source: string): string[] {
  return [...source.matchAll(/(?<!!)\[[^\]]+\]\(([^)]+)\)/gu)]
    .map((match) => match[1].trim().split(/\s+/u)[0])
    .filter((target) => !/^(?:https?:|mailto:|#)/u.test(target));
}

function verifyLocalLink(sourcePath: string, target: string, relativePath: string) {
  const [pathname] = target.split("#");
  if (!pathname) return;
  const decoded = decodeURIComponent(pathname);
  const resolved = path.resolve(path.dirname(sourcePath), decoded);
  const candidates = [
    resolved,
    `${resolved}.md`,
    `${resolved}.mdx`,
    path.join(resolved, "README.md"),
    path.join(resolved, "index.mdx"),
  ];
  assert.ok(
    candidates.some((candidate) => existsSync(candidate)),
    `${relativePath} links to missing local target ${target}`,
  );
}

function verifyNavigationMetadata(metaPath: string) {
  const directory = path.dirname(metaPath);
  const relativePath = path.relative(repoRoot, metaPath).replaceAll(path.sep, "/");
  const metadata = JSON.parse(readFileSync(metaPath, "utf8")) as {
    pages?: string[];
    pagesIndex?: string;
  };
  assert.ok(Array.isArray(metadata.pages), `${relativePath} must declare a pages array`);

  const declaredPages = metadata.pages.filter((entry) => !entry.startsWith("---")).toSorted();
  const availablePages = readdirSync(directory, { withFileTypes: true })
    .flatMap((entry) => {
      if (entry.isDirectory()) {
        return existsSync(path.join(directory, entry.name, "meta.json")) ? [entry.name] : [];
      }
      const match = /^(.*)\.(?:md|mdx)$/u.exec(entry.name);
      return match ? [match[1]] : [];
    })
    .toSorted();

  assert.deepEqual(
    declaredPages,
    availablePages,
    `${relativePath} pages must exactly match the local content and child navigation trees`,
  );
  if (metadata.pagesIndex) {
    assert.ok(
      declaredPages.includes(metadata.pagesIndex),
      `${relativePath} pagesIndex must name a declared page`,
    );
  }
}

function readFile(relativePath: string): string {
  return readFileSync(path.join(repoRoot, relativePath), "utf8");
}

function readCssModuleTokenRotationContract(): CssModuleTokenRotationContractV1 {
  const source = readFile("docs/releases/css-modules-token-rotation.md");
  const match =
    /<!-- omena-css-module-token-rotation-contract:start -->\s*```json\s*([\s\S]*?)\s*```\s*<!-- omena-css-module-token-rotation-contract:end -->/u.exec(
      source,
    );
  // FALSIFIER: delete either marker or the JSON fence. The authored page can
  // emit that shape, and the site contract must reject it before publication.
  assert.ok(match, "the CSS Modules token rotation contract block is missing");
  return JSON.parse(match[1]) as CssModuleTokenRotationContractV1;
}

function assertFileIncludes(relativePath: string, expected: string, message: string) {
  assert.ok(readFile(relativePath).includes(expected), message);
}
