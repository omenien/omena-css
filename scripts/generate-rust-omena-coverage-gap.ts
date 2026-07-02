import { strict as assert } from "node:assert";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

import {
  COVERAGE_GAP_REPORT_PATH,
  buildCoverageGapReportFromRepo,
  serializeCoverageGapReport,
} from "./coverage-gap-report";

const repoRoot = process.cwd();
const checkOnly = process.argv.includes("--check");
const writeMode = process.argv.includes("--write") || !checkOnly;
const reportPath = path.join(repoRoot, COVERAGE_GAP_REPORT_PATH);

const report = buildCoverageGapReportFromRepo(repoRoot);
const reportSource = serializeCoverageGapReport(report);

if (checkOnly) {
  assert.equal(
    readFileSync(reportPath, "utf8"),
    reportSource,
    `${COVERAGE_GAP_REPORT_PATH} is stale; run \`node --import tsx ./scripts/generate-rust-omena-coverage-gap.ts --write\``,
  );
} else if (writeMode) {
  writeFileSync(reportPath, reportSource);
}

process.stdout.write(
  `${JSON.stringify(
    {
      product: "omena-spec-audit.coverage-gap-generator",
      mode: checkOnly ? "check" : "write",
      generatedFiles: [COVERAGE_GAP_REPORT_PATH],
      rowCount: report.summary.rowCount,
      recognitionGapCount: report.summary.recognitionGapCount,
      foldGapCount: report.summary.foldGapCount,
      advisory: report.policy.advisory,
    },
    null,
    2,
  )}\n`,
);
