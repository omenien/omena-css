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
];

let failures = 0;
for (const [checker, variable, args] of redCases) {
  const result = spawnSync("node", ["--import", "tsx", checker, ...args], {
    cwd: repoRoot,
    encoding: "utf8",
    env: { ...process.env, [variable]: "1" },
  });
  const passed = result.status !== 0;
  if (!passed) failures += 1;
  const suffix = args.length > 0 ? ` ${args.join(" ")}` : "";
  process.stdout.write(`${passed ? "ok  " : "FAIL"} ${variable}${suffix} exits non-zero\n`);
}

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
  `${blindSpotDisclosed ? "ok  " : "FAIL"} unlabelled binding remains outside the idiom arm\n`,
);

process.stdout.write(`\n${redCases.length + 1 - failures}/${redCases.length + 1} passed\n`);
process.exit(failures === 0 ? 0 : 1);
