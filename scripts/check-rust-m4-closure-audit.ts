import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";

const root = process.cwd();
const packageJson = JSON.parse(read("package.json")) as {
  readonly scripts: Record<string, string>;
};

const readinessScript = requiredScript("check:rust-m4-readiness");
const axisBClosureAudit = read("scripts/check-rust-m4-axis-b-closure-audit.ts");

const requiredReadinessTargets = [
  "rust/m4-axis-a-readiness",
  "rust/m4-axis-b-readiness",
  "rust/m4-axis-c-readiness",
  "rust/m4-axis-d-readiness",
  "rust/z5-performance-baseline-readiness",
  "rust/m4-closure-audit",
] as const;
const requiredAxisClosureScripts = [
  "check:rust-m4-axis-a-closure-audit",
  "check:rust-m4-axis-b-closure-audit",
  "check:rust-m4-axis-c-closure-audit",
  "check:rust-m4-axis-d-closure-audit",
] as const;

for (const target of requiredReadinessTargets) {
  assertIncludes(readinessScript, target, `M4 readiness must include ${target}`);
}

for (const scriptName of requiredAxisClosureScripts) {
  requiredScript(scriptName);
}

for (const scriptName of [
  "check:rust-m4-axis-a-readiness",
  "check:rust-m4-axis-b-readiness",
  "check:rust-m4-axis-c-readiness",
  "check:rust-m4-axis-d-readiness",
  "check:rust-z5-performance-baseline-readiness",
] as const) {
  requiredScript(scriptName);
}

assertIncludes(
  axisBClosureAudit,
  "externalAcceptanceStillRequired: true",
  "M4 aggregate audit must not hide the current #38 external acceptance blocker",
);
assertIncludes(
  axisBClosureAudit,
  "packagedGate",
  "M4 aggregate audit must retain packaged LSP protocol gate tracking for #38",
);

const status = "m4BlockedOnIssue38ExternalAcceptance";

process.stdout.write(
  JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.m4-closure-audit",
      status,
      m4Complete: false,
      readinessScript: "check:rust-m4-readiness",
      closureAudits: [...requiredAxisClosureScripts, "check:rust-m4-closure-audit"],
      axes: {
        axisA: {
          gate: "rust/m4-axis-a-readiness",
          scope: "automation-testkit-conformance",
          localGateRequired: true,
        },
        axisB: {
          gate: "rust/m4-axis-b-readiness",
          scope: "issue-61-resolver-perimeter-and-issue-38-lsp-regression",
          localGateRequired: true,
          externalAcceptanceStillRequired: true,
        },
        axisC: {
          gate: "rust/m4-axis-c-readiness",
          scope: "typed-provenance-and-cross-file-summary-edge-substrate",
          localGateRequired: true,
        },
        axisD: {
          gate: "rust/m4-axis-d-readiness",
          scope: "behavior-preserving-structural-splits",
          localGateRequired: true,
        },
      },
      benchmark: {
        gate: "rust/z5-performance-baseline-readiness",
        scope: "symmetric-benchmark-measurement-boundary",
        localGateRequired: true,
      },
      issue38: {
        githubIssue: "https://github.com/yongsk0066/css-module-explainer/issues/38",
        stateExpectedBeforeM4Close: "closed-or-maintainer-accepted-non-blocking",
        currentLocalStatus: "root-cause-regression-gates-present",
        externalAcceptanceStillRequired: true,
        packagedGate: "release/check/packaged-omena-lsp-server-type-fact-protocol",
      },
      theoryClaimGuard: {
        dynamicDyck: "notClaimed",
        externalDatalog: "notClaimed",
        egglogExecution: "notClaimed",
        sheafOrModalTheorem: "notClaimed",
        fullPerceptualTooling: "notClaimed",
      },
      nextPriorities: [
        "issue38PackagedExtensionRealWorkspaceAcceptance",
        "continueAxisARealCorpusAndSpecAuditExpansion",
        "continueAxisBResolverPerimeterEvidence",
      ],
    },
    null,
    2,
  ),
);
process.stdout.write("\n");

function read(relativePath: string): string {
  return readFileSync(path.join(root, relativePath), "utf8");
}

function requiredScript(name: string): string {
  const script = packageJson.scripts[name];
  assert.equal(typeof script, "string", `${name} must be declared in package.json`);
  return script;
}

function assertIncludes(source: string, marker: string, message: string): void {
  assert.ok(source.includes(marker), message);
}
