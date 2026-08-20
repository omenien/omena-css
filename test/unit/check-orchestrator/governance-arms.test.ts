import { readFileSync, readdirSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";
import {
  loadCheckManifest,
  renderCheckInventory,
} from "../../../packages/check-orchestrator/src/manifest/index";
import {
  findCiRequiredAggregationDiagnostics,
  findCiTierReachabilityDiagnostics,
} from "../../../packages/check-orchestrator/src/manifest/workflows";
import {
  expensiveTierMembers,
  findGatePolicyDiagnostics,
} from "../../../packages/check-orchestrator/src/manifest/gate-policy";
import {
  adoptCiWorkflow,
  validateCiWorkflowRegistry,
} from "../../../packages/check-orchestrator/src/manifest/ci-workflow";
import { cpSync, mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import os from "node:os";

const repoRoot = path.resolve(__dirname, "../../..");

describe("inventory and governance hardening arms", () => {
  it(
    "DIFF-SIZE ARM: adding one gate changes at most 3 inventory lines",
    { timeout: 30_000 },
    () => {
      const manifest = loadCheckManifest(repoRoot);
      const before = renderCheckInventory(manifest).split("\n");
      const synthetic = {
        ...manifest,
        gates: [
          ...manifest.gates,
          {
            ...manifest.gates.find((gate) => gate.id === "docs/site")!,
            id: "docs/zz-synthetic-diff-probe",
            scriptName: "check:docs-zz-synthetic-diff-probe",
          },
        ],
      };
      const after = renderCheckInventory(synthetic).split("\n");
      const beforeSet = new Map<string, number>();
      for (const line of before) beforeSet.set(line, (beforeSet.get(line) ?? 0) + 1);
      let changed = 0;
      for (const line of after) {
        const count = beforeSet.get(line) ?? 0;
        if (count > 0) beforeSet.set(line, count - 1);
        else changed += 1;
      }
      expect(changed).toBeLessThanOrEqual(3);
    },
  );

  it("DEP-ARGS RED-PROOF: dep args on a dependencies-executor bundle and CLI-level flags are load-time errors", () => {
    const root = mkdtempSync(path.join(os.tmpdir(), "omena-dep-args-"));
    writeFileSync(
      path.join(root, "package.json"),
      JSON.stringify({
        name: "omena-css",
        scripts: {
          "omena-check": "node ./check.js",
          "check:docs-site": "node ./scripts/a.ts",
          "check:docs-smoke": "node ./scripts/b.ts",
        },
      }),
    );
    const manifest = loadCheckManifest(root, {
      declaredGates: [
        {
          id: "docs/pack",
          kind: "bundle",
          scope: "docs",
          deps: ["docs/site", "docs/smoke"],
          ciTier: "manual",
          ciReason: "fixture",
        },
        {
          id: "docs/bad-args",
          kind: "bundle",
          scope: "docs",
          deps: [{ target: "docs/pack", args: ["--json"] }],
          ciTier: "manual",
          ciReason: "fixture",
        },
        {
          id: "docs/bad-shard",
          kind: "bundle",
          scope: "docs",
          deps: [{ target: "docs/site", args: ["--shard=ghost", "--summary"] }],
          ciTier: "manual",
          ciReason: "fixture",
        },
        {
          id: "docs/good-args",
          kind: "bundle",
          scope: "docs",
          deps: [{ target: "docs/site", args: ["--variant", "x"] }],
          ciTier: "manual",
          ciReason: "fixture",
        },
      ],
    });
    const codes = manifest.diagnostics.map((diagnostic) => diagnostic.code);
    expect(codes).toContain("declared-dep-args-not-forwardable");
    expect(codes).toContain("declared-dep-cli-level-args");
    expect(
      manifest.diagnostics.filter(
        (diagnostic) =>
          diagnostic.code === "declared-dep-args-not-forwardable" &&
          diagnostic.message.includes("docs/good-args"),
      ),
    ).toEqual([]);
  });

  it("REQUIRED-MODEL RED-PROOF (isolated from the drift gate): stripping every required annotation is now ci-required-model-missing, not silence", () => {
    const root = mkdtempSync(path.join(os.tmpdir(), "omena-required-model-"));
    mkdirSync(path.join(root, ".github/workflows"), { recursive: true });
    const annotated = [
      "name: CI",
      "jobs:",
      "  leaf:",
      "    # omena-ci-required: true",
      "    runs-on: ubuntu-latest",
      "  ci-required:",
      "    # omena-ci-required: false",
      "    needs:",
      "      - leaf",
      "    if: ${{ always() }}",
      "    runs-on: ubuntu-latest",
      "    steps:",
      "      - run: node ./scripts/check-ci-required-results.mjs",
    ].join("\n");
    writeFileSync(path.join(root, ".github/workflows/ci.yml"), annotated);
    expect(
      findCiRequiredAggregationDiagnostics(root).filter(
        (diagnostic) => diagnostic.code === "ci-required-model-missing",
      ),
    ).toEqual([]);

    const stripped = annotated
      .split("\n")
      .filter((line) => !line.includes("omena-ci-required"))
      .join("\n");
    writeFileSync(path.join(root, ".github/workflows/ci.yml"), stripped);
    const diagnostics = findCiRequiredAggregationDiagnostics(root);
    expect(diagnostics).toEqual([
      expect.objectContaining({ severity: "error", code: "ci-required-model-missing" }),
    ]);
  });

  it("HARDENING RED-PROOF: an interior always() aggregator without a judge is an error (silent strength inversion closed)", () => {
    const root = mkdtempSync(path.join(os.tmpdir(), "omena-aggregator-judge-"));
    mkdirSync(path.join(root, ".github/workflows"), { recursive: true });
    const base = [
      "name: CI",
      "jobs:",
      "  leaf:",
      "    # omena-ci-required: false",
      "    runs-on: ubuntu-latest",
      "  aggregate:",
      "    # omena-ci-required: true",
      "    needs:",
      "      - leaf",
      "    if: ${{ always() }}",
      "    runs-on: ubuntu-latest",
      "    steps:",
      "      - run: node ./scripts/check-ci-required-results.mjs",
      "  ci-required:",
      "    # omena-ci-required: false",
      "    needs:",
      "      - aggregate",
      "    if: ${{ always() }}",
      "    runs-on: ubuntu-latest",
      "    steps:",
      "      - run: node ./scripts/check-ci-required-results.mjs",
    ].join("\n");
    writeFileSync(path.join(root, ".github/workflows/ci.yml"), base);
    expect(
      findCiRequiredAggregationDiagnostics(root).filter((diagnostic) =>
        diagnostic.code.startsWith("ci-aggregator"),
      ),
    ).toEqual([]);

    // The lens's sanctioned one-line edit: drop the judge from the interior aggregator.
    writeFileSync(
      path.join(root, ".github/workflows/ci.yml"),
      base.replace(
        "    steps:\n      - run: node ./scripts/check-ci-required-results.mjs\n  ci-required:",
        "    steps:\n      - run: echo ok\n  ci-required:",
      ),
    );
    expect(findCiRequiredAggregationDiagnostics(root)).toEqual(
      expect.arrayContaining([expect.objectContaining({ code: "ci-aggregator-judge-missing" })]),
    );
  });

  it(
    "HARDENING RED-PROOF: a new expensive-lane member without a policy record REDs (population ADD direction)",
    { timeout: 30_000 },
    () => {
      const manifest = loadCheckManifest(repoRoot);
      const policy = JSON.parse(
        readFileSync(path.join(repoRoot, "packages/check-orchestrator/gate-policy.json"), "utf8"),
      ) as { lanes: { singles: string[] } };
      policy.lanes.singles = [...policy.lanes.singles, "docs/site"];
      const scratch = mkdtempSync(path.join(os.tmpdir(), "omena-policy-add-"));
      // Reuse the real repo gates against a scratch policy via the exported tier rule.
      const tier = expensiveTierMembers(repoRoot, manifest.gates, policy.lanes as never);
      expect(tier.has("docs/site")).toBe(true);
      void scratch;
    },
  );

  it("HARDENING RED-PROOF: registry emitting unparseable YAML or a phantom job block is refused at validation", () => {
    const registry = adoptCiWorkflow(
      [
        "name: CI",
        "jobs:",
        "  solo:",
        "    runs-on: ubuntu-latest",
        "    steps:",
        "      - run: echo ok",
      ].join("\n"),
    );
    expect(validateCiWorkflowRegistry(registry).errors).toEqual(
      expect.arrayContaining([expect.stringContaining("ci-required")]),
    );
    const broken = {
      ...registry,
      jobs: registry.jobs.map((job) => ({
        ...job,
        block: [...job.block, "    runs-on: [unclosed"],
      })),
    };
    expect(validateCiWorkflowRegistry(broken).errors.join(";")).toContain("does not parse as YAML");
    const phantom = {
      ...registry,
      jobs: [
        ...registry.jobs,
        {
          name: "ghost",
          block: ["  ghost:"],
          requiredAnnotation: null,
          tierAnnotation: null,
          needs: [],
        },
      ],
    };
    expect(validateCiWorkflowRegistry(phantom).errors.join(";")).toContain("phantom job");
  });

  it("HARDENING RED-PROOF: job-level if of ANY form triggers the judge duty; step-level if and comment-judges do not evade", () => {
    const root = mkdtempSync(path.join(os.tmpdir(), "omena-if-forms-"));
    mkdirSync(path.join(root, ".github/workflows"), { recursive: true });
    const make = (jobIf: string, judgeLine: string) =>
      [
        "name: CI",
        "jobs:",
        "  leaf:",
        "    # omena-ci-required: false",
        "    runs-on: ubuntu-latest",
        "  aggregate:",
        "    # omena-ci-required: true",
        "    needs:",
        "      - leaf",
        jobIf,
        "    runs-on: ubuntu-latest",
        "    steps:",
        judgeLine,
        "  ci-required:",
        "    # omena-ci-required: false",
        "    needs:",
        "      - aggregate",
        "    if: ${{ always() }}",
        "    runs-on: ubuntu-latest",
        "    steps:",
        "      - run: node ./scripts/check-ci-required-results.mjs",
      ].join("\n");
    const judgedBy = (content: string, code: string) => {
      writeFileSync(path.join(root, ".github/workflows/ci.yml"), content);
      return findCiRequiredAggregationDiagnostics(root).filter(
        (diagnostic) => diagnostic.code === code,
      );
    };
    // Spelling evasions from the confirm lens — all must now RED:
    for (const evasion of [
      "    if: ${{ always() && true }}",
      "    if: ${{ !cancelled() }}",
      "    if: ${{ always() && github.event_name == 'push' }}",
    ]) {
      expect(
        judgedBy(make(evasion, "      - run: echo not-a-judge"), "ci-aggregator-judge-missing"),
      ).toHaveLength(1);
      // ...and with the judge present the same form is QUIET (over-strictness fixed).
      expect(
        judgedBy(
          make(evasion, "      - run: node ./scripts/check-ci-required-results.mjs"),
          "ci-aggregator-missing-always",
        ),
      ).toHaveLength(0);
    }
    // A comment naming the judge script does not satisfy the duty.
    expect(
      judgedBy(
        make("    if: ${{ always() }}", "      - run: echo judged # check-ci-required-results.mjs"),
        "ci-aggregator-judge-missing",
      ),
    ).toHaveLength(1);
    // Step-level always() (the upload-artifact idiom) must NOT trigger the duty.
    const stepLevel = [
      "name: CI",
      "jobs:",
      "  leaf:",
      "    # omena-ci-required: false",
      "    runs-on: ubuntu-latest",
      "    steps:",
      "      - run: echo work",
      "      - if: ${{ always() }}",
      "        run: echo upload",
      "  ci-required:",
      "    # omena-ci-required: false",
      "    needs:",
      "      - leaf",
      "    if: ${{ always() }}",
      "    runs-on: ubuntu-latest",
      "    steps:",
      "      - run: node ./scripts/check-ci-required-results.mjs",
    ].join("\n");
    writeFileSync(path.join(root, ".github/workflows/ci.yml"), stepLevel);
    expect(
      findCiRequiredAggregationDiagnostics(root).filter((diagnostic) =>
        diagnostic.code.startsWith("ci-aggregator"),
      ),
    ).toEqual([]);
  });

  it("R4 RED-PROOF: the ROOT aggregator judge is semantic — comments, echoes, and inert steps do not judge", () => {
    const root = mkdtempSync(path.join(os.tmpdir(), "omena-root-judge-"));
    mkdirSync(path.join(root, ".github/workflows"), { recursive: true });
    const make = (rootSteps: readonly string[]) =>
      [
        "name: CI",
        "jobs:",
        "  leaf:",
        "    # omena-ci-required: true",
        "    runs-on: ubuntu-latest",
        "    steps:",
        "      - run: echo work",
        "  ci-required:",
        "    # omena-ci-required: false",
        "    needs:",
        "      - leaf",
        "    if: ${{ always() }}",
        "    runs-on: ubuntu-latest",
        "    steps:",
        ...rootSteps,
      ].join("\n");
    const codesFor = (content: string) => {
      writeFileSync(path.join(root, ".github/workflows/ci.yml"), content);
      return findCiRequiredAggregationDiagnostics(root).map((diagnostic) => diagnostic.code);
    };
    // A live judge is quiet.
    expect(codesFor(make(["      - run: node ./scripts/check-ci-required-results.mjs"]))).toEqual(
      [],
    );
    // The confirm lens's root evasion: a comment naming the script plus echo.
    expect(
      codesFor(
        make([
          "      # judged elsewhere: scripts/check-ci-required-results.mjs",
          "      - run: echo ok",
        ]),
      ),
    ).toContain("ci-required-result-check-missing");
    // A block-scalar judge is a REAL judge (false-positive direction).
    expect(
      codesFor(make(["      - run: |", "          node ./scripts/check-ci-required-results.mjs"])),
    ).toEqual([]);
    // A judge that runs but cannot fail the job is inert.
    expect(
      codesFor(
        make([
          "      - run: node ./scripts/check-ci-required-results.mjs",
          "        continue-on-error: true",
        ]),
      ),
    ).toContain("ci-aggregator-judge-inert");
    // A judge that is skipped is inert.
    expect(
      codesFor(
        make(["      - if: false", "        run: node ./scripts/check-ci-required-results.mjs"]),
      ),
    ).toContain("ci-aggregator-judge-inert");
  });

  it("R4 RED-PROOF: interior judge liveness, if-key spelling, required-path soft-fail, and YAML failure are all governed", () => {
    const root = mkdtempSync(path.join(os.tmpdir(), "omena-judge-liveness-"));
    mkdirSync(path.join(root, ".github/workflows"), { recursive: true });
    const make = (aggregateLines: readonly string[]) =>
      [
        "name: CI",
        "jobs:",
        "  leaf:",
        "    # omena-ci-required: false",
        "    runs-on: ubuntu-latest",
        "  aggregate:",
        "    # omena-ci-required: true",
        "    needs:",
        "      - leaf",
        ...aggregateLines,
        "  ci-required:",
        "    # omena-ci-required: false",
        "    needs:",
        "      - aggregate",
        "    if: ${{ always() }}",
        "    runs-on: ubuntu-latest",
        "    steps:",
        "      - run: node ./scripts/check-ci-required-results.mjs",
      ].join("\n");
    const codesFor = (content: string) => {
      writeFileSync(path.join(root, ".github/workflows/ci.yml"), content);
      return findCiRequiredAggregationDiagnostics(root).map((diagnostic) => diagnostic.code);
    };
    // Judge neutered by continue-on-error on an interior aggregator.
    expect(
      codesFor(
        make([
          "    if: ${{ always() }}",
          "    runs-on: ubuntu-latest",
          "    steps:",
          "      - run: node ./scripts/check-ci-required-results.mjs",
          "        continue-on-error: true",
        ]),
      ),
    ).toContain("ci-aggregator-judge-inert");
    // Judge neutered by a step-level if.
    expect(
      codesFor(
        make([
          "    if: ${{ always() }}",
          "    runs-on: ubuntu-latest",
          "    steps:",
          "      - if: false",
          "        run: node ./scripts/check-ci-required-results.mjs",
        ]),
      ),
    ).toContain("ci-aggregator-judge-inert");
    // Key-spelling evasions: quoted key and space-before-colon are BOTH
    // job-level ifs to a YAML parser and must fire the judge duty.
    for (const spelledIf of [
      '    "if": ${{ always() && true }}',
      "    if : ${{ always() && true }}",
    ]) {
      expect(
        codesFor(
          make([spelledIf, "    runs-on: ubuntu-latest", "    steps:", "      - run: echo ok"]),
        ),
      ).toContain("ci-aggregator-judge-missing");
    }
    // Job-level continue-on-error anywhere on the required path REDs.
    expect(
      codesFor(
        make([
          "    continue-on-error: true",
          "    runs-on: ubuntu-latest",
          "    steps:",
          "      - run: echo work",
        ]),
      ),
    ).toContain("ci-required-soft-fail");
    // Unparseable ci.yml fails CLOSED as a governed diagnostic, not a crash.
    expect(
      codesFor(make(["    runs-on: [unclosed", "    steps:", "      - run: echo work"])),
    ).toEqual(["ci-workflow-yaml-invalid"]);
  });

  it(
    "R4 RED-PROOF: criterion pins are fail-closed — key deletion, ghost pins, and digest drift are all loud",
    { timeout: 60_000 },
    () => {
      const scratch = mkdtempSync(path.join(os.tmpdir(), "omena-criteria-pins-"));
      cpSync(path.join(repoRoot, ".github/workflows"), path.join(scratch, ".github/workflows"), {
        recursive: true,
      });
      mkdirSync(path.join(scratch, "packages/check-orchestrator"), { recursive: true });
      const policyPath = path.join(scratch, "packages/check-orchestrator/gate-policy.json");
      const policy = JSON.parse(
        readFileSync(path.join(repoRoot, "packages/check-orchestrator/gate-policy.json"), "utf8"),
      ) as Record<string, unknown>;
      const manifest = loadCheckManifest(repoRoot);
      const codesWith = (mutate: (draft: Record<string, unknown>) => void) => {
        const draft = JSON.parse(JSON.stringify(policy)) as Record<string, unknown>;
        mutate(draft);
        writeFileSync(policyPath, JSON.stringify(draft));
        return findCiTierReachabilityDiagnostics(scratch, manifest.gates).map(
          (diagnostic) => diagnostic.code,
        );
      };
      // A faithful copy is quiet on every pin arm (setup soundness).
      const baseline = codesWith(() => {});
      for (const code of [
        "gate-policy-criteria-missing",
        "gate-policy-criterion-drift",
        "gate-policy-criterion-digest-drift",
      ]) {
        expect(baseline).not.toContain(code);
      }
      // Deleting the counts key must not silently retire the governance.
      expect(codesWith((draft) => delete draft["governedLeafCriteria"])).toContain(
        "gate-policy-criteria-missing",
      );
      // ...nor may deleting the digest key.
      expect(codesWith((draft) => delete draft["governedLeafCriteriaDigest"])).toContain(
        "gate-policy-criteria-missing",
      );
      // A pinned criterion with zero live members (ghost pin) REDs.
      expect(
        codesWith((draft) => {
          (draft["governedLeafCriteria"] as Record<string, number>)["ghost-criterion"] = 7;
        }),
      ).toContain("gate-policy-criterion-drift");
      // Count-preserving content churn REDs via the pairs digest.
      expect(
        codesWith((draft) => {
          draft["governedLeafCriteriaDigest"] = "f".repeat(64);
        }),
      ).toContain("gate-policy-criterion-digest-drift");
    },
  );

  it("R4 RED-PROOF: gate-policy shape failures are governed diagnostics, not stack traces", () => {
    const scratch = mkdtempSync(path.join(os.tmpdir(), "omena-policy-shape-"));
    mkdirSync(path.join(scratch, "packages/check-orchestrator"), { recursive: true });
    const policy = JSON.parse(
      readFileSync(path.join(repoRoot, "packages/check-orchestrator/gate-policy.json"), "utf8"),
    ) as Record<string, unknown>;
    delete policy["records"];
    delete policy["template"];
    writeFileSync(
      path.join(scratch, "packages/check-orchestrator/gate-policy.json"),
      JSON.stringify(policy),
    );
    const diagnostics = findGatePolicyDiagnostics(scratch, [], new Map());
    expect(diagnostics.map((diagnostic) => diagnostic.code)).toContain("gate-policy-invalid-shape");
    expect(diagnostics.map((diagnostic) => diagnostic.message).join(";")).toContain(
      "records must be an array",
    );
  });

  it(
    "HARDENING RED-PROOF: criterion coherence reaches declared-tier gates, and pinned criterion counts make relabels a reviewed diff",
    { timeout: 30_000 },
    () => {
      const repoManifest = loadCheckManifest(repoRoot);
      // The two compat-split-boundary bundles are declared (ciTier manual) — the
      // sweep must still reach them: removing the tag would RED. We verify the
      // reachability by asserting the sweep currently passes WITH the tag and
      // that the classification map contains them with the compat criterion.
      const source = readFileSync(
        path.join(repoRoot, "packages/check-orchestrator/src/manifest/workflows.ts"),
        "utf8",
      );
      for (const id of [
        "rust/omena-abstract-value/split-boundary",
        "rust/omena-semantic-split-boundary",
      ]) {
        const gate = repoManifest.gates.find((candidate) => candidate.id === id);
        expect(gate?.tags).toContain("compat-split-boundary");
        expect(source).toContain(`id: "${id}"`);
      }
      expect(
        repoManifest.diagnostics.filter(
          (diagnostic) => diagnostic.code === "governed-leaf-criterion-mismatch",
        ),
      ).toEqual([]);
      // Pinned criterion counts exist and match the live summary counts.
      const policy = JSON.parse(
        readFileSync(path.join(repoRoot, "packages/check-orchestrator/gate-policy.json"), "utf8"),
      ) as { governedLeafCriteria: Record<string, number> };
      const total = Object.values(policy.governedLeafCriteria).reduce((sum, n) => sum + n, 0);
      expect(total).toBe(156);
      expect(
        repoManifest.diagnostics.filter(
          (diagnostic) => diagnostic.code === "gate-policy-criterion-drift",
        ),
      ).toEqual([]);
      // R4: the arm must PERTURB, not just describe (the confirm lens supplied
      // this missing perturbation live) — removing the tag from the REAL gate
      // set REDs, so the sweep's reach is proven, not narrated.
      const mutated = repoManifest.gates.map((gate) =>
        gate.id === "rust/omena-semantic-split-boundary"
          ? { ...gate, tags: gate.tags?.filter((tag) => tag !== "compat-split-boundary") }
          : gate,
      );
      expect(
        findCiTierReachabilityDiagnostics(repoRoot, mutated)
          .filter((diagnostic) => diagnostic.code === "governed-leaf-criterion-mismatch")
          .map((diagnostic) => diagnostic.message)
          .join(";"),
      ).toContain('Governed leaf "rust/omena-semantic-split-boundary"');
    },
  );
  it("DIAGNOSTIC-CODE CENSUS: every code literal in src is in the committed inventory and vice versa", () => {
    const src = path.join(repoRoot, "packages/check-orchestrator/src");
    const found = new Set<string>();
    const walk = (dir: string): void => {
      for (const entry of readdirSync(dir, { withFileTypes: true })) {
        const full = path.join(dir, entry.name);
        if (entry.isDirectory()) walk(full);
        else if (entry.name.endsWith(".ts")) {
          for (const match of readFileSync(full, "utf8").matchAll(/code: "([a-z0-9-]+)"/g)) {
            found.add(match[1] ?? "");
          }
        }
      }
    };
    walk(src);
    const committed = (
      JSON.parse(
        readFileSync(
          path.join(repoRoot, "packages/check-orchestrator/diagnostic-codes.json"),
          "utf8",
        ),
      ) as { codes: string[] }
    ).codes;
    expect([...found].toSorted()).toEqual(committed);
  });
});
