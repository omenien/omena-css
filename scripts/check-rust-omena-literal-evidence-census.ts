import { strict as assert } from "node:assert";
import fs from "node:fs";
import path from "node:path";

interface LiteralEvidenceAssignment {
  readonly sourcePath: string;
  readonly line: number;
  readonly fieldName: string;
  readonly literal: boolean;
}

interface LiteralEvidenceDisposition {
  readonly sourcePath: string;
  readonly fieldName: string;
  readonly disposition: "computed" | "renamed";
  readonly authority: string;
}

interface LiteralEvidenceCensus {
  readonly schemaVersion: "0";
  readonly product: "omena.literal-evidence-census";
  readonly fieldNameClasses: readonly string[];
  readonly excludedTestSurfaces: readonly string[];
  readonly dispositions: readonly LiteralEvidenceDisposition[];
  readonly productionLiteralAssignments: readonly LiteralEvidenceAssignment[];
}

const repoRoot = process.cwd();
const censusPath = path.join(repoRoot, "rust/omena-literal-evidence-census.json");
const writeMode = process.argv.includes("--write");
const fieldNamePattern = /(?:within_source|verified|parity)/u;
const assignmentPattern = /^\s*([a-z][a-z0-9_]*)\s*:\s*(true|false),?\s*(?:(?:\/\/).*)?$/u;

const productionLiteralAssignments = rustSourcePaths(path.join(repoRoot, "rust/crates"))
  .flatMap((sourcePath) => {
    const source = fs.readFileSync(sourcePath, "utf8");
    return collectLiteralEvidenceAssignments(path.relative(repoRoot, sourcePath), source);
  })
  .toSorted(
    (left, right) => left.sourcePath.localeCompare(right.sourcePath) || left.line - right.line,
  );

const injected = collectLiteralEvidenceAssignments(
  "injected/literal-evidence.rs",
  "EvidenceReport {\n  precision_parity: true,\n}\n",
);
assert.deepEqual(
  injected.map(({ fieldName, literal }) => ({ fieldName, literal })),
  [{ fieldName: "precision_parity", literal: true }],
  "literal-evidence predicate must detect a newly introduced parity assertion",
);
if (process.env.OMENA_LITERAL_EVIDENCE_TEST_INJECT === "1") {
  productionLiteralAssignments.push(...injected);
}

assert.deepEqual(
  productionLiteralAssignments,
  [],
  "production evidence fields must compute their value or use expectation vocabulary",
);

const census: LiteralEvidenceCensus = {
  schemaVersion: "0",
  product: "omena.literal-evidence-census",
  fieldNameClasses: ["within_source", "verified", "parity"],
  excludedTestSurfaces: [],
  dispositions: [
    {
      sourcePath: "rust/crates/omena-semantic/src/lib.rs",
      fieldName: "all_token_spans_within_source",
      disposition: "computed",
      authority: "ParsedCst token text ranges bounded by sourceByteLen",
    },
    {
      sourcePath: "rust/crates/omena-semantic/src/lib.rs",
      fieldName: "all_node_spans_within_source",
      disposition: "computed",
      authority: "ParsedCst node text ranges bounded by sourceByteLen",
    },
    {
      sourcePath: "rust/crates/omena-streaming-ifds/src/lib.rs",
      fieldName: "precision_parity_with_batch",
      disposition: "renamed",
      authority: "exactReachabilitySelected describes the selected exact SCC traversal",
    },
    {
      sourcePath: "rust/crates/omena-zk-audit/src/arkworks.rs",
      fieldName: "proof_verified",
      disposition: "computed",
      authority: "R1CS witness satisfaction result on the early rejection branch",
    },
    {
      sourcePath: "rust/crates/omena-cli/src/lock.rs",
      fieldName: "verified",
      disposition: "computed",
      authority: "Sigstore verification result after the fail-closed success check",
    },
  ],
  productionLiteralAssignments,
};

const serialized = `${JSON.stringify(census, null, 2)}\n`;
if (writeMode) {
  fs.writeFileSync(censusPath, serialized);
} else {
  assert.deepEqual(
    JSON.parse(fs.readFileSync(censusPath, "utf8")),
    census,
    "literal-evidence census is stale",
  );
}

process.stdout.write(
  `Omena literal evidence census OK: dispositions=${census.dispositions.length} productionLiterals=0\n`,
);

function collectLiteralEvidenceAssignments(
  sourcePath: string,
  source: string,
): LiteralEvidenceAssignment[] {
  const assignments: LiteralEvidenceAssignment[] = [];
  for (const [index, line] of source.split("\n").entries()) {
    const match = assignmentPattern.exec(line);
    if (!match || !fieldNamePattern.test(match[1])) {
      continue;
    }
    assignments.push({
      sourcePath,
      line: index + 1,
      fieldName: match[1],
      literal: match[2] === "true",
    });
  }
  return assignments;
}

function rustSourcePaths(root: string): string[] {
  const paths: string[] = [];
  for (const entry of fs.readdirSync(root, { withFileTypes: true })) {
    const entryPath = path.join(root, entry.name);
    if (entry.isDirectory()) {
      paths.push(...rustSourcePaths(entryPath));
      continue;
    }
    if (entry.name.endsWith(".rs")) {
      paths.push(entryPath);
    }
  }
  return paths;
}
