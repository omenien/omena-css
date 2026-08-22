#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const rawScanChecker = "scripts/check-rust-omena-syntax-authority-raw-scan-census.ts";
const identifierChecker = "scripts/check-rust-omena-identifier-authority-census.ts";

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
]);

let failures = 0;
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
  const requiresRawPropertySite = rawPropertyMutationVariables.has(variable);
  const passed = result.status !== 0 && (!requiresRawPropertySite || rawPropertySiteCount > 0);
  if (!passed) failures += 1;
  const suffix = args.length > 0 ? ` ${args.join(" ")}` : "";
  const rawReceipt = requiresRawPropertySite
    ? `; rawPropertyIdentitySiteCount=${rawPropertySiteCount}`
    : "";
  process.stdout.write(
    `${passed ? "ok  " : "FAIL"} ${variable}${suffix} exits non-zero${rawReceipt}\n`,
  );
}
const redFailures = failures;

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
  `\n${redCases.length - redFailures}/${redCases.length} RED mutation arms; ${blindSpotDisclosed ? "1/1" : "0/1"} disclosed GREEN control arm\n`,
);
process.exit(failures === 0 ? 0 : 1);
