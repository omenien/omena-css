import { strict as assert } from "node:assert";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

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

const pagePaths = collectPages(docsRoot);
assert.ok(pagePaths.length >= 20, "the public documentation surface unexpectedly shrank");

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

assert.ok(
  existsSync(path.join(repoRoot, "apps/docs/source.config.ts")),
  "the documentation schema must remain in the site application",
);
assert.ok(
  existsSync(path.join(repoRoot, "apps/docs/app/docs/[[...slug]]/page.tsx")),
  "the static documentation route must remain wired",
);
assert.ok(
  existsSync(path.join(repoRoot, "docs/playground.mdx")),
  "the browser playground guide must remain discoverable",
);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "docs.site-contracts",
      pageCount: pagePaths.length,
      generatedPageCount,
      hybridPageCount,
      kindCounts: Object.fromEntries(
        [...kindCounts].toSorted(([left], [right]) => left.localeCompare(right)),
      ),
      linkCount,
      missingMetadataCount: 0,
      brokenLocalLinkCount: 0,
      forbiddenPlanningTermCount: 0,
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
