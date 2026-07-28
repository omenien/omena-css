import { strict as assert } from "node:assert";
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

type DefectClass = "shaking" | "liveness" | "placement" | "accounting" | "structuralEntailment";

interface InstrumentDomainV0 {
  readonly schemaVersion: "0";
  readonly product: "omena.linked-emission-instrument-domain";
  readonly files: readonly string[];
  readonly widenWhen: string;
}

interface InstrumentSiteV0 {
  readonly id: string;
  readonly file: string;
  readonly symbol: string;
  readonly lineHint: number;
  readonly syntax: string;
  readonly defectClass: DefectClass;
  readonly falsifier: string;
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

const scriptPath = fileURLToPath(import.meta.url);
const repositoryRoot = resolve(dirname(scriptPath), "..");
const domainPath = resolve(repositoryRoot, "rust/omena-linked-emission-instrument-domain.json");
const mapPath = resolve(repositoryRoot, "rust/omena-linked-emission-instrument-map.json");
const evidencePath = resolve(repositoryRoot, "rust/omena-linked-emission-falsifier-evidence.json");
const assertionPattern = /assert!|assert_eq!|assert\.ok|assert\.deepEqual|return Err\(/;
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
    if (rustFunction?.[1]) {
      return rustFunction[1];
    }
    const tsFunction = line.match(/\bfunction\s+([A-Za-z0-9_]+)/);
    if (tsFunction?.[1]) {
      return tsFunction[1];
    }
  }
  return "<module>";
}

function enumerateSites(file: string): readonly InstrumentSiteV0[] {
  const source = readFileSync(resolve(repositoryRoot, file), "utf8");
  const lines = source.split(/\r?\n/);
  const sites: InstrumentSiteV0[] = [];
  const orphanNotes = new Set<number>();

  lines.forEach((line, index) => {
    if (notePattern.test(line)) {
      orphanNotes.add(index);
    }
    const syntax = line.match(assertionPattern)?.[0];
    if (!syntax) {
      return;
    }

    const noteLineIndex = index - 1;
    const note = lines[noteLineIndex]?.match(notePattern)?.[1];
    assert.ok(note, `assertion site has no FALSIFIER note: ${file}:${index + 1}`);
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

    const structural = defectClass === "structuralEntailment";
    assert.equal(
      fields.via === "STRUCTURAL",
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
      falsifier: fields.via,
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
  return sites;
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
  const sites = domain.files.flatMap((file) => enumerateSites(file));
  const ids = sites.map((site) => site.id);
  assert.equal(new Set(ids).size, ids.length, "FALSIFIER site ids must be unique");

  const knownSource = knownFalsifierSource()
    .split(/\r?\n/)
    .filter((line) => !notePattern.test(line))
    .join("\n");
  for (const site of sites) {
    if (site.falsifier === "STRUCTURAL") {
      continue;
    }
    assert.ok(
      knownSource.includes(site.falsifier),
      `FALSIFIER target does not resolve: ${site.id} -> ${site.falsifier}`,
    );
  }

  return {
    schemaVersion: "0",
    product: "omena.linked-emission-instrument-map",
    domainFiles: domain.files,
    domainWideningCondition: domain.widenWhen,
    assertionSiteCount: sites.length,
    structuralEntailmentCount: sites.filter((site) => site.defectClass === "structuralEntailment")
      .length,
    limitations: [
      "This map verifies disclosure coverage and identifier resolution; firing is enforced by the registered falsifier-disclosure closure gate.",
      "The mechanism cannot prove that a selected defect class is semantically correct.",
    ],
    sites,
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
  const firedSiteOwner = new Map<string, string>();
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
        !firedSiteOwner.has(siteId),
        `assertion site appears in multiple firing records: ${siteId}`,
      );
      firedSiteOwner.set(siteId, record.perturbation);
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
    if (site.falsifier === "STRUCTURAL") {
      const structural = structuralRecords.get(site.id);
      assert.ok(structural, `STRUCTURAL site has no owned census row: ${site.id}`);
      assert.equal(structural.owner, site.owner, `STRUCTURAL owner drift: ${site.id}`);
      assert.equal(structural.reentry, site.reentry, `STRUCTURAL re-entry drift: ${site.id}`);
      continue;
    }
    const record = firingRecords.get(site.falsifier);
    assert.ok(record, `FALSIFIER target has no firing record: ${site.id} -> ${site.falsifier}`);
    assert.ok(
      record.firedSiteIds.includes(site.id),
      `FALSIFIER did not fire the disclosed assertion site: ${site.id} -> ${site.falsifier}`,
    );
  }

  for (const [siteId, perturbation] of firedSiteOwner) {
    const site = sitesById.get(siteId);
    assert.ok(site);
    assert.equal(
      site.falsifier,
      perturbation,
      `firing record assigns ${siteId} to ${perturbation}, but its disclosure names ${site.falsifier}`,
    );
  }

  assert.deepEqual(
    [...structuralRecords.keys()].sort(),
    instrumentMap.sites
      .filter((site) => site.falsifier === "STRUCTURAL")
      .map((site) => site.id)
      .sort(),
    "STRUCTURAL entitlement census is stale",
  );

  const expectedAuditTests = [
    "directional_no_loss_rejects_composed_declaration_removal",
    "directional_no_loss_rejects_cross_module_declaration_removal",
    "missing_target_source_precedes_attribution_domain_validation",
    "module_reachability_partition_rejects_unassigned_live_names",
    "observedEmissionPaths",
  ];
  assert.deepEqual(
    evidence.deadTestAudit.map((record) => record.test).sort(),
    expectedAuditTests,
    "dead-test audit does not cover the declared regression lineage",
  );
  for (const record of evidence.deadTestAudit) {
    assert.ok(record.namedFix, `dead-test audit row has no named fix: ${record.test}`);
    assert.ok(record.evidence, `dead-test audit row has no evidence: ${record.test}`);
    assert.ok(record.owner, `dead-test audit row has no owner: ${record.test}`);
    assert.ok(record.reentry, `dead-test audit row has no re-entry: ${record.test}`);
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
assert.equal(
  new Set(domain.files).size,
  domain.files.length,
  "instrument domain files must be unique",
);
const instrumentMap = buildMap(domain);
const falsifierEvidence = JSON.parse(readFileSync(evidencePath, "utf8")) as FalsifierEvidenceV0;
validateFiringEvidence(instrumentMap, falsifierEvidence);
const serialized = `${JSON.stringify(instrumentMap, null, 2)}\n`;

if (process.argv.includes("--write")) {
  writeFileSync(mapPath, serialized);
} else {
  const committed = readFileSync(mapPath, "utf8");
  assert.equal(committed, serialized, "linked-emission instrument map is stale");
}

console.log(
  JSON.stringify({
    assertionSiteCount: instrumentMap.assertionSiteCount,
    structuralEntailmentCount: instrumentMap.structuralEntailmentCount,
    domainFileCount: instrumentMap.domainFiles.length,
    firingRecordCount: falsifierEvidence.firedSitesByPerturbation.length,
    deadTestAuditCount: falsifierEvidence.deadTestAudit.length,
  }),
);
