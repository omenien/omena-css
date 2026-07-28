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

const scriptPath = fileURLToPath(import.meta.url);
const repositoryRoot = resolve(dirname(scriptPath), "..");
const domainPath = resolve(repositoryRoot, "rust/omena-linked-emission-instrument-domain.json");
const mapPath = resolve(repositoryRoot, "rust/omena-linked-emission-instrument-map.json");
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
  }),
);
