import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

type FamilyId = "f01" | "f02" | "f03" | "f04" | "f05" | "f06";
type Disposition = "claim" | "compat" | "identityEscalation" | "governanceData";

interface RenameFamilyRow {
  readonly familyId: FamilyId;
  readonly oldForms: readonly string[];
  readonly implementationAccurateNames: readonly string[];
  readonly rationale: string;
}

interface ExpectedClaimRow {
  readonly familyId: FamilyId;
  readonly sourcePath: string;
  readonly line: number;
  readonly lineSha256: string;
  readonly occurrenceInLine: number;
  readonly justification: string;
}

interface CompatibilityDeclarationRow {
  readonly familyId: FamilyId;
  readonly sourcePath: string;
  readonly declarations: readonly string[];
  readonly owner: string;
  readonly removalCondition: string;
}

interface PublishedCrateEscalationRow {
  readonly crate: string;
  readonly owner: "user";
  readonly reentryCondition: string;
}

interface GovernanceRuleRow {
  readonly id: string;
  readonly pathExact?: string;
  readonly pathPrefix?: string;
  readonly familyIds: readonly FamilyId[];
  readonly owner: string;
  readonly justification: string;
  readonly reviewCondition: string;
}

interface RenameMap {
  readonly schemaVersion: "0";
  readonly product: "omena-domain-claims.rename-map";
  readonly renameFamilies: readonly RenameFamilyRow[];
  readonly expectedClaims: readonly ExpectedClaimRow[];
  readonly compatibilityDeclarations: readonly CompatibilityDeclarationRow[];
  readonly publishedCrateEscalations: readonly PublishedCrateEscalationRow[];
  readonly governanceRules: readonly GovernanceRuleRow[];
}

interface FamilySeed {
  readonly familyId: FamilyId;
  readonly pattern: RegExp;
}

interface TextFile {
  readonly sourcePath: string;
  readonly source: string;
}

interface TextDomain {
  readonly files: TextFile[];
  readonly trackedFileCount: number;
  readonly textualFileCount: number;
  readonly nulExcludedCount: number;
  readonly bootstrapUntrackedFileCount: number;
  readonly deletedTrackedFileCount: number;
  readonly fileListSha256: string;
}

interface DeprecatedItemSpan {
  readonly start: number;
  readonly end: number;
  readonly declaration: string;
}

interface Occurrence {
  readonly familyId: FamilyId;
  readonly sourcePath: string;
  readonly matchedText: string;
  readonly line: number;
  readonly column: number;
  readonly lineSha256: string;
  readonly occurrenceInLine: number;
  readonly siteId: string;
  readonly disposition: Disposition;
  readonly declaration?: string;
  readonly governanceRuleId?: string;
}

interface FamilySummary {
  readonly familyId: FamilyId;
  readonly occurrenceCount: number;
  readonly claimCount: number;
  readonly compatCount: number;
  readonly identityEscalationCount: number;
  readonly governanceDataCount: number;
  readonly siteSetSha256: string;
}

interface CensusSnapshot {
  readonly sourceRef: string | null;
  readonly domain: {
    readonly trackedFileCount: number;
    readonly textualFileCount: number;
    readonly nulExcludedCount: number;
    readonly bootstrapUntrackedFileCount: number;
    readonly deletedTrackedFileCount: number;
    readonly fileListSha256: string;
  };
  readonly occurrenceCount: number;
  readonly dispositionCounts: Readonly<Record<Disposition, number>>;
  readonly families: readonly FamilySummary[];
  readonly siteSetSha256: string;
  readonly claimSiteSetSha256: string;
  readonly compatDeclarationSetSha256: string;
  readonly identityCrateSetSha256: string;
  readonly governanceRuleSetSha256: string;
}

interface CensusArtifact {
  readonly schemaVersion: "0";
  readonly product: "omena-domain-claims.census";
  readonly entry: CensusSnapshot;
  readonly exit: CensusSnapshot;
}

interface ScanResult {
  readonly snapshot: CensusSnapshot;
  readonly occurrences: readonly Occurrence[];
  readonly publishedPackages: ReadonlySet<string>;
  readonly governanceRuleMatches: ReadonlyMap<string, number>;
}

const repoRoot = process.cwd();
const mapPath = "rust/omena-domain-claim-rename-map.json";
const artifactPath = "rust/omena-domain-claim-census.json";
const entrySourceRef = "4638d90c84e878ae7c82d6baa4e316b92dd9a5b3";
const entrySnapshotSha256 = "9bc92b8d8417bb32c796fa0f2262d92fd043d83a6d359ffa388836a1d2c957dc";

// These lexical seeds are deliberately declared here rather than loaded from the
// authored rename map. Deleting a map row therefore cannot shrink the scan domain.
const siteFamilyStem = ["si", "te"].join("");
const siteFamilyPattern = new RegExp(
  [
    `(?:verify|categorical)[.-]${siteFamilyStem}[.-]stability`,
    `(?:cascade|categorical)[._/ -]?${siteFamilyStem}`,
    `(?<![A-Za-z0-9_])${siteFamilyStem}_(?:id(?![A-Za-z0-9_])|(?:axis|axiom|truth|object|theorem|gluing|cover|product|module)(?:_[A-Za-z0-9]+)*)`,
    `(?<![A-Za-z0-9_])${siteFamilyStem}(?:Axiom|Axis|Truth|Object|Theorem|Gluing|Cover)[A-Za-z0-9_]*`,
    `\\b(?:mod|module|as)\\s+${siteFamilyStem}\\b`,
    `(?<![A-Za-z0-9_-])${siteFamilyStem}\\s*::`,
    `(?<![A-Za-z0-9_-])${siteFamilyStem}\\.rs\\b`,
    `(?<![A-Za-z0-9_-])${siteFamilyStem}[ -](?:axiom|theorem|gluing|covering|product|module)\\b`,
  ].join("|"),
  "giu",
);
const familySeeds: readonly FamilySeed[] = [
  { familyId: "f01", pattern: /finite[-_ ]?height/giu },
  { familyId: "f02", pattern: /belief[-_ ]?propagation/giu },
  { familyId: "f03", pattern: new RegExp(`if${"ds"}`, "giu") },
  { familyId: "f04", pattern: new RegExp(`law${"vere"}`, "giu") },
  {
    familyId: "f05",
    pattern: new RegExp(`(?:pre|co)?sh${"eaf"}|${siteFamilyPattern.source}`, "giu"),
  },
  {
    familyId: "f06",
    pattern: /(?:rg[-_ ]?flow|renormalization[-_ ]?group)/giu,
  },
];
const familyIds = familySeeds.map(({ familyId }) => familyId);

const args = new Set(process.argv.slice(2));
const writeMode = args.has("--write");
const reportSourceRef = valueAfter("--report-source-ref");
const reportCurrent = args.has("--report-current");
const listClaimSites = args.has("--list-claim-sites");
const snapshotJsonMode = args.has("--snapshot-json");
const injectionFlags = [
  "--inject-map-row-deletion",
  "--inject-new-claim",
  "--inject-missing-compat-owner",
  "--inject-artifact-hand-edit",
].filter((flag) => args.has(flag));

assert.ok(injectionFlags.length <= 1, "only one permanent injection may run at a time");
assert.ok(
  !(writeMode && injectionFlags.length > 0),
  "--write cannot be combined with a permanent injection",
);
assert.ok(!(writeMode && reportCurrent), "--write cannot be combined with --report-current");
assert.ok(
  !snapshotJsonMode || reportSourceRef !== undefined,
  "--snapshot-json requires --report-source-ref",
);

let activeRenameMap = readJson<RenameMap>(mapPath);
if (args.has("--inject-map-row-deletion")) {
  assert.ok(
    activeRenameMap.compatibilityDeclarations.length > 0,
    "map-row deletion injection requires a live compatibility declaration row",
  );
  activeRenameMap = {
    ...activeRenameMap,
    compatibilityDeclarations: activeRenameMap.compatibilityDeclarations.slice(1),
  };
}
if (args.has("--inject-missing-compat-owner")) {
  assert.ok(
    activeRenameMap.compatibilityDeclarations.length > 0,
    "missing-owner injection requires a compatibility declaration",
  );
  activeRenameMap = {
    ...activeRenameMap,
    compatibilityDeclarations: activeRenameMap.compatibilityDeclarations.map((row, index) =>
      index === 0 ? { ...row, owner: "" } : row,
    ),
  };
}

const mapErrors = validateRenameMap(activeRenameMap);
validateCompatibilitySpanParser();
validateSiteFamilySeedBoundary();
validateIdentityPatternConstructionBoundary();

if (reportSourceRef !== undefined) {
  assert.equal(injectionFlags.length, 0, "source-ref reporting does not accept injections");
  assert.equal(mapErrors.length, 0, mapErrors.join("\n"));
  const resolved = resolveCommit(reportSourceRef);
  const result = scanDomain(readSourceRefDomain(resolved), activeRenameMap, resolved);
  process.stdout.write(
    snapshotJsonMode
      ? `${JSON.stringify(result.snapshot)}\n`
      : `${renderSnapshot(result.snapshot)}\n${renderClaimFiles(result)}\n${renderClaimSites(result)}\n`,
  );
  process.exit(0);
}

let exitDomain = readWorkingTreeDomain();
if (writeMode) {
  assert.equal(
    exitDomain.bootstrapUntrackedFileCount,
    0,
    "official --write requires every intended census-domain file to be staged first",
  );
  assert.equal(
    exitDomain.deletedTrackedFileCount,
    0,
    "official --write requires tracked deletions and renames to be staged first",
  );
}
if (args.has("--inject-new-claim")) {
  const injectedFile = {
    sourcePath: "__injection__/new-claim.rs",
    source: `/// ${"belief"} propagation is the implemented algorithm.\n`,
  };
  const files = [...exitDomain.files, injectedFile].toSorted((left, right) =>
    left.sourcePath.localeCompare(right.sourcePath),
  );
  exitDomain = {
    ...exitDomain,
    files,
    trackedFileCount: exitDomain.trackedFileCount + 1,
    textualFileCount: exitDomain.textualFileCount + 1,
    fileListSha256: sha256(`${files.map(({ sourcePath }) => sourcePath).join("\n")}\n`),
  };
}
const exitResult = scanDomain(exitDomain, activeRenameMap, null);
if (reportCurrent) {
  assert.equal(injectionFlags.length, 0, "current reporting does not accept injections");
  assert.equal(mapErrors.length, 0, mapErrors.join("\n"));
  process.stdout.write(
    `${renderSnapshot(exitResult.snapshot)}\n${renderClaimFiles(exitResult)}\n` +
      `${listClaimSites ? `${renderClaimSites(exitResult)}\n` : ""}` +
      `${renderCompatDeclarations(exitResult)}\n`,
  );
  process.exit(0);
}
const validationErrors = [
  ...mapErrors,
  ...validatePublishedCrateEscalations(activeRenameMap, exitResult.publishedPackages),
  ...validateExpectedClaims(activeRenameMap, exitResult.occurrences),
  ...validateCompatibilityDeclarations(activeRenameMap, exitResult.occurrences),
  ...validateGovernanceRules(activeRenameMap, exitResult.governanceRuleMatches),
];

if (validationErrors.length > 0) {
  throw new Error(`domain claim census validation failed:\n${validationErrors.join("\n")}`);
}

let committedArtifact: CensusArtifact | undefined;
const entrySnapshot = writeMode
  ? scanDomain(readSourceRefDomain(entrySourceRef), activeRenameMap, entrySourceRef).snapshot
  : (committedArtifact = readJson<CensusArtifact>(artifactPath)).entry;
validateFrozenEntrySnapshot(entrySnapshot);

const artifact: CensusArtifact = {
  schemaVersion: "0",
  product: "omena-domain-claims.census",
  entry: entrySnapshot,
  exit: exitResult.snapshot,
};
const serialized = `${JSON.stringify(artifact, null, 2)}\n`;

if (writeMode) {
  fs.writeFileSync(path.join(repoRoot, artifactPath), serialized);
  execFileSync("pnpm", ["exec", "oxfmt", artifactPath], {
    cwd: repoRoot,
    stdio: "inherit",
  });
} else {
  let committed = committedArtifact;
  assert.ok(committed, "committed census artifact must be loaded in check mode");
  if (args.has("--inject-artifact-hand-edit")) {
    committed = {
      ...committed,
      exit: { ...committed.exit, occurrenceCount: committed.exit.occurrenceCount + 1 },
    };
  }
  assert.deepEqual(
    committed,
    artifact,
    "domain claim census artifact is stale or hand-edited; run the official --write command",
  );
}

process.stdout.write(
  `domain claim census OK\nentry ${renderSnapshot(entrySnapshot)}\nexit ${renderSnapshot(exitResult.snapshot)}\n`,
);

function readWorkingTreeDomain(): TextDomain {
  const tracked = git(["ls-files", "-z"]).split("\0").filter(Boolean).toSorted();
  const untracked = git(["ls-files", "--others", "--exclude-standard", "-z"])
    .split("\0")
    .filter(Boolean)
    .toSorted();
  const existingTracked = tracked.filter((sourcePath) =>
    fs.existsSync(path.join(repoRoot, sourcePath)),
  );
  const domainPaths = [...new Set([...existingTracked, ...untracked])].toSorted();
  const files = domainPaths
    .map((sourcePath) => ({ sourcePath, bytes: fs.readFileSync(path.join(repoRoot, sourcePath)) }))
    .filter(({ bytes }) => !bytes.includes(0))
    .map(({ sourcePath, bytes }) => ({ sourcePath, source: bytes.toString("utf8") }));
  return {
    files,
    trackedFileCount: tracked.length,
    textualFileCount: files.length,
    nulExcludedCount: domainPaths.length - files.length,
    bootstrapUntrackedFileCount: untracked.length,
    deletedTrackedFileCount: tracked.length - existingTracked.length,
    fileListSha256: sha256(`${files.map(({ sourcePath }) => sourcePath).join("\n")}\n`),
  };
}

function readSourceRefDomain(sourceRef: string): TextDomain {
  const resolved = resolveCommit(sourceRef);
  const tracked = git(["ls-tree", "-r", "--name-only", "-z", resolved])
    .split("\0")
    .filter(Boolean)
    .toSorted();
  const files = tracked
    .map((sourcePath) => ({ sourcePath, bytes: gitBuffer(["show", `${resolved}:${sourcePath}`]) }))
    .filter(({ bytes }) => !bytes.includes(0))
    .map(({ sourcePath, bytes }) => ({ sourcePath, source: bytes.toString("utf8") }));
  return {
    files,
    trackedFileCount: tracked.length,
    textualFileCount: files.length,
    nulExcludedCount: tracked.length - files.length,
    bootstrapUntrackedFileCount: 0,
    deletedTrackedFileCount: 0,
    fileListSha256: sha256(`${files.map(({ sourcePath }) => sourcePath).join("\n")}\n`),
  };
}

function scanDomain(
  domain: TextDomain,
  renameMap: RenameMap,
  sourceRef: string | null,
): ScanResult {
  const { files } = domain;
  const publishedPackages = publishedCargoPackages(files);
  const escalatedCrates = new Set(renameMap.publishedCrateEscalations.map((row) => row.crate));
  const identityPatterns = identityPatternsByFamily(publishedPackages, escalatedCrates);
  validateIdentityPatternBoundary(identityPatterns);
  const deprecatedRustDeclarations = new Set(
    files
      .filter(({ sourcePath }) => sourcePath.endsWith(".rs"))
      .flatMap(({ sourcePath, source }) =>
        compatibilityItemSpans(sourcePath, source).map(({ declaration }) => declaration),
      ),
  );
  const occurrences: Occurrence[] = [];
  const governanceRuleMatches = new Map(renameMap.governanceRules.map((row) => [row.id, 0]));

  for (const { sourcePath, source } of files) {
    const lineStarts = sourceLineStarts(source);
    const compatSpans = [
      ...compatibilityItemSpans(sourcePath, source),
      ...generatedDeprecatedDeclarationCompatibilitySpans(
        sourcePath,
        source,
        deprecatedRustDeclarations,
      ),
    ];
    const identitySpans = new Map(
      familyIds.map((familyId) => [
        familyId,
        [...source.matchAll(identityPatterns.get(familyId) ?? /$a/gu)].map((match) => ({
          start: match.index,
          end: match.index + match[0].length,
        })),
      ]),
    );
    const perLineFamilyOrdinal = new Map<string, number>();

    for (const { familyId, pattern } of familySeeds) {
      for (const match of source.matchAll(freshPattern(pattern))) {
        const start = match.index;
        const lineIndex = lineIndexAtOffset(lineStarts, start);
        const lineStart = lineStarts[lineIndex];
        const lineEnd = source.indexOf("\n", lineStart);
        const lineSource = source.slice(lineStart, lineEnd === -1 ? source.length : lineEnd);
        const ordinalKey = `${familyId}:${lineIndex}`;
        const occurrenceInLine = perLineFamilyOrdinal.get(ordinalKey) ?? 0;
        perLineFamilyOrdinal.set(ordinalKey, occurrenceInLine + 1);
        const identity = identitySpans
          .get(familyId)
          ?.some((span) => start >= span.start && start + match[0].length <= span.end);
        const compat = compatSpans.find(
          (span) => start >= span.start && start + match[0].length <= span.end,
        );
        const governance = renameMap.governanceRules.find(
          (rule) =>
            rule.familyIds.includes(familyId) &&
            ((rule.pathExact !== undefined && rule.pathExact === sourcePath) ||
              (rule.pathPrefix !== undefined && sourcePath.startsWith(rule.pathPrefix))),
        );
        const disposition: Disposition = identity
          ? "identityEscalation"
          : compat !== undefined
            ? "compat"
            : governance !== undefined
              ? "governanceData"
              : "claim";
        if (disposition === "governanceData" && governance !== undefined) {
          governanceRuleMatches.set(
            governance.id,
            (governanceRuleMatches.get(governance.id) ?? 0) + 1,
          );
        }
        const lineSha256 = sha256(lineSource);
        occurrences.push({
          familyId,
          sourcePath,
          matchedText: match[0],
          line: lineIndex + 1,
          column: start - lineStart + 1,
          lineSha256,
          occurrenceInLine,
          siteId: sha256(
            `${familyId}\0${sourcePath}\0${lineIndex + 1}\0${lineSha256}\0${occurrenceInLine}`,
          ),
          disposition,
          ...(compat === undefined ? {} : { declaration: compat.declaration }),
          ...(governance === undefined ? {} : { governanceRuleId: governance.id }),
        });
      }
    }
  }

  occurrences.sort((left, right) =>
    `${left.familyId}:${left.sourcePath}:${left.line}:${left.column}`.localeCompare(
      `${right.familyId}:${right.sourcePath}:${right.line}:${right.column}`,
    ),
  );
  const dispositionCounts = Object.fromEntries(
    (["claim", "compat", "identityEscalation", "governanceData"] as const).map((disposition) => [
      disposition,
      occurrences.filter((row) => row.disposition === disposition).length,
    ]),
  ) as Record<Disposition, number>;
  const families = familyIds.map((familyId): FamilySummary => {
    const rows = occurrences.filter((row) => row.familyId === familyId);
    return {
      familyId,
      occurrenceCount: rows.length,
      claimCount: rows.filter((row) => row.disposition === "claim").length,
      compatCount: rows.filter((row) => row.disposition === "compat").length,
      identityEscalationCount: rows.filter((row) => row.disposition === "identityEscalation")
        .length,
      governanceDataCount: rows.filter((row) => row.disposition === "governanceData").length,
      siteSetSha256: sha256(
        `${rows
          .map((row) => row.siteId)
          .toSorted()
          .join("\n")}\n`,
      ),
    };
  });
  const compatDeclarations = occurrences
    .filter((row) => row.disposition === "compat")
    .map((row) => `${row.familyId}:${row.sourcePath}:${row.declaration}`)
    .filter((row, index, all) => all.indexOf(row) === index)
    .toSorted();
  const identityCrates = renameMap.publishedCrateEscalations
    .map((row) => row.crate)
    .filter((crateName) => publishedPackages.has(crateName))
    .toSorted();
  const matchedGovernanceRules = [...governanceRuleMatches.entries()]
    .filter(([, count]) => count > 0)
    .map(([id]) => id)
    .toSorted();
  const snapshot: CensusSnapshot = {
    sourceRef,
    domain: {
      trackedFileCount: domain.trackedFileCount,
      textualFileCount: domain.textualFileCount,
      nulExcludedCount: domain.nulExcludedCount,
      bootstrapUntrackedFileCount: domain.bootstrapUntrackedFileCount,
      deletedTrackedFileCount: domain.deletedTrackedFileCount,
      fileListSha256: domain.fileListSha256,
    },
    occurrenceCount: occurrences.length,
    dispositionCounts,
    families,
    siteSetSha256: sha256(
      `${occurrences
        .map((row) => row.siteId)
        .toSorted()
        .join("\n")}\n`,
    ),
    claimSiteSetSha256: sha256(
      `${occurrences
        .filter((row) => row.disposition === "claim")
        .map((row) => row.siteId)
        .toSorted()
        .join("\n")}\n`,
    ),
    compatDeclarationSetSha256: sha256(`${compatDeclarations.join("\n")}\n`),
    identityCrateSetSha256: sha256(`${identityCrates.join("\n")}\n`),
    governanceRuleSetSha256: sha256(`${matchedGovernanceRules.join("\n")}\n`),
  };
  assert.equal(
    Object.values(dispositionCounts).reduce((sum, count) => sum + count, 0),
    occurrences.length,
    "every seeded occurrence must have exactly one disposition",
  );
  return { snapshot, occurrences, publishedPackages, governanceRuleMatches };
}

function validateRenameMap(renameMap: RenameMap): string[] {
  const errors: string[] = [];
  if (renameMap.schemaVersion !== "0") errors.push("rename map schemaVersion must be 0");
  if (renameMap.product !== "omena-domain-claims.rename-map") {
    errors.push("rename map product is not omena-domain-claims.rename-map");
  }
  const observedFamilies = renameMap.renameFamilies.map((row) => row.familyId).toSorted();
  if (JSON.stringify(observedFamilies) !== JSON.stringify([...familyIds].toSorted())) {
    errors.push(
      `rename map family rows differ from the independent lexical seeds: expected=${familyIds.join(",")} observed=${observedFamilies.join(",")}`,
    );
  }
  for (const row of renameMap.renameFamilies) {
    if (row.oldForms.length === 0) errors.push(`${row.familyId}: oldForms is empty`);
    if (row.implementationAccurateNames.length === 0) {
      errors.push(`${row.familyId}: implementationAccurateNames is empty`);
    }
    if (row.rationale.trim().length === 0) errors.push(`${row.familyId}: rationale is empty`);
  }
  for (const row of renameMap.expectedClaims) {
    if (!familyIds.includes(row.familyId)) {
      errors.push(`expected claim has unknown family: ${row.familyId}`);
    }
    if (row.justification.trim().length === 0) {
      errors.push(`expected claim ${expectedClaimKey(row)} has no justification`);
    }
  }
  for (const row of renameMap.compatibilityDeclarations) {
    if (!familyIds.includes(row.familyId)) {
      errors.push(`compatibility declaration has unknown family: ${row.familyId}`);
    }
    const rowLabel = `${row.familyId}:${row.sourcePath}`;
    if (row.declarations.length === 0) {
      errors.push(`compatibility group ${rowLabel} has no declarations`);
    }
    if (new Set(row.declarations).size !== row.declarations.length) {
      errors.push(`compatibility group ${rowLabel} repeats a declaration`);
    }
    if (row.owner.trim().length === 0) {
      errors.push(`compatibility group ${rowLabel} has no owner`);
    }
    if (
      !/1\.0/u.test(row.removalCondition) ||
      !/migration/iu.test(row.removalCondition) ||
      !/zero[\s\S]*non-compat/iu.test(row.removalCondition)
    ) {
      errors.push(
        `compatibility group ${rowLabel} must require not-before-1.0, migration, and zero non-compat use`,
      );
    }
  }
  for (const row of renameMap.publishedCrateEscalations) {
    if (row.owner !== "user") errors.push(`crate escalation ${row.crate} is not user-owned`);
    if (row.reentryCondition.trim().length === 0) {
      errors.push(`crate escalation ${row.crate} has no re-entry condition`);
    }
  }
  for (const row of renameMap.governanceRules) {
    if ((row.pathExact === undefined) === (row.pathPrefix === undefined)) {
      errors.push(`governance rule ${row.id} must set exactly one path selector`);
    }
    if (row.familyIds.length === 0) errors.push(`governance rule ${row.id} has no families`);
    if (row.familyIds.some((familyId) => !familyIds.includes(familyId))) {
      errors.push(`governance rule ${row.id} has an unknown family`);
    }
    if (new Set(row.familyIds).size !== row.familyIds.length) {
      errors.push(`governance rule ${row.id} repeats a family`);
    }
    if (row.owner.trim().length === 0) errors.push(`governance rule ${row.id} has no owner`);
    if (row.justification.trim().length === 0) {
      errors.push(`governance rule ${row.id} has no justification`);
    }
    if (row.reviewCondition.trim().length === 0) {
      errors.push(`governance rule ${row.id} has no review condition`);
    }
  }
  errors.push(...duplicateErrors(renameMap.expectedClaims.map(expectedClaimKey), "expected claim"));
  errors.push(
    ...duplicateErrors(
      renameMap.compatibilityDeclarations.flatMap(compatibilityKeys),
      "compatibility declaration",
    ),
  );
  errors.push(
    ...duplicateErrors(
      renameMap.publishedCrateEscalations.map((row) => row.crate),
      "crate escalation",
    ),
  );
  errors.push(
    ...duplicateErrors(
      renameMap.governanceRules.map((row) => row.id),
      "governance rule",
    ),
  );
  return errors;
}

function validateExpectedClaims(
  renameMap: RenameMap,
  occurrences: readonly Occurrence[],
): string[] {
  const expected = new Map(renameMap.expectedClaims.map((row) => [expectedClaimKey(row), row]));
  const observed = new Map(
    occurrences
      .filter((row) => row.disposition === "claim")
      .map((row) => [occurrenceClaimKey(row), row]),
  );
  const errors: string[] = [];
  for (const [key, row] of observed) {
    if (!expected.has(key)) {
      errors.push(
        `matched occurrence with no authored claim row: ${row.familyId} ${row.sourcePath}:${row.line}:${row.column} lineSha256=${row.lineSha256} occurrenceInLine=${row.occurrenceInLine}`,
      );
    }
  }
  for (const [key, row] of expected) {
    if (!observed.has(key)) {
      errors.push(
        `authored claim row with no matched occurrence: ${row.familyId} ${row.sourcePath}:${row.line} lineSha256=${row.lineSha256} occurrenceInLine=${row.occurrenceInLine}`,
      );
    }
  }
  return errors;
}

function validateCompatibilityDeclarations(
  renameMap: RenameMap,
  occurrences: readonly Occurrence[],
): string[] {
  const expected = new Set(renameMap.compatibilityDeclarations.flatMap(compatibilityKeys));
  const observed = new Map<string, Occurrence>();
  for (const row of occurrences.filter((candidate) => candidate.disposition === "compat")) {
    assert.ok(row.declaration, "compat occurrence must name its declaration");
    observed.set(`${row.familyId}:${row.sourcePath}:${row.declaration}`, row);
  }
  const errors: string[] = [];
  for (const [key, row] of observed) {
    if (!expected.has(key)) {
      errors.push(
        `deprecated compatibility occurrence has no authored owner/removal row: ${key} at ${row.sourcePath}:${row.line}:${row.column}`,
      );
    }
  }
  for (const key of expected) {
    if (!observed.has(key)) {
      errors.push(`authored compatibility row has no deprecated declaration occurrence: ${key}`);
    }
  }
  return errors;
}

function validatePublishedCrateEscalations(
  renameMap: RenameMap,
  publishedPackages: ReadonlySet<string>,
): string[] {
  const expected = [
    `omena-${"categorical"}`,
    `omena-${["law", "vere"].join("")}`,
    `omena-${"rg"}-flow`,
    `omena-${"streaming"}-${["if", "ds"].join("")}`,
    `omena-${"variational"}`,
  ].toSorted();
  const observed = renameMap.publishedCrateEscalations.map((row) => row.crate).toSorted();
  const errors: string[] = [];
  if (JSON.stringify(expected) !== JSON.stringify(observed)) {
    errors.push(
      `published crate escalation rows differ: expected=${expected.join(",")} observed=${observed.join(",")}`,
    );
  }
  for (const crateName of expected) {
    if (!publishedPackages.has(crateName)) {
      errors.push(`crate escalation no longer names a published workspace package: ${crateName}`);
    }
  }
  return errors;
}

function validateGovernanceRules(
  renameMap: RenameMap,
  ruleMatches: ReadonlyMap<string, number>,
): string[] {
  return renameMap.governanceRules.flatMap((row) =>
    (ruleMatches.get(row.id) ?? 0) === 0
      ? [`authored governance rule matched no occurrence: ${row.id}`]
      : [],
  );
}

function compatibilityItemSpans(sourcePath: string, source: string): DeprecatedItemSpan[] {
  if (/^rust\/crates\/[^/]+\/Cargo\.toml$/u.test(sourcePath)) {
    return cargoFeatureCompatibilitySpans(source);
  }
  if (/\.(?:[cm]?[jt]s|[jt]sx)$/u.test(sourcePath)) {
    return deprecatedScriptItemSpans(source);
  }
  if (!sourcePath.endsWith(".rs")) return [];
  const lineStarts = sourceLineStarts(source);
  const lines = source.split(/(?<=\n)/u);
  const spans: DeprecatedItemSpan[] = [];
  for (let index = 0; index < lines.length; index += 1) {
    const deprecatedAttribute = /#\s*\[\s*deprecated\b/u.test(lines[index]);
    const compatibilityReexport = /#\s*\[\s*allow\s*\(\s*deprecated\s*\)\s*\]/u.test(lines[index]);
    if (!deprecatedAttribute && !compatibilityReexport) continue;
    const startLine = index;
    while (index < lines.length && !/\]\s*$/u.test(lines[index].trimEnd())) index += 1;
    let itemLine = index + 1;
    while (itemLine < lines.length && /^(?:\s*#\[|\s*$)/u.test(lines[itemLine])) itemLine += 1;
    const itemSource = lines[itemLine] ?? "";
    const declarationMatch =
      /^\s*(?:pub(?:\([^)]*\))?\s+)?(type|use|mod|const\s+fn|fn|const|static|struct|enum)\s+([A-Za-z_][A-Za-z0-9_]*)/u.exec(
        itemSource,
      );
    const groupedUse = /^\s*(?:pub(?:\([^)]*\))?\s+)?use\s*\{/u.test(itemSource);
    const variantMatch = /^\s*([A-Za-z_][A-Za-z0-9_]*)\s*(?:[({=,])/u.exec(itemSource);
    const fieldMatch = /^\s*(?:pub(?:\([^)]*\))?\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:/u.exec(
      itemSource,
    );
    if (compatibilityReexport && declarationMatch?.[1] !== "use" && !groupedUse) continue;
    if (!declarationMatch && !groupedUse && !variantMatch && !fieldMatch) continue;
    let endLine = itemLine;
    let braceDepth = 0;
    let sawBrace = false;
    for (; endLine < lines.length; endLine += 1) {
      for (const character of lines[endLine]) {
        if (character === "{") {
          braceDepth += 1;
          sawBrace = true;
        } else if (character === "}") {
          braceDepth -= 1;
        }
      }
      if (
        (sawBrace && braceDepth === 0) ||
        (!sawBrace && /;\s*$/u.test(lines[endLine].trimEnd())) ||
        (!sawBrace &&
          (variantMatch !== null || fieldMatch !== null) &&
          /,\s*$/u.test(lines[endLine].trimEnd()))
      ) {
        break;
      }
    }
    const itemStart = lineStarts[itemLine];
    const itemEnd = endLine + 1 < lineStarts.length ? lineStarts[endLine + 1] : source.length;
    if (groupedUse || declarationMatch?.[1] === "use") {
      const useSource = source.slice(itemStart, itemEnd);
      for (const { pattern } of familySeeds) {
        for (const match of useSource.matchAll(freshPattern(pattern))) {
          const matchStart = itemStart + match.index;
          let tokenStart = matchStart;
          let tokenEnd = matchStart + match[0].length;
          while (tokenStart > itemStart && /[A-Za-z0-9_]/u.test(source[tokenStart - 1])) {
            tokenStart -= 1;
          }
          while (tokenEnd < itemEnd && /[A-Za-z0-9_]/u.test(source[tokenEnd])) tokenEnd += 1;
          spans.push({
            start: tokenStart,
            end: tokenEnd,
            declaration: source.slice(tokenStart, tokenEnd),
          });
        }
      }
    } else {
      spans.push({
        start: lineStarts[startLine],
        end: itemEnd,
        declaration: declarationMatch?.[2] ?? variantMatch?.[1] ?? fieldMatch?.[1] ?? "",
      });
    }
    index = endLine;
  }
  return spans;
}

function cargoFeatureCompatibilitySpans(source: string): DeprecatedItemSpan[] {
  const lineStarts = sourceLineStarts(source);
  const lines = source.split(/(?<=\n)/u);
  const spans: DeprecatedItemSpan[] = [];
  let inFeatures = false;
  for (let index = 0; index < lines.length; index += 1) {
    const heading = /^\s*\[([^\]]+)\]\s*$/u.exec(lines[index].trimEnd());
    if (heading !== null) {
      inFeatures = heading[1] === "features";
      continue;
    }
    if (!inFeatures) continue;
    const owner = /^\s*#\s*compatibility-owner:\s*(\S[\s\S]*?)\s*$/u.exec(lines[index])?.[1];
    if (owner === undefined) continue;
    const removal = /^\s*#\s*compatibility-removal:\s*(\S[\s\S]*?)\s*$/u.exec(
      lines[index + 1] ?? "",
    )?.[1];
    if (
      removal === undefined ||
      !/not before 1\.0/iu.test(removal) ||
      !/migration/iu.test(removal) ||
      !/zero[\s\S]*non-compat/iu.test(removal)
    ) {
      continue;
    }
    const assignmentLine = index + 2;
    const assignment = /^\s*(?:"([^"]+)"|([A-Za-z0-9_-]+))\s*=/u.exec(lines[assignmentLine] ?? "");
    const declaration = assignment?.[1] ?? assignment?.[2];
    if (declaration === undefined) continue;
    let endLine = assignmentLine;
    let bracketDepth = 0;
    let sawBracket = false;
    for (; endLine < lines.length; endLine += 1) {
      for (const character of lines[endLine]) {
        if (character === "[") {
          bracketDepth += 1;
          sawBracket = true;
        } else if (character === "]") {
          bracketDepth -= 1;
        }
      }
      if (!sawBracket || bracketDepth === 0) break;
    }
    spans.push({
      start: lineStarts[assignmentLine],
      end: endLine + 1 < lineStarts.length ? lineStarts[endLine + 1] : source.length,
      declaration,
    });
    index = endLine;
  }
  return spans;
}

function generatedDeprecatedDeclarationCompatibilitySpans(
  sourcePath: string,
  source: string,
  deprecatedRustDeclarations: ReadonlySet<string>,
): DeprecatedItemSpan[] {
  if (sourcePath === "rust/omena-rust-semver-intent.json") {
    return semverIntentDeprecatedDeclarationCompatibilitySpans(source, deprecatedRustDeclarations);
  }
  if (
    !/^rust\/crates\/[^/]+\/tests\/snapshots\/public-api\.txt$/u.test(sourcePath) &&
    sourcePath !== "rust/omena-response-surface-split-census.json"
  ) {
    return [];
  }
  return [...source.matchAll(/[A-Za-z_][A-Za-z0-9_]*/gu)]
    .filter((match) => deprecatedRustDeclarations.has(match[0]))
    .map((match) => ({
      start: match.index,
      end: match.index + match[0].length,
      declaration: match[0],
    }));
}

function semverIntentDeprecatedDeclarationCompatibilitySpans(
  source: string,
  deprecatedRustDeclarations: ReadonlySet<string>,
): DeprecatedItemSpan[] {
  const spans: DeprecatedItemSpan[] = [];
  const evidenceArray = /(?<!\\)"(?:evidenceNeedles|expectedWitnesses|needles)"\s*:\s*\[/gu;
  for (const match of source.matchAll(evidenceArray)) {
    const arrayStart = match.index + match[0].lastIndexOf("[");
    const arrayEnd = matchingJsonArrayEnd(source, arrayStart);
    const arraySource = source.slice(arrayStart + 1, arrayEnd);
    const evidenceValues = JSON.parse(source.slice(arrayStart, arrayEnd + 1)) as unknown;
    assert.ok(
      Array.isArray(evidenceValues) && evidenceValues.every((value) => typeof value === "string"),
      "semver intent evidence compatibility must be carried by a JSON string array",
    );
    for (const stringMatch of arraySource.matchAll(/"(?:\\.|[^"\\])*"/gu)) {
      const stringStart = arrayStart + 1 + stringMatch.index;
      const linkedTokens = [...stringMatch[0].matchAll(/[A-Za-z_][A-Za-z0-9_]*/gu)].filter(
        (token) => deprecatedRustDeclarations.has(token[0]),
      );
      for (const token of linkedTokens) {
        spans.push({
          start: stringStart + token.index,
          end: stringStart + token.index + token[0].length,
          declaration: token[0],
        });
        for (const { pattern } of familySeeds) {
          for (const seeded of stringMatch[0].matchAll(freshPattern(pattern))) {
            const tokenEnd = token.index + token[0].length;
            const seededEnd = seeded.index + seeded[0].length;
            const overlaps = seeded.index < tokenEnd && seededEnd > token.index;
            const extendsPastToken = seeded.index < token.index || seededEnd > tokenEnd;
            if (!overlaps || !extendsPastToken) continue;
            spans.push({
              start: stringStart + seeded.index,
              end: stringStart + seededEnd,
              declaration: token[0],
            });
          }
        }
      }
    }
  }
  return spans;
}

function matchingJsonArrayEnd(source: string, arrayStart: number): number {
  let depth = 0;
  let inString = false;
  let escaped = false;
  for (let index = arrayStart; index < source.length; index += 1) {
    const character = source[index];
    if (inString) {
      if (escaped) escaped = false;
      else if (character === "\\") escaped = true;
      else if (character === '"') inString = false;
      continue;
    }
    if (character === '"') inString = true;
    else if (character === "[") depth += 1;
    else if (character === "]") {
      depth -= 1;
      if (depth === 0) return index;
    }
  }
  throw new Error(`unterminated JSON evidence array at offset ${arrayStart}`);
}

function deprecatedScriptItemSpans(source: string): DeprecatedItemSpan[] {
  const lineStarts = sourceLineStarts(source);
  const lines = source.split(/(?<=\n)/u);
  const spans: DeprecatedItemSpan[] = [];
  for (let index = 0; index < lines.length; index += 1) {
    if (!/\/\*\*/u.test(lines[index])) continue;
    const startLine = index;
    let deprecated = /@deprecated\b/u.test(lines[index]);
    while (index < lines.length && !/\*\//u.test(lines[index])) {
      index += 1;
      deprecated ||= /@deprecated\b/u.test(lines[index] ?? "");
    }
    if (!deprecated) continue;
    let itemLine = index + 1;
    while (itemLine < lines.length && /^\s*$/u.test(lines[itemLine])) itemLine += 1;
    const declaration =
      /^\s*(?:export\s+)?(?:declare\s+)?(?:const|let|function|class|interface|type)\s+([A-Za-z_$][A-Za-z0-9_$]*)/u.exec(
        lines[itemLine] ?? "",
      )?.[1];
    if (declaration === undefined) continue;
    let endLine = itemLine;
    let braceDepth = 0;
    let sawBrace = false;
    for (; endLine < lines.length; endLine += 1) {
      for (const character of lines[endLine]) {
        if (character === "{") {
          braceDepth += 1;
          sawBrace = true;
        } else if (character === "}") {
          braceDepth -= 1;
        }
      }
      if (
        (sawBrace && braceDepth === 0) ||
        (!sawBrace && /;\s*$/u.test(lines[endLine].trimEnd()))
      ) {
        break;
      }
    }
    spans.push({
      start: lineStarts[startLine],
      end: endLine + 1 < lineStarts.length ? lineStarts[endLine + 1] : source.length,
      declaration,
    });
    index = endLine;
  }
  return spans;
}

function validateCompatibilitySpanParser(): void {
  const mixed = ["If", "ds"].join("");
  const upper = ["IF", "DS"].join("_");
  const legacyModuleName = ["si", "te"].join("");
  const source = [
    `#[deprecated(since = "0.4.0", note = "compat")]\npub struct Legacy${mixed}StructV0 { value: u8 }\n`,
    `#[deprecated]\npub use { CurrentTypeV0 as Legacy${mixed}AliasV0 };\n`,
    `#[deprecated]\nLegacy${mixed}Variant,\n`,
    `#[deprecated]\nconst LEGACY_${upper}_WIRE: &str = "legacy";\n`,
    `#[deprecated]\npub const fn legacy_${["if", "ds"].join("_")}_rank() -> u8 { 1 }\n`,
    `#[deprecated]\npub mod ${legacyModuleName};\n`,
    `#[deprecated]\n#[serde(rename = "legacy${mixed}Field")]\npub legacy_${["if", "ds"].join(
      "_",
    )}_field: Option<u8>,\n`,
    `#[serde(rename = "unowned${mixed}Field")]\npub unowned_${["if", "ds"].join(
      "_",
    )}_field: Option<u8>,\n`,
  ].join("\n");
  const declarations = compatibilityItemSpans("__selftest__.rs", source)
    .map((span) => span.declaration)
    .toSorted();
  assert.deepEqual(
    declarations,
    [
      `LEGACY_${upper}_WIRE`,
      `Legacy${mixed}AliasV0`,
      `Legacy${mixed}StructV0`,
      `Legacy${mixed}Variant`,
      `legacy_${["if", "ds"].join("_")}_rank`,
      `legacy_${["if", "ds"].join("_")}_field`,
      legacyModuleName,
    ].toSorted(),
    "deprecated compatibility syntax parser must cover structs, grouped aliases, variants, and private wire constants",
  );
  const scriptSource = `/** @deprecated owner and removal */\nconst LEGACY_${upper}_LABEL = "legacy-${[
    "if",
    "ds",
  ].join("")}";\n`;
  assert.deepEqual(
    compatibilityItemSpans("__selftest__.ts", scriptSource).map((span) => span.declaration),
    [`LEGACY_${upper}_LABEL`],
    "deprecated script constants must be compatibility spans",
  );
  const cargoSource = [
    "[features]",
    "# compatibility-owner: example maintainers",
    "# compatibility-removal: not before 1.0, after downstream migration and zero non-compat uses",
    `legacy-${["if", "ds"].join("")} = [`,
    '  "canonical-feature",',
    "]",
    `unowned-${["if", "ds"].join("")} = []`,
    "# compatibility-owner: example maintainers",
    "# compatibility-removal: after downstream migration only",
    `premature-${["if", "ds"].join("")} = []`,
  ].join("\n");
  assert.deepEqual(
    compatibilityItemSpans("rust/crates/example/Cargo.toml", cargoSource).map(
      (span) => span.declaration,
    ),
    [`legacy-${["if", "ds"].join("")}`],
    "Cargo feature compatibility requires adjacent owner and complete removal metadata",
  );
  const deprecatedAlias = `Legacy${mixed}AliasV0`;
  const arbitraryOldToken = `Arbitrary${mixed}TokenV0`;
  assert.deepEqual(
    generatedDeprecatedDeclarationCompatibilitySpans(
      "rust/crates/example/tests/snapshots/public-api.txt",
      `pub type ${deprecatedAlias} = CurrentV0;\npub type ${arbitraryOldToken} = CurrentV0;\n`,
      new Set([deprecatedAlias]),
    ).map((span) => span.declaration),
    [deprecatedAlias],
    "generated snapshot compatibility must resolve to an observed deprecated Rust declaration",
  );
  assert.deepEqual(
    generatedDeprecatedDeclarationCompatibilitySpans(
      "rust/omena-response-surface-split-census.json",
      `{"deprecated":"${deprecatedAlias}","newClaim":"${arbitraryOldToken}"}\n`,
      new Set([deprecatedAlias]),
    ).map((span) => span.declaration),
    [deprecatedAlias],
    "response-surface compatibility must not absorb a nondeprecated old-name token",
  );
  const semverSource = JSON.stringify({
    reason: `prose about ${deprecatedAlias} is still a claim`,
    expectedFailures: [
      {
        evidenceNeedles: [`type ${deprecatedAlias}`, `type ${arbitraryOldToken}`],
        expectedWitnesses: [deprecatedAlias],
      },
    ],
    expectedRuntimeValueChanges: [{ evidence: [{ needles: [`function ${deprecatedAlias}`] }] }],
  });
  assert.deepEqual(
    generatedDeprecatedDeclarationCompatibilitySpans(
      "rust/omena-rust-semver-intent.json",
      semverSource,
      new Set([deprecatedAlias]),
    ).map((span) => span.declaration),
    [deprecatedAlias, deprecatedAlias, deprecatedAlias],
    "semver intent compatibility must be exact-linked evidence, never prose or a nondeprecated old token",
  );
  const contextualSiteEvidence = JSON.stringify({
    evidenceNeedles: [`module omena_categorical::${legacyModuleName}::legacy`],
  });
  const contextualSiteSpans = generatedDeprecatedDeclarationCompatibilitySpans(
    "rust/omena-rust-semver-intent.json",
    contextualSiteEvidence,
    new Set(["site"]),
  );
  assert.ok(
    contextualSiteSpans.some(
      (span) =>
        span.declaration === legacyModuleName &&
        contextualSiteEvidence.slice(span.start, span.end) === `${legacyModuleName}::`,
    ),
    "semver evidence must link a contextual module seed to its exact deprecated module token",
  );
}

function validateSiteFamilySeedBoundary(): void {
  const pattern = familySeeds.find(({ familyId }) => familyId === "f05")?.pattern;
  assert.ok(pattern, "f05 syntactic seed must exist");
  const stem = ["si", "te"].join("");
  const titleStem = `${stem[0].toUpperCase()}${stem.slice(1)}`;
  const matches = (source: string): readonly string[] =>
    [...source.matchAll(freshPattern(pattern))].map((match) => match[0]);
  const oldSurfaces = [
    `pub mod ${stem};`,
    `pub struct Cascade${titleStem}V0;`,
    `pub fn cascade_${stem}_v0() {}`,
    `pub struct ${titleStem}AxiomCheckV0;`,
    `pub ${stem}_id: String,`,
    `The ${stem} gluing theorem is implemented.`,
    `rust/omena-categorical/verify-${stem}-stability`,
    `fixture.categorical.${stem}-stability.v0`,
  ];
  for (const source of oldSurfaces) {
    assert.ok(matches(source).length > 0, `f05 syntactic seed missed old surface: ${source}`);
  }
  const unrelatedNoise = [
    `call_${stem}_id`,
    `reference_${stem}_identity`,
    `https://example.com/${stem}`,
    `web${stem}`,
    `generic ${stem} content`,
    `call-${stem} product`,
    `unsupported-${stem} product`,
    `call-${stem}-stability`,
    `reference-${stem}-stability`,
    `web-${stem}-stability`,
  ];
  for (const source of unrelatedNoise) {
    assert.deepEqual(matches(source), [], `f05 syntactic seed matched unrelated noise: ${source}`);
  }
}

function publishedCargoPackages(files: readonly TextFile[]): ReadonlySet<string> {
  const packages = new Set<string>();
  for (const { sourcePath, source } of files) {
    if (!/^rust\/crates\/[^/]+\/Cargo\.toml$/u.test(sourcePath)) continue;
    const packageHeading = /^\[package\]\s*$/mu.exec(source);
    if (packageHeading === null) continue;
    const packageTail = source.slice(packageHeading.index + packageHeading[0].length);
    const nextHeading = /^\[/mu.exec(packageTail);
    const packageSection = packageTail.slice(0, nextHeading?.index ?? packageTail.length);
    const name = /^name\s*=\s*"([^"]+)"/mu.exec(packageSection)?.[1];
    const explicitlyPrivate = /^publish\s*=\s*(?:false\b|\[\s*\])/mu.test(packageSection);
    if (name !== undefined && !explicitlyPrivate) packages.add(name);
  }
  return packages;
}

function identityPatternsByFamily(
  publishedPackages: ReadonlySet<string>,
  escalatedCrates: ReadonlySet<string>,
): ReadonlyMap<FamilyId, RegExp> {
  // Publishing an old-family package cannot mint its own escape. Only the
  // intersection with the authored, user-owned escalation rows is identity.
  return new Map(
    familySeeds.map(({ familyId, pattern }) => {
      const tokens = [...publishedPackages]
        .filter((packageName) => escalatedCrates.has(packageName))
        .filter((packageName) => freshPattern(pattern).test(packageName))
        .flatMap((packageName) => [packageName, packageName.replaceAll("-", "_")])
        .filter((value, index, all) => all.indexOf(value) === index)
        .toSorted();
      const source = tokens.length === 0 ? "$a" : `(?:${tokens.map(escapeRegExp).join("|")})`;
      return [familyId, new RegExp(`(?<![A-Za-z0-9_-])${source}(?![A-Za-z0-9_-])`, "gu")];
    }),
  );
}

function validateIdentityPatternConstructionBoundary(): void {
  const approved = ["omena", "streaming", ["if", "ds"].join("")].join("-");
  const unapproved = ["omena", "experimental", ["if", "ds"].join("")].join("-");
  const patterns = identityPatternsByFamily(new Set([approved, unapproved]), new Set([approved]));
  const pattern = patterns.get("f03");
  const seed = familySeeds.find(({ familyId }) => familyId === "f03")?.pattern;
  assert.ok(pattern, "f03 exact-identity pattern must exist");
  assert.ok(seed, "f03 syntactic seed must exist");
  assert.ok(freshPattern(pattern).test(approved), "approved published identity must match");
  assert.ok(
    freshPattern(seed).test(unapproved),
    "a sixth old-family published package must remain in the syntactic claim domain",
  );
  assert.ok(
    !freshPattern(pattern).test(unapproved),
    "a sixth old-family published package must remain a claim without a user escalation row",
  );
}

function validateIdentityPatternBoundary(patterns: ReadonlyMap<FamilyId, RegExp>): void {
  const packageName = ["omena", "streaming", ["if", "ds"].join("")].join("-");
  const crateSpelling = packageName.replaceAll("-", "_");
  const pattern = patterns.get("f03");
  assert.ok(pattern, "f03 exact-identity pattern must exist");
  for (const accepted of [packageName, crateSpelling, `${packageName}.relocation-gate`]) {
    assert.ok(
      freshPattern(pattern).test(accepted),
      `exact published identity must match: ${accepted}`,
    );
  }
  for (const rejected of [
    ["omena", "checker", "streaming", ["if", "ds"].join("")].join("-"),
    ["omena", "experimental", ["if", "ds"].join("")].join("-"),
    ["streaming", ["if", "ds"].join("")].join("-"),
    `ordinary prose about ${["streaming", ["if", "ds"].join("")].join(" ")}`,
  ]) {
    assert.ok(
      !freshPattern(pattern).test(rejected),
      `non-identity text must not match: ${rejected}`,
    );
  }
}

function sourceLineStarts(source: string): number[] {
  const starts = [0];
  for (let index = 0; index < source.length; index += 1) {
    if (source[index] === "\n") starts.push(index + 1);
  }
  return starts;
}

function lineIndexAtOffset(starts: readonly number[], offset: number): number {
  let low = 0;
  let high = starts.length;
  while (low + 1 < high) {
    const middle = Math.floor((low + high) / 2);
    if (starts[middle] <= offset) low = middle;
    else high = middle;
  }
  return low;
}

function expectedClaimKey(row: ExpectedClaimRow): string {
  return `${row.familyId}:${row.sourcePath}:${row.line}:${row.lineSha256}:${row.occurrenceInLine}`;
}

function occurrenceClaimKey(row: Occurrence): string {
  return `${row.familyId}:${row.sourcePath}:${row.line}:${row.lineSha256}:${row.occurrenceInLine}`;
}

function compatibilityKeys(row: CompatibilityDeclarationRow): string[] {
  return row.declarations.map((declaration) => `${row.familyId}:${row.sourcePath}:${declaration}`);
}

function duplicateErrors(values: readonly string[], label: string): string[] {
  const seen = new Set<string>();
  const duplicates = new Set<string>();
  for (const value of values) {
    if (seen.has(value)) duplicates.add(value);
    seen.add(value);
  }
  return [...duplicates].toSorted().map((value) => `duplicate ${label}: ${value}`);
}

function renderSnapshot(snapshot: CensusSnapshot): string {
  const families = snapshot.families
    .map(
      (row) =>
        `${row.familyId}=${row.occurrenceCount}` +
        `(claim:${row.claimCount},compat:${row.compatCount},identity:${row.identityEscalationCount},governance:${row.governanceDataCount})`,
    )
    .join(" ");
  return (
    `ref=${snapshot.sourceRef ?? "WORKTREE"} ` +
    `domainTracked=${snapshot.domain.trackedFileCount} ` +
    `domainTextual=${snapshot.domain.textualFileCount} ` +
    `nulExcluded=${snapshot.domain.nulExcludedCount} ` +
    `bootstrapUntracked=${snapshot.domain.bootstrapUntrackedFileCount} ` +
    `deletedTracked=${snapshot.domain.deletedTrackedFileCount} ` +
    `fileListSha256=${snapshot.domain.fileListSha256} ` +
    `occurrences=${snapshot.occurrenceCount} ${families}`
  );
}

function renderClaimFiles(result: ScanResult): string {
  return familyIds
    .map((familyId) => {
      const files = new Map<string, number>();
      for (const row of result.occurrences) {
        if (row.familyId !== familyId || row.disposition !== "claim") continue;
        files.set(row.sourcePath, (files.get(row.sourcePath) ?? 0) + 1);
      }
      const rows = [...files.entries()].toSorted(
        ([leftPath, leftCount], [rightPath, rightCount]) =>
          rightCount - leftCount || leftPath.localeCompare(rightPath),
      );
      return `${familyId} claimFiles=${rows.length}\n${rows
        .map(([sourcePath, count]) => `  ${count}\t${sourcePath}`)
        .join("\n")}`;
    })
    .join("\n");
}

function renderClaimSites(result: ScanResult): string {
  const rows = result.occurrences.filter((row) => row.disposition === "claim");
  if (rows.length === 0) return "claimSites=0";
  return `claimSites=${rows.length}\n${rows
    .map(
      (row) =>
        `  ${row.familyId}\t${row.sourcePath}:${row.line}:${row.column}` +
        `\tmatchedText=${JSON.stringify(row.matchedText)}` +
        `\tlineSha256=${row.lineSha256}\toccurrenceInLine=${row.occurrenceInLine}`,
    )
    .join("\n")}`;
}

function renderCompatDeclarations(result: ScanResult): string {
  const rows = result.occurrences
    .filter((row) => row.disposition === "compat")
    .map((row) => `${row.familyId}:${row.sourcePath}:${row.declaration}`)
    .filter((row, index, all) => all.indexOf(row) === index)
    .toSorted();
  return `compatDeclarations=${rows.length}\n${rows.map((row) => `  ${row}`).join("\n")}`;
}

function validateFrozenEntrySnapshot(snapshot: CensusSnapshot): void {
  assert.equal(snapshot.sourceRef, entrySourceRef, "entry census must name the exact S4 pin");
  assert.equal(
    sha256(JSON.stringify(snapshot)),
    entrySnapshotSha256,
    "entry census baseline changed; remeasure the S4 pin and review its frozen digest",
  );
}

function valueAfter(flag: string): string | undefined {
  const index = process.argv.indexOf(flag);
  if (index === -1) return undefined;
  const value = process.argv[index + 1];
  assert.ok(value && !value.startsWith("--"), `${flag} requires a value`);
  return value;
}

function resolveCommit(sourceRef: string): string {
  return git(["rev-parse", `${sourceRef}^{commit}`]).trim();
}

function readJson<T>(sourcePath: string): T {
  return JSON.parse(fs.readFileSync(path.join(repoRoot, sourcePath), "utf8")) as T;
}

function freshPattern(pattern: RegExp): RegExp {
  return new RegExp(pattern.source, pattern.flags);
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
}

function sha256(value: string | Buffer): string {
  return createHash("sha256").update(value).digest("hex");
}

function git(arguments_: readonly string[]): string {
  return execFileSync("git", arguments_, {
    cwd: repoRoot,
    encoding: "utf8",
    maxBuffer: 128 * 1024 * 1024,
  });
}

function gitBuffer(arguments_: readonly string[]): Buffer {
  return execFileSync("git", arguments_, {
    cwd: repoRoot,
    encoding: "buffer",
    maxBuffer: 128 * 1024 * 1024,
  });
}
