import { strict as assert } from "node:assert";
import { execFileSync, spawnSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

type DefectClass = "shaking" | "liveness" | "placement" | "accounting" | "structuralEntailment";

interface InstrumentDomainV0 {
  readonly schemaVersion: "0";
  readonly product: "omena.linked-emission-instrument-domain";
  readonly files: readonly string[];
  readonly minimumFileCount: number;
  readonly excludedAssertionForms: readonly ExcludedAssertionFormV0[];
  readonly widenWhen: string;
}

interface ExcludedAssertionFormV0 {
  readonly file: string;
  readonly syntax: string;
  readonly expectedCount: number;
  readonly owner: string;
  readonly reentry: string;
}

interface InstrumentSiteV0 {
  readonly id: string;
  readonly file: string;
  readonly symbol: string;
  readonly lineHint: number;
  readonly syntax: string;
  readonly defectClass: DefectClass;
  readonly falsifiers: readonly string[];
  readonly producerReachability: "can-fail" | "entailed";
  readonly entryState: string;
  readonly owner: string;
  readonly reentry?: string;
}

interface InstrumentMapV0 {
  readonly schemaVersion: "0";
  readonly product: "omena.linked-emission-instrument-map";
  readonly domainFiles: readonly string[];
  readonly domainWideningCondition: string;
  readonly assertionSiteCount: number;
  readonly excludedAssertionCount: number;
  readonly structuralEntailmentCount: number;
  readonly limitations: readonly string[];
  readonly sites: readonly InstrumentSiteV0[];
}

interface FalsifierFiringRecordV0 {
  readonly perturbation: string;
  readonly observedRedCommand: string;
  readonly firedSiteIds: readonly string[];
}

interface StructuralEntailmentRecordV0 {
  readonly siteId: string;
  readonly owner: string;
  readonly reentry: string;
}

interface DeadTestAuditRecordV0 {
  readonly test: string;
  readonly namedFix: string;
  readonly entryColor: "red" | "green";
  readonly currentColor: "red" | "green";
  readonly disposition: "repaired" | "owned";
  readonly evidence: string;
  readonly owner: string;
  readonly reentry: string;
}

interface FalsifierEvidenceV0 {
  readonly schemaVersion: "0";
  readonly product: "omena.linked-emission-falsifier-evidence";
  readonly limitations: readonly string[];
  readonly firedSitesByPerturbation: readonly FalsifierFiringRecordV0[];
  readonly structuralEntailments: readonly StructuralEntailmentRecordV0[];
  readonly deadTestAudit: readonly DeadTestAuditRecordV0[];
}

interface UnobservedExecutableSiteSummaryV0 {
  readonly shadowedSiteIds: readonly string[];
  readonly reachedGreenSiteIds: readonly string[];
  readonly notExecutedSiteIds: readonly string[];
  readonly committedArtifactSiteIds: readonly string[];
}

const scriptPath = fileURLToPath(import.meta.url);
const repositoryRoot = resolve(dirname(scriptPath), "..");
const domainPath = resolve(repositoryRoot, "rust/omena-linked-emission-instrument-domain.json");
const mapPath = resolve(repositoryRoot, "rust/omena-linked-emission-instrument-map.json");
const evidencePath = resolve(repositoryRoot, "rust/omena-linked-emission-falsifier-evidence.json");
const ansiEscapeSequencePattern = new RegExp(
  `${String.fromCodePoint(27)}\\[[0-?]*[ -/]*[@-~]`,
  "gu",
);
const assertionPatterns = [
  /assert\.deepEqual/,
  /assert\.strictEqual/,
  /assert\.notEqual/,
  /assert\.equal/,
  /assert\.match/,
  /assert\.throws/,
  /assert\.ok/,
  /assert_ne!/,
  /assert_eq!/,
  /assert!/,
  /return Err\(/,
] as const;
const notePattern = /^\s*\/\/ FALSIFIER: (.+)$/;
const allowedClasses = new Set<DefectClass>([
  "shaking",
  "liveness",
  "placement",
  "accounting",
  "structuralEntailment",
]);

function parseFields(note: string): Readonly<Record<string, string>> {
  return Object.fromEntries(
    note.split(/\s+/).map((field) => {
      const separator = field.indexOf("=");
      assert.notEqual(separator, -1, `malformed FALSIFIER field: ${field}`);
      return [field.slice(0, separator), field.slice(separator + 1)];
    }),
  );
}

function nearestSymbol(lines: readonly string[], lineIndex: number): string {
  for (let index = lineIndex; index >= 0; index -= 1) {
    const line = lines[index] ?? "";
    const rustFunction = line.match(/\bfn\s+([A-Za-z0-9_]+)/);
    const tsFunction = line.match(/\bfunction\s+([A-Za-z0-9_]+)/);
    const functionName = rustFunction?.[1] ?? tsFunction?.[1];
    if (!functionName) continue;

    const functionToSite = lines.slice(index, lineIndex + 1).join("\n");
    const scopeDepth =
      [...functionToSite].filter((character) => character === "{").length -
      [...functionToSite].filter((character) => character === "}").length;
    if (scopeDepth > 0) return functionName;
  }
  return "<module>";
}

function assertionSyntax(line: string): string | undefined {
  return assertionPatterns.map((pattern) => line.match(pattern)?.[0]).find(Boolean);
}

function trackedRustTestNames(): ReadonlySet<string> {
  const files = execFileSync("git", ["ls-files", "rust/crates"], {
    cwd: repositoryRoot,
    encoding: "utf8",
  })
    .split(/\r?\n/u)
    .filter((file) => file.endsWith(".rs"));
  const names = new Set<string>();

  for (const file of files) {
    const source = readFileSync(resolve(repositoryRoot, file), "utf8");
    for (const match of source.matchAll(/#\[test\]\s*fn\s+([A-Za-z_][A-Za-z0-9_]*)\b/gu)) {
      const name = match[1];
      if (name) names.add(name);
    }
  }

  return names;
}

function enumerateSites(file: string): {
  readonly sites: readonly InstrumentSiteV0[];
  readonly excludedCounts: ReadonlyMap<string, number>;
} {
  const source = readFileSync(resolve(repositoryRoot, file), "utf8");
  const lines = source.split(/\r?\n/);
  const sites: InstrumentSiteV0[] = [];
  const excludedCounts = new Map<string, number>();
  const orphanNotes = new Set<number>();

  lines.forEach((line, index) => {
    if (notePattern.test(line)) {
      orphanNotes.add(index);
    }
    const syntax = assertionSyntax(line);
    if (!syntax) {
      return;
    }

    const noteLineIndex = index - 1;
    const note = lines[noteLineIndex]?.match(notePattern)?.[1];
    if (!note) {
      excludedCounts.set(syntax, (excludedCounts.get(syntax) ?? 0) + 1);
      return;
    }
    orphanNotes.delete(noteLineIndex);
    const fields = parseFields(note);
    const defectClass = fields.class as DefectClass;
    assert.ok(
      allowedClasses.has(defectClass),
      `invalid defect class at ${file}:${index + 1}: ${fields.class ?? "<missing>"}`,
    );
    assert.ok(fields.id, `FALSIFIER id is missing at ${file}:${index + 1}`);
    assert.ok(fields.via, `FALSIFIER target is missing at ${file}:${index + 1}`);
    assert.ok(fields.owner, `FALSIFIER owner is missing at ${file}:${index + 1}`);
    assert.ok(fields.entry, `FALSIFIER entry state is missing at ${file}:${index + 1}`);

    const falsifiers = fields.via?.split(",").filter(Boolean) ?? [];
    const structural = defectClass === "structuralEntailment";
    assert.equal(
      falsifiers.length === 1 && falsifiers[0] === "STRUCTURAL",
      structural,
      `STRUCTURAL target/class mismatch at ${file}:${index + 1}`,
    );
    assert.equal(
      fields.producer,
      structural ? "entailed" : "can-fail",
      `producer reachability mismatch at ${file}:${index + 1}`,
    );
    if (structural) {
      assert.ok(fields.reentry, `STRUCTURAL site has no re-entry at ${file}:${index + 1}`);
    }

    sites.push({
      id: fields.id,
      file,
      symbol: nearestSymbol(lines, index),
      lineHint: index + 1,
      syntax,
      defectClass,
      falsifiers,
      producerReachability: structural ? "entailed" : "can-fail",
      entryState: fields.entry,
      owner: fields.owner,
      ...(fields.reentry ? { reentry: fields.reentry } : {}),
    });
  });

  assert.deepEqual(
    [...orphanNotes].map((index) => `${file}:${index + 1}`),
    [],
    `FALSIFIER notes without syntactic assertion sites in ${file}`,
  );
  return { sites, excludedCounts };
}

function knownFalsifierSource(): string {
  return [
    "rust/crates/omena-diff-test/src/linked_emission.rs",
    "rust/crates/omena-diff-test/src/bin/omena-linked-emission-byte-differential.rs",
    "scripts/check-rust-omena-bundler-linked-emission-byte-differential.ts",
    "scripts/check-rust-omena-query-linked-source-map-fallback.ts",
  ]
    .map((file) => readFileSync(resolve(repositoryRoot, file), "utf8"))
    .join("\n");
}

function buildMap(domain: InstrumentDomainV0): InstrumentMapV0 {
  const enumerated = domain.files.map((file) => ({ file, ...enumerateSites(file) }));
  const sites = enumerated.flatMap((entry) => entry.sites);
  const ids = sites.map((site) => site.id);
  assert.equal(new Set(ids).size, ids.length, "FALSIFIER site ids must be unique");

  const actualExclusions = enumerated
    .flatMap(({ file, excludedCounts }) =>
      [...excludedCounts].map(([syntax, expectedCount]) => ({ file, syntax, expectedCount })),
    )
    .sort((left, right) =>
      `${left.file}\0${left.syntax}`.localeCompare(`${right.file}\0${right.syntax}`),
    );
  if (process.argv.includes("--inject-unannotated-assertion")) {
    const first = actualExclusions[0];
    assert.ok(first, "assertion exclusion injection needs a non-empty domain");
    first.expectedCount += 1;
  }
  const expectedExclusions = domain.excludedAssertionForms
    .map(({ file, syntax, expectedCount }) => ({ file, syntax, expectedCount }))
    .sort((left, right) =>
      `${left.file}\0${left.syntax}`.localeCompare(`${right.file}\0${right.syntax}`),
    );
  assert.deepEqual(
    actualExclusions,
    expectedExclusions,
    "unannotated assertion census drifted; annotate the site or update the owned exclusion",
  );
  for (const exclusion of domain.excludedAssertionForms) {
    assert.ok(exclusion.owner, `assertion exclusion has no owner: ${exclusion.file}`);
    assert.ok(exclusion.reentry, `assertion exclusion has no re-entry: ${exclusion.file}`);
    assert.ok(domain.files.includes(exclusion.file), `assertion exclusion is outside the domain`);
  }

  const knownSource = knownFalsifierSource()
    .split(/\r?\n/)
    .filter((line) => !notePattern.test(line))
    .join("\n");
  for (const site of sites) {
    for (const falsifier of site.falsifiers) {
      if (falsifier === "STRUCTURAL") {
        continue;
      }
      const escaped = falsifier.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
      assert.ok(
        new RegExp(`(?<![A-Za-z0-9_-])${escaped}(?![A-Za-z0-9_-])`, "u").test(knownSource),
        `FALSIFIER target does not resolve: ${site.id} -> ${falsifier}`,
      );
    }
  }

  return {
    schemaVersion: "0",
    product: "omena.linked-emission-instrument-map",
    domainFiles: domain.files,
    domainWideningCondition: domain.widenWhen,
    assertionSiteCount: sites.length,
    excludedAssertionCount: actualExclusions.reduce(
      (count, exclusion) => count + exclusion.expectedCount,
      0,
    ),
    structuralEntailmentCount: sites.filter((site) => site.defectClass === "structuralEntailment")
      .length,
    limitations: [
      "Firing observations are attached after executable perturbations run; all other rows remain disclosures.",
      "Unannotated assertion forms are accepted only through exact, owner-bound per-file counts.",
      "The mechanism cannot prove that a selected defect class is semantically correct.",
    ],
    sites,
  };
}

function generatedFiringRecords(
  instrumentMap: InstrumentMapV0,
): readonly FalsifierFiringRecordV0[] {
  const directFalsifiers = [
    ...new Set(
      instrumentMap.sites
        .flatMap((site) => site.falsifiers)
        .filter((falsifier) => falsifier.startsWith("--inject-")),
    ),
  ].sort();

  return directFalsifiers.map((perturbation) => {
    const candidateFiles = instrumentMap.domainFiles.filter((file) => {
      if (!file.endsWith(".ts")) return false;
      const sourceWithoutDisclosures = readFileSync(resolve(repositoryRoot, file), "utf8")
        .split(/\r?\n/u)
        .filter((line) => !notePattern.test(line))
        .join("\n");
      return sourceWithoutDisclosures.includes(perturbation);
    });
    assert.equal(
      candidateFiles.length,
      1,
      `direct falsifier must resolve to one executable gate: ${perturbation}`,
    );
    const file = candidateFiles[0];
    assert.ok(file);
    const commandArguments = ["--import", "tsx", `./${file}`, perturbation];
    const run = spawnSync(process.execPath, commandArguments, {
      cwd: repositoryRoot,
      encoding: "utf8",
    });
    assert.notEqual(run.status, 0, `direct falsifier stayed green: ${perturbation}`);
    const output = [run.stdout, run.stderr]
      .filter(Boolean)
      .join("\n")
      .replace(ansiEscapeSequencePattern, "");
    const escapedRoot = repositoryRoot.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
    const firstRepositoryFrame = new RegExp(`${escapedRoot}/([^:\\n]+):(\\d+):(\\d+)`, "u").exec(
      output,
    );
    assert.ok(firstRepositoryFrame, `direct falsifier emitted no repository stack frame`);
    const [, failureFile, failureLine] = firstRepositoryFrame;
    const line = Number(failureLine);
    const firedSite = instrumentMap.sites.find(
      (site) =>
        site.file === failureFile &&
        site.lineHint === line &&
        site.falsifiers.includes(perturbation),
    );
    assert.ok(
      firedSite,
      `first RED is not an instrumented site: ${perturbation} -> ${failureFile}:${line}`,
    );
    return {
      perturbation,
      observedRedCommand: `node ${commandArguments.join(" ")}`,
      firedSiteIds: [firedSite.id],
    };
  });
}

function summarizeUnobservedExecutableSites(
  instrumentMap: InstrumentMapV0,
  firingRecords: readonly FalsifierFiringRecordV0[],
): UnobservedExecutableSiteSummaryV0 {
  const recordsByPerturbation = new Map(
    firingRecords.map((record) => [record.perturbation, record] as const),
  );
  const sitesById = new Map(instrumentMap.sites.map((site) => [site.id, site] as const));
  const firedSiteIds = new Set(firingRecords.flatMap((record) => record.firedSiteIds));
  const shadowedSiteIds: string[] = [];
  const reachedGreenSiteIds: string[] = [];
  const notExecutedSiteIds: string[] = [];

  for (const site of instrumentMap.sites) {
    if (site.producerReachability !== "can-fail" || firedSiteIds.has(site.id)) continue;

    const executableRecords = site.falsifiers
      .map((falsifier) => recordsByPerturbation.get(falsifier))
      .filter((record): record is FalsifierFiringRecordV0 => record !== undefined);
    if (executableRecords.length === 0) continue;
    const classifications = executableRecords.map((record) => {
      assert.equal(
        record.firedSiteIds.length,
        1,
        `first-failure record must name exactly one site: ${record.perturbation}`,
      );
      const firstFailure = sitesById.get(record.firedSiteIds[0] ?? "");
      assert.ok(firstFailure, `first-failure record names an unknown site: ${record.perturbation}`);

      if (site.file !== firstFailure.file) return "not-executed";
      assert.notEqual(
        site.lineHint,
        firstFailure.lineHint,
        `unobserved site overlaps the first-failure location: ${site.id}`,
      );
      return site.lineHint < firstFailure.lineHint ? "reached-green" : "shadowed";
    });
    assert.equal(
      new Set(classifications).size,
      1,
      `executable perturbations disagree on the unobserved-site classification: ${site.id}`,
    );

    const classification = classifications[0];
    if (classification === "not-executed") notExecutedSiteIds.push(site.id);
    if (classification === "reached-green") reachedGreenSiteIds.push(site.id);
    if (classification === "shadowed") shadowedSiteIds.push(site.id);
  }

  const committedArtifactSiteIds = [
    "linked-emission-gate-001",
    "linked-emission-baseline-domain-floor",
    "linked-emission-gate-002",
    "linked-emission-gate-003",
    "linked-emission-gate-018",
    "linked-emission-gate-019",
  ];
  const reachedGreenSet = new Set(reachedGreenSiteIds);
  assert.deepEqual(
    committedArtifactSiteIds.filter((siteId) => reachedGreenSet.has(siteId)),
    committedArtifactSiteIds,
    "committed-artifact assertions must remain explicit reached-and-green witnesses",
  );

  return {
    shadowedSiteIds,
    reachedGreenSiteIds,
    notExecutedSiteIds,
    committedArtifactSiteIds,
  };
}

function validateFiringEvidence(
  instrumentMap: InstrumentMapV0,
  evidence: FalsifierEvidenceV0,
): void {
  assert.equal(evidence.schemaVersion, "0");
  assert.equal(evidence.product, "omena.linked-emission-falsifier-evidence");
  assert.ok(
    evidence.limitations.some((limitation) => limitation.includes("classification")),
    "falsifier evidence must disclose that it cannot validate classification semantics",
  );

  const sitesById = new Map(instrumentMap.sites.map((site) => [site.id, site]));
  const firingRecords = new Map<string, FalsifierFiringRecordV0>();
  for (const record of evidence.firedSitesByPerturbation) {
    assert.ok(record.perturbation, "firing record has no perturbation");
    assert.ok(record.observedRedCommand, `${record.perturbation} has no observed RED command`);
    assert.ok(record.firedSiteIds.length > 0, `${record.perturbation} fired no assertion sites`);
    assert.ok(
      !firingRecords.has(record.perturbation),
      `duplicate firing record: ${record.perturbation}`,
    );
    firingRecords.set(record.perturbation, record);
    for (const siteId of record.firedSiteIds) {
      const site = sitesById.get(siteId);
      assert.ok(site, `firing record names an unknown assertion site: ${siteId}`);
      assert.ok(
        site.falsifiers.includes(record.perturbation),
        `firing record assigns ${siteId} to an undisclosed perturbation`,
      );
    }
  }

  const structuralRecords = new Map<string, StructuralEntailmentRecordV0>();
  for (const record of evidence.structuralEntailments) {
    assert.ok(
      !structuralRecords.has(record.siteId),
      `duplicate STRUCTURAL entitlement record: ${record.siteId}`,
    );
    assert.ok(record.owner, `STRUCTURAL entitlement has no owner: ${record.siteId}`);
    assert.ok(record.reentry, `STRUCTURAL entitlement has no re-entry: ${record.siteId}`);
    structuralRecords.set(record.siteId, record);
  }

  for (const site of instrumentMap.sites) {
    if (site.falsifiers.includes("STRUCTURAL")) {
      const structural = structuralRecords.get(site.id);
      assert.ok(structural, `STRUCTURAL site has no owned census row: ${site.id}`);
      assert.equal(structural.owner, site.owner, `STRUCTURAL owner drift: ${site.id}`);
      assert.equal(structural.reentry, site.reentry, `STRUCTURAL re-entry drift: ${site.id}`);
      continue;
    }
  }

  assert.deepEqual(
    [...structuralRecords.keys()].sort(),
    instrumentMap.sites
      .filter((site) => site.falsifiers.includes("STRUCTURAL"))
      .map((site) => site.id)
      .sort(),
    "STRUCTURAL entitlement census is stale",
  );

  const expectedAuditTests = [
    "linked_emission_preserves_composes_target_from_dependency",
    "linked_emission_routes_reachability_by_owning_module",
    "missing_target_source_precedes_attribution_domain_validation",
    "production_attribution_domains_assign_every_admitted_name",
    "token_collision_path_gate_rejects_path_specific_selector_loss",
  ];
  assert.deepEqual(
    evidence.deadTestAudit.map((record) => record.test).sort(),
    expectedAuditTests,
    "dead-test audit does not cover the declared regression lineage",
  );
  const currentRustTests = trackedRustTestNames();
  for (const [auditIndex, record] of evidence.deadTestAudit.entries()) {
    assert.ok(record.namedFix, `dead-test audit row has no named fix: ${record.test}`);
    assert.ok(record.evidence, `dead-test audit row has no evidence: ${record.test}`);
    assert.ok(record.owner, `dead-test audit row has no owner: ${record.test}`);
    assert.ok(record.reentry, `dead-test audit row has no re-entry: ${record.test}`);
    const lookupTest =
      process.argv.includes("--inject-dead-test-audit-drift") && auditIndex === 0
        ? "injected_missing_regression_test"
        : record.test;
    assert.ok(
      currentRustTests.has(lookupTest),
      `dead-test audit names no current Rust test: ${record.test}`,
    );
    assert.equal(
      record.currentColor === "green",
      record.disposition === "owned",
      `a still-green dead-test row must remain explicitly owned: ${record.test}`,
    );
  }
}

const domain = JSON.parse(readFileSync(domainPath, "utf8")) as InstrumentDomainV0;
assert.equal(domain.schemaVersion, "0");
assert.equal(domain.product, "omena.linked-emission-instrument-domain");
assert.ok(domain.files.length > 0, "instrument domain must contain at least one file");
assert.ok(
  domain.files.length >= domain.minimumFileCount,
  `instrument domain shrank below its committed floor ${domain.minimumFileCount}`,
);
assert.equal(
  new Set(domain.files).size,
  domain.files.length,
  "instrument domain files must be unique",
);
const instrumentMapWithoutFiringCounts = buildMap(domain);
const falsifierEvidence = JSON.parse(readFileSync(evidencePath, "utf8")) as FalsifierEvidenceV0;
const observedFiringRecords = generatedFiringRecords(instrumentMapWithoutFiringCounts);
const observedFiringSiteCount = new Set(
  observedFiringRecords.flatMap((record) => record.firedSiteIds),
).size;
const unobservedExecutableSites = summarizeUnobservedExecutableSites(
  instrumentMapWithoutFiringCounts,
  observedFiringRecords,
);
const nonStructuralSiteCount =
  instrumentMapWithoutFiringCounts.assertionSiteCount -
  instrumentMapWithoutFiringCounts.structuralEntailmentCount;
const instrumentMap: InstrumentMapV0 = {
  ...instrumentMapWithoutFiringCounts,
  limitations: [
    `Firing is observed only as the first-failure site of each executable perturbation (currently ${observedFiringSiteCount} of ${nonStructuralSiteCount} non-structural sites); the remaining rows are disclosures whose producerReachability: can-fail is not verified by this mechanism.`,
    ...instrumentMapWithoutFiringCounts.limitations.slice(1),
  ],
};
const writeMode = process.argv.includes("--write");
const unobservedExecutableLimitation = `Among the ${
  unobservedExecutableSites.shadowedSiteIds.length +
  unobservedExecutableSites.reachedGreenSiteIds.length +
  unobservedExecutableSites.notExecutedSiteIds.length
} executable-but-unobserved sites, ${unobservedExecutableSites.shadowedSiteIds.length} are shadowed by an earlier first failure in the same command, ${unobservedExecutableSites.reachedGreenSiteIds.length} are reached and remain green, and ${unobservedExecutableSites.notExecutedSiteIds.length} are not executed by that command. At least ${unobservedExecutableSites.committedArtifactSiteIds.length} reached-and-green sites assert committed artifacts that their disclosed perturbation cannot change; no site in these three buckets is claimed to have fired.`;
const generatedEvidence: FalsifierEvidenceV0 = {
  ...falsifierEvidence,
  limitations: [
    falsifierEvidence.limitations[0] ??
      "First-failure records are recomputed from direct perturbation runs; semantic defect classification remains authored.",
    falsifierEvidence.limitations[1] ??
      "Non-executable semantic perturbation names are disclosures only and are not represented as observed firing records.",
    unobservedExecutableLimitation,
  ],
  firedSitesByPerturbation: observedFiringRecords,
};
validateFiringEvidence(instrumentMap, writeMode ? generatedEvidence : falsifierEvidence);
const committedFiringRecords = process.argv.includes("--inject-firing-evidence-drift")
  ? falsifierEvidence.firedSitesByPerturbation.map((record, index) =>
      index === 0 ? { ...record, firedSiteIds: ["injected-unknown-site"] } : record,
    )
  : falsifierEvidence.firedSitesByPerturbation;
if (!writeMode) {
  assert.deepEqual(
    falsifierEvidence.limitations,
    generatedEvidence.limitations,
    "committed falsifier limitations do not match measured first-failure coverage",
  );
  assert.deepEqual(
    committedFiringRecords,
    observedFiringRecords,
    "committed firing evidence does not match the generated first-failure observations",
  );
}
const serializedMap = `${JSON.stringify(instrumentMap, null, 2)}\n`;
const serializedEvidence = `${JSON.stringify(generatedEvidence, null, 2)}\n`;

if (writeMode) {
  writeFileSync(mapPath, serializedMap);
  writeFileSync(evidencePath, serializedEvidence);
  execFileSync(
    "pnpm",
    ["exec", "oxfmt", relative(repositoryRoot, mapPath), relative(repositoryRoot, evidencePath)],
    {
      cwd: repositoryRoot,
      stdio: "inherit",
    },
  );
} else {
  const committed = JSON.parse(readFileSync(mapPath, "utf8")) as InstrumentMapV0;
  assert.deepEqual(committed, instrumentMap, "linked-emission instrument map is stale");
}

console.log(
  JSON.stringify({
    assertionSiteCount: instrumentMap.assertionSiteCount,
    excludedAssertionCount: instrumentMap.excludedAssertionCount,
    structuralEntailmentCount: instrumentMap.structuralEntailmentCount,
    domainFileCount: instrumentMap.domainFiles.length,
    firingRecordCount: generatedEvidence.firedSitesByPerturbation.length,
    shadowedExecutableSiteCount: unobservedExecutableSites.shadowedSiteIds.length,
    reachedGreenExecutableSiteCount: unobservedExecutableSites.reachedGreenSiteIds.length,
    notExecutedExecutableSiteCount: unobservedExecutableSites.notExecutedSiteIds.length,
    committedArtifactReachedGreenSiteCount:
      unobservedExecutableSites.committedArtifactSiteIds.length,
    deadTestAuditCount: falsifierEvidence.deadTestAudit.length,
  }),
);
