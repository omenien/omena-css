#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const rawScanChecker = "scripts/check-rust-omena-syntax-authority-raw-scan-census.ts";
const identifierChecker = "scripts/check-rust-omena-identifier-authority-census.ts";
const generatedMatrixOnly = process.argv.includes("--generated-matrix-only");

const generatedOrigins = [
  "field-access",
  "bare-parameter",
  "fqn-parameter",
  "alias-parameter",
  "closure-inferred",
  "two-statement-local",
  "wrapper-function",
  "display-escape",
];
const generatedComparisonGrammars = [
  "binary-eq",
  "binary-ne",
  "method-eq",
  "method-ne",
  "eq-ignore-ascii-case",
  "cmp-is-eq",
  "partial-cmp-is-eq",
  "ufcs-str-eq",
  "ufcs-partial-eq",
  "map-insert",
  "map-get",
  "map-entry",
  "map-contains-key",
  "map-remove",
  "set-insert",
  "set-get",
  "set-contains",
  "set-remove",
  "sort",
  "sort-by",
  "sort-by-key",
  "sort-by-cached-key",
  "sort-unstable",
  "sort-unstable-by",
  "sort-unstable-by-key",
  "dedup",
  "dedup-by",
  "dedup-by-key",
  "binary-search",
  "binary-search-by",
  "binary-search-by-key",
  "match-literal",
  "matches-literal",
];
const generatedPositions = ["same-file", "cross-file", "authority-zero-file"];

function vectorComparisonBody(operation) {
  return `{ let mut values: Vec<String> = Vec::new(); values.push(value); ${operation}; true }`;
}

function mapComparisonBody(operation) {
  return `{ let mut values: std::collections::BTreeMap<String, u8> = std::collections::BTreeMap::new(); values.insert(value, 1); ${operation}; true }`;
}

function comparisonBody(grammar) {
  switch (grammar) {
    case "binary-eq":
      return "value == expected";
    case "binary-ne":
      return "value != expected";
    case "method-eq":
      return "value.eq(expected)";
    case "method-ne":
      return "value.ne(expected)";
    case "eq-ignore-ascii-case":
      return "value.eq_ignore_ascii_case(expected)";
    case "cmp-is-eq":
      return "value.cmp(&expected.to_string()).is_eq()";
    case "partial-cmp-is-eq":
      return "value.partial_cmp(&expected.to_string()).is_some_and(std::cmp::Ordering::is_eq)";
    case "ufcs-str-eq":
      return "str::eq(value.as_str(), expected)";
    case "ufcs-partial-eq":
      return "PartialEq::eq(&value, &expected.to_string())";
    case "map-insert":
      return "{ let mut values: std::collections::BTreeMap<String, u8> = std::collections::BTreeMap::new(); values.insert(value, 1); true }";
    case "map-get":
      return mapComparisonBody("let _ = values.get(expected)");
    case "map-entry":
      return mapComparisonBody("let _ = values.entry(expected.to_string())");
    case "map-contains-key":
      return mapComparisonBody("let _ = values.contains_key(expected)");
    case "map-remove":
      return mapComparisonBody("let _ = values.remove(expected)");
    case "set-insert":
      return "{ let mut values: std::collections::BTreeSet<String> = std::collections::BTreeSet::new(); values.insert(value); true }";
    case "set-get":
      return "{ let mut values: std::collections::BTreeSet<String> = std::collections::BTreeSet::new(); values.insert(value); let _ = values.get(expected); true }";
    case "set-contains":
      return "{ let mut values: std::collections::BTreeSet<String> = std::collections::BTreeSet::new(); values.insert(value); let _ = values.contains(expected); true }";
    case "set-remove":
      return "{ let mut values: std::collections::BTreeSet<String> = std::collections::BTreeSet::new(); values.insert(value); let _ = values.remove(expected); true }";
    case "sort":
      return vectorComparisonBody("values.sort() ");
    case "sort-by":
      return vectorComparisonBody("values.sort_by(|left, right| left.cmp(right))");
    case "sort-by-key":
      return vectorComparisonBody("values.sort_by_key(|item| item.clone())");
    case "sort-by-cached-key":
      return vectorComparisonBody("values.sort_by_cached_key(|item| item.clone())");
    case "sort-unstable":
      return vectorComparisonBody("values.sort_unstable() ");
    case "sort-unstable-by":
      return vectorComparisonBody("values.sort_unstable_by(|left, right| left.cmp(right))");
    case "sort-unstable-by-key":
      return vectorComparisonBody("values.sort_unstable_by_key(|item| item.clone())");
    case "dedup":
      return vectorComparisonBody("values.dedup() ");
    case "dedup-by":
      return vectorComparisonBody("values.dedup_by(|left, right| left == right)");
    case "dedup-by-key":
      return vectorComparisonBody("values.dedup_by_key(|item| item.clone())");
    case "binary-search":
      return vectorComparisonBody("let _ = values.binary_search(&expected.to_string())");
    case "binary-search-by":
      return vectorComparisonBody(
        "let _ = values.binary_search_by(|item| item.as_str().cmp(expected))",
      );
    case "binary-search-by-key":
      return vectorComparisonBody(
        "let _ = values.binary_search_by_key(&expected.to_string(), |item| item.clone())",
      );
    case "match-literal":
      return 'match value.as_str() { "--token" => true, _ => false }';
    case "matches-literal":
      return 'matches!(value.as_str(), "--token")';
    default:
      throw new Error(`unknown generated comparison grammar: ${grammar}`);
  }
}

function generatedCellSource(functionName, origin, carrierType, wrapperName, grammar) {
  const operation = comparisonBody(grammar);
  switch (origin) {
    case "field-access":
      return `fn ${functionName}(carrier: &${carrierType}, expected: &str) -> bool { let value = carrier.property.to_string(); ${operation} }`;
    case "bare-parameter":
      return `fn ${functionName}(property: &AuthoredPropertyTextV0, expected: &str) -> bool { let value = property.to_string(); ${operation} }`;
    case "fqn-parameter":
      return `fn ${functionName}(property: &omena_syntax::ident::AuthoredPropertyTextV0, expected: &str) -> bool { let value = property.to_string(); ${operation} }`;
    case "alias-parameter":
      return `fn ${functionName}(property: &AuthoredText, expected: &str) -> bool { let value = property.to_string(); ${operation} }`;
    case "closure-inferred":
      return `fn ${functionName}(carriers: &[${carrierType}], expected: &str) -> bool { carriers.iter().any(|carrier| { let value = carrier.property.to_string(); ${operation} }) }`;
    case "two-statement-local":
      return `fn ${functionName}(carrier: &${carrierType}, expected: &str) -> bool { let rendered = carrier.property.to_string(); let value = rendered; ${operation} }`;
    case "wrapper-function":
      return `fn ${functionName}(carrier: &${carrierType}, expected: &str) -> bool { let value = ${wrapperName}(&carrier.property); ${operation} }`;
    case "display-escape":
      return `fn ${functionName}(carrier: &${carrierType}, expected: &str) -> bool { let value = format!("{}", carrier.property); ${operation} }`;
    default:
      throw new Error(`unknown generated origin: ${origin}`);
  }
}

function generatedMatrixManifest() {
  const sources = [];
  const expectedCellFunctions = [];
  for (const [positionIndex, position] of generatedPositions.entries()) {
    const carrierName = `GeneratedPropertyCarrier${positionIndex}`;
    const wrapperName = `render_generated_property_${positionIndex}`;
    const carrierPath =
      position === "cross-file"
        ? "rust/crates/omena-cascade/src/generated_property_matrix_axis_order.rs"
        : `rust/crates/omena-query/src/generated_property_matrix_${positionIndex}.rs`;
    const consumerPath =
      position === "cross-file"
        ? "rust/crates/omena-cascade/src/generated_property_matrix_ranking.rs"
        : carrierPath;
    const prelude = `use omena_syntax::ident::AuthoredPropertyTextV0;\nuse omena_syntax::ident::AuthoredPropertyTextV0 as AuthoredText;\nstruct ${carrierName} { property: AuthoredPropertyTextV0 }\nfn ${wrapperName}(value: &AuthoredPropertyTextV0) -> String { value.to_string() }\n`;
    if (carrierPath !== consumerPath) sources.push({ relativePath: carrierPath, source: prelude });
    const functions = [];
    for (const [originIndex, origin] of generatedOrigins.entries()) {
      for (const [grammarIndex, grammar] of generatedComparisonGrammars.entries()) {
        const functionName = `generated_property_cell_${positionIndex}_${originIndex}_${grammarIndex}`;
        expectedCellFunctions.push(functionName);
        functions.push(
          generatedCellSource(functionName, origin, carrierName, wrapperName, grammar),
        );
      }
    }
    sources.push({
      relativePath: consumerPath,
      source: `${carrierPath === consumerPath ? prelude : `use crate::axis_order::${carrierName};\nuse omena_syntax::ident::AuthoredPropertyTextV0;\nuse omena_syntax::ident::AuthoredPropertyTextV0 as AuthoredText;\nfn ${wrapperName}(value: &AuthoredPropertyTextV0) -> String { value.to_string() }\n`}\n${functions.join("\n")}`,
    });
  }
  return { schemaVersion: "0", sources, expectedCellFunctions };
}

const redCases = [
  [rawScanChecker, "OMENA_SYNTAX_AUTHORITY_TEST_INJECT_RAW_SCAN", []],
  [rawScanChecker, "OMENA_SYNTAX_AUTHORITY_TEST_INJECT_TOKEN_CASE_COMPARE", []],
  [rawScanChecker, "OMENA_SYNTAX_AUTHORITY_TEST_INJECT_LEXER_CASE_COMPARE", []],
  [rawScanChecker, "OMENA_SYNTAX_AUTHORITY_TEST_INJECT_TOKEN_CASE_EXEMPTION_DRIFT", []],
  [rawScanChecker, "OMENA_SYNTAX_AUTHORITY_TEST_INJECT_LESS_SCANNER_CALL", []],
  [rawScanChecker, "OMENA_SYNTAX_AUTHORITY_TEST_INJECT_CLASS_SCANNER", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_CLASSNAME_EQUALITY", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_EGRESS", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_EGRESS", ["--write"]],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_LABELLED_COMPARISON", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PREDICATE_COPY", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PREDICATE_COPY_EXPLICIT", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PREDICATE_COPY_REVERSED", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_STRUCTURAL_EQUALITY", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_ROUNDTRIP_EQUALITY", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_RAW_MAP", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_RAW_CANONICALIZATION", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_FQN_RAW_MAP", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_VALUES_RAW_MAP", []],
  [
    identifierChecker,
    "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_SAME_LINE_RAW_OPERATION",
    [],
  ],
  [
    identifierChecker,
    "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_NEW_FILE_RAW_COMPARISON",
    [],
  ],
  [
    identifierChecker,
    "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_NEW_FILE_RAW_CANONICALIZATION",
    [],
  ],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_TRIM_CHAIN", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_CONTEXT_RAW_OPERATIONS", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_AUTOMATIC_CARRIER", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_REAL_FILE_MUTATION", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_CASE_FOLD", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_DECODE_NEUTER", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_MIGRATE_LOWERCASE_COMPARISON", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_MIGRATE_FQN_PARAMETER", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_MIGRATE_ALIAS_PARAMETER", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_MIGRATE_BARE_PARAMETER", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_MIGRATE_CLOSURE_PARAMETER", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_AUTHORED_UPPERCASE_TRANSFORM", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_AUTHORED_TRIM_MATCHES_TRANSFORM", []],
  [identifierChecker, "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_AUTHORED_STRIP_PREFIX_TRANSFORM", []],
];

const rawPropertyMutationVariables = new Set([
  "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_STRUCTURAL_EQUALITY",
  "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_ROUNDTRIP_EQUALITY",
  "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_RAW_MAP",
  "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_RAW_CANONICALIZATION",
  "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_FQN_RAW_MAP",
  "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_VALUES_RAW_MAP",
  "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_SAME_LINE_RAW_OPERATION",
  "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_NEW_FILE_RAW_COMPARISON",
  "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_NEW_FILE_RAW_CANONICALIZATION",
  "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_TRIM_CHAIN",
  "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_CONTEXT_RAW_OPERATIONS",
  "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_AUTOMATIC_CARRIER",
  "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_REAL_FILE_MUTATION",
  "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_AUTHORED_UPPERCASE_TRANSFORM",
  "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_AUTHORED_TRIM_MATCHES_TRANSFORM",
  "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_AUTHORED_STRIP_PREFIX_TRANSFORM",
]);
const residualConsumerMutationVariables = new Set([
  "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_MIGRATE_LOWERCASE_COMPARISON",
  "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_MIGRATE_FQN_PARAMETER",
  "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_MIGRATE_ALIAS_PARAMETER",
  "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_MIGRATE_BARE_PARAMETER",
  "OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_MIGRATE_CLOSURE_PARAMETER",
]);

let failures = 0;
if (!generatedMatrixOnly) {
  for (const [checker, variable, args] of redCases) {
    const result = spawnSync("node", ["--import", "tsx", checker, ...args], {
      cwd: repoRoot,
      encoding: "utf8",
      env: { ...process.env, [variable]: "1" },
    });
    const output = `${result.stdout ?? ""}\n${result.stderr ?? ""}`;
    const rawPropertySiteCount = Number(
      output.match(/rawPropertyIdentitySiteCount=(\d+)/u)?.[1] ?? "0",
    );
    const residualIdentityConsumerCount = Number(
      output.match(/residualIdentityShapedConsumerCount=(\d+)/u)?.[1] ?? "0",
    );
    const requiresRawPropertySite = rawPropertyMutationVariables.has(variable);
    const requiresResidualIdentityConsumer = residualConsumerMutationVariables.has(variable);
    const passed =
      result.status !== 0 &&
      (!requiresRawPropertySite || rawPropertySiteCount > 0) &&
      (!requiresResidualIdentityConsumer || residualIdentityConsumerCount > 0);
    if (!passed) failures += 1;
    const suffix = args.length > 0 ? ` ${args.join(" ")}` : "";
    const rawReceipt = requiresRawPropertySite
      ? `; rawPropertyIdentitySiteCount=${rawPropertySiteCount}`
      : "";
    const residualReceipt = requiresResidualIdentityConsumer
      ? `; residualIdentityShapedConsumerCount=${residualIdentityConsumerCount}`
      : "";
    process.stdout.write(
      `${passed ? "ok  " : "FAIL"} ${variable}${suffix} exits non-zero${rawReceipt}${residualReceipt}\n`,
    );
  }
}
const redFailures = failures;

const generatedTempRoot = mkdtempSync(path.join(tmpdir(), "omena-identifier-matrix-"));
const generatedManifestPath = path.join(generatedTempRoot, "manifest.json");
const generatedManifest = generatedMatrixManifest();
writeFileSync(generatedManifestPath, `${JSON.stringify(generatedManifest)}\n`);
const generatedResult = spawnSync("node", ["--import", "tsx", identifierChecker], {
  cwd: repoRoot,
  encoding: "utf8",
  env: {
    ...process.env,
    OMENA_IDENTIFIER_AUTHORITY_GENERATED_FIXTURE_MANIFEST: generatedManifestPath,
  },
});
const generatedOutput = `${generatedResult.stdout ?? ""}\n${generatedResult.stderr ?? ""}`;
const generatedDetectedCount = Number(
  generatedOutput.match(/generatedPropertyIdentityCellCount=(\d+)/u)?.[1] ?? "0",
);
const generatedPassed =
  generatedResult.status !== 0 &&
  generatedDetectedCount === generatedManifest.expectedCellFunctions.length &&
  !generatedOutput.includes("undetected origin x grammar x position cells");
rmSync(generatedTempRoot, { recursive: true, force: true });
if (!generatedPassed) {
  failures += 1;
  process.stderr.write(generatedOutput);
}
process.stdout.write(
  `${generatedPassed ? "ok  " : "FAIL"} generated authored-origin matrix: ${generatedDetectedCount}/${generatedManifest.expectedCellFunctions.length} cells RED (${generatedOrigins.length} origins x ${generatedComparisonGrammars.length} grammars x ${generatedPositions.length} positions)\n`,
);
if (generatedMatrixOnly) process.exit(generatedPassed ? 0 : 1);

const committedLaunderingCensusPath = path.join(
  repoRoot,
  "rust/omena-identifier-authority-census.json",
);
const exactLaunderingTempRoot = mkdtempSync(path.join(tmpdir(), "omena-identifier-laundering-"));
const exactLaunderingCensusPath = path.join(exactLaunderingTempRoot, "census.json");
writeFileSync(exactLaunderingCensusPath, readFileSync(committedLaunderingCensusPath));
const originalLaunderingCensus = readFileSync(exactLaunderingCensusPath);
const baselineAuthorityCount = JSON.parse(originalLaunderingCensus.toString("utf8"))
  .propertyIdentity.authoritySiteCount;
let exactLaunderingPassed = false;
let exactLaunderingWriteCount = 0;
let exactLaunderingRecheckCount = 0;
let exactLaunderingWriteAuthorityCount = 0;
let exactLaunderingRecheckAuthorityCount = 0;

try {
  const writeAttempt = spawnSync("node", ["--import", "tsx", identifierChecker, "--write"], {
    cwd: repoRoot,
    encoding: "utf8",
    env: {
      ...process.env,
      OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_AUTHORITY_DECREASE_LAUNDERING: "1",
      OMENA_IDENTIFIER_AUTHORITY_CENSUS_PATH: exactLaunderingCensusPath,
    },
  });
  const writeOutput = `${writeAttempt.stdout ?? ""}\n${writeAttempt.stderr ?? ""}`;
  exactLaunderingWriteCount = Number(
    writeOutput.match(/rawPropertyIdentitySiteCount=(\d+)/u)?.[1] ?? "0",
  );
  exactLaunderingWriteAuthorityCount = Number(
    writeOutput.match(/propertyAuthoritySiteCount=(\d+)/u)?.[1] ?? "0",
  );
  const censusUnchangedAfterWrite =
    readFileSync(exactLaunderingCensusPath).equals(originalLaunderingCensus);

  const recheck = spawnSync("node", ["--import", "tsx", identifierChecker], {
    cwd: repoRoot,
    encoding: "utf8",
    env: {
      ...process.env,
      OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_PROPERTY_AUTHORITY_DECREASE_LAUNDERING: "1",
      OMENA_IDENTIFIER_AUTHORITY_CENSUS_PATH: exactLaunderingCensusPath,
    },
  });
  const recheckOutput = `${recheck.stdout ?? ""}\n${recheck.stderr ?? ""}`;
  exactLaunderingRecheckCount = Number(
    recheckOutput.match(/rawPropertyIdentitySiteCount=(\d+)/u)?.[1] ?? "0",
  );
  exactLaunderingRecheckAuthorityCount = Number(
    recheckOutput.match(/propertyAuthoritySiteCount=(\d+)/u)?.[1] ?? "0",
  );
  const censusUnchangedAfterRecheck =
    readFileSync(exactLaunderingCensusPath).equals(originalLaunderingCensus);

  exactLaunderingPassed =
    writeAttempt.status !== 0 &&
    exactLaunderingWriteCount > 0 &&
    exactLaunderingWriteAuthorityCount === baselineAuthorityCount - 1 &&
    recheck.status !== 0 &&
    exactLaunderingRecheckCount > 0 &&
    exactLaunderingRecheckAuthorityCount === baselineAuthorityCount - 1 &&
    censusUnchangedAfterWrite &&
    censusUnchangedAfterRecheck;
} catch (error) {
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
} finally {
  rmSync(exactLaunderingTempRoot, { recursive: true, force: true });
}

if (!exactLaunderingPassed) failures += 1;
process.stdout.write(
  `${exactLaunderingPassed ? "ok  " : "FAIL"} exact laundering: eq_ignore_ascii_case; authority ${baselineAuthorityCount}->${exactLaunderingWriteAuthorityCount}; --write ${exactLaunderingWriteCount > 0 ? "RED" : "MISS"} raw=${exactLaunderingWriteCount}; recheck ${exactLaunderingRecheckCount > 0 ? "RED" : "MISS"} raw=${exactLaunderingRecheckCount}; census unchanged; in-memory mutation\n`,
);

const unlabelled = spawnSync("node", ["--import", "tsx", identifierChecker], {
  cwd: repoRoot,
  encoding: "utf8",
  env: {
    ...process.env,
    OMENA_IDENTIFIER_AUTHORITY_TEST_INJECT_UNLABELLED_COMPARISON: "1",
  },
});
const blindSpotDisclosed = unlabelled.status === 0;
if (!blindSpotDisclosed) failures += 1;
process.stdout.write(
  `${blindSpotDisclosed ? "ok  " : "FAIL"} disclosed GREEN control: unlabelled class binding remains outside the idiom arm\n`,
);

process.stdout.write(
  `\n${redCases.length - redFailures}/${redCases.length} injected RED mutation arms; ${generatedPassed ? `${generatedDetectedCount}/${generatedManifest.expectedCellFunctions.length}` : `0/${generatedManifest.expectedCellFunctions.length}`} generated matrix cells; ${exactLaunderingPassed ? "1/1" : "0/1"} exact laundering arm; ${blindSpotDisclosed ? "1/1" : "0/1"} disclosed GREEN control arm\n`,
);
process.exit(failures === 0 ? 0 : 1);
