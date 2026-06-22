import { spawnSync } from "node:child_process";
import { strict as assert } from "node:assert";

interface NativeCssEvaluatorSummaryV0 {
  readonly product: string;
  readonly dialect: string;
  readonly supportedDialect: boolean;
  readonly nativeStaticEvalSpecSnapshot: string;
  readonly nativeStaticEvalOptInPolicy: string;
  readonly nativeStaticEvalDialectRestriction: string;
  readonly nativeStaticEvalExplicitOptInRequired: boolean;
  readonly nativeFunctionCallFoldableCount: number;
  readonly nativeFunctionCallStructuralErrorCount: number;
  readonly nativeStaticEditCount: number;
  readonly nativeStaticEditOutputChanged: boolean;
  readonly ifFunctionFoldableCount: number;
  readonly ifFunctionPreservedCount: number;
  readonly nativeStaticEditPlan?: {
    readonly editedCss: string;
    readonly editCount: number;
    readonly whenRuleEditCount: number;
    readonly ifFunctionEditCount: number;
    readonly functionCallEditCount: number;
  } | null;
}

interface TransformExecuteSummaryV0 {
  readonly product: string;
  readonly unknownPassIds: readonly string[];
  readonly execution: {
    readonly outputCss: string;
    readonly executedPassIds: readonly string[];
    readonly mutationCount: number;
  };
}

interface ConsumerBuildSummaryV0 {
  readonly product: string;
  readonly execution: {
    readonly outputCss: string;
    readonly executedPassIds: readonly string[];
    readonly mutationCount: number;
  };
}

function runRunner<T>(command: string, input: unknown): T {
  const result = spawnSync(
    "cargo",
    [
      "run",
      "--quiet",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "engine-shadow-runner",
      "--",
      command,
    ],
    {
      cwd: process.cwd(),
      encoding: "utf8",
      input: JSON.stringify(input),
      maxBuffer: 1024 * 1024 * 16,
    },
  );

  assert.equal(
    result.status,
    0,
    `${command} failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return JSON.parse(result.stdout) as T;
}

function engineInput(stylePath: string, styleSource: string) {
  return {
    version: "native-css-routing-check-v0",
    sources: [],
    styles: [
      {
        filePath: stylePath,
        source: styleSource,
        document: {
          selectors: [],
        },
      },
    ],
    typeFacts: [],
  };
}

const nativeSource =
  "@function --inner() returns <length> { result: 1px; } " +
  "@function --outer() returns <length> { result: --inner(); } " +
  ".card { width: --inner(); display: if(supports(display: grid): grid; else: block); " +
  "margin: if(media(width >= 1px): 1rem; else: 2rem); } " +
  "@when supports(display: grid) { .grid { display: if(supports(display: grid): grid; else: block); } } " +
  "@else { .fallback { display: block; } }";

const nativeEvaluator = runRunner<NativeCssEvaluatorSummaryV0>("input-native-css-evaluator", {
  targetStylePath: "/tmp/Native.module.css",
  engineInput: engineInput("/tmp/Native.module.css", nativeSource),
});

assert.equal(nativeEvaluator.product, "omena-query.native-css-evaluator");
assert.equal(nativeEvaluator.dialect, "css");
assert.equal(nativeEvaluator.supportedDialect, true);
assert.equal(
  nativeEvaluator.nativeStaticEvalSpecSnapshot,
  "css-values-5-if-css-mixins-1-function-ed-2026-06-22",
);
assert.equal(
  nativeEvaluator.nativeStaticEvalOptInPolicy,
  "explicit-pass-id-required-default-consumer-build-excludes",
);
assert.equal(nativeEvaluator.nativeStaticEvalDialectRestriction, "css-only");
assert.equal(nativeEvaluator.nativeStaticEvalExplicitOptInRequired, true);
assert.equal(nativeEvaluator.nativeFunctionCallFoldableCount, 2);
assert.equal(nativeEvaluator.nativeFunctionCallStructuralErrorCount, 0);
assert.equal(nativeEvaluator.ifFunctionFoldableCount, 2);
assert.equal(nativeEvaluator.ifFunctionPreservedCount, 1);
assert.equal(nativeEvaluator.nativeStaticEditOutputChanged, true);
assert.equal(nativeEvaluator.nativeStaticEditPlan?.whenRuleEditCount, 1);
assert.equal(nativeEvaluator.nativeStaticEditPlan?.ifFunctionEditCount, 1);
assert.equal(nativeEvaluator.nativeStaticEditPlan?.functionCallEditCount, 1);
assert.ok(
  nativeEvaluator.nativeStaticEditPlan?.editedCss.includes("result: --inner();"),
  "native function declaration bodies must stay preserved until scope/tree-shake analysis is stronger",
);
assert.ok(
  nativeEvaluator.nativeStaticEditPlan?.editedCss.includes(".grid { display: grid; }"),
  "nested static edits inside a folded @when body must compose into the replacement",
);
assert.ok(
  nativeEvaluator.nativeStaticEditPlan?.editedCss.includes(
    "margin: if(media(width >= 1px): 1rem; else: 2rem)",
  ),
  "runtime media-dependent inline if() must remain verbatim",
);

const cycleEvaluator = runRunner<NativeCssEvaluatorSummaryV0>("input-native-css-evaluator", {
  targetStylePath: "/tmp/Cycle.module.css",
  engineInput: engineInput(
    "/tmp/Cycle.module.css",
    "@function --loop() returns <length> { result: --loop(); } .card { width: --loop(); }",
  ),
});

assert.equal(cycleEvaluator.nativeFunctionCallFoldableCount, 0);
assert.ok(
  cycleEvaluator.nativeFunctionCallStructuralErrorCount >= 1,
  "guaranteed native CSS function cycles must surface as structural errors",
);
assert.equal(cycleEvaluator.nativeStaticEditCount, 0);
assert.equal(cycleEvaluator.nativeStaticEditOutputChanged, false);

const transformExecution = runRunner<TransformExecuteSummaryV0>("transform-execute", {
  stylePath: "Native.module.css",
  styleSource: nativeSource,
  requestedPassIds: ["native-css-static-eval", "print-css"],
});

assert.equal(transformExecution.product, "omena-query.transform-execute");
assert.deepEqual(transformExecution.unknownPassIds, []);
assert.deepEqual(transformExecution.execution.executedPassIds, [
  "native-css-static-eval",
  "print-css",
]);
assert.equal(transformExecution.execution.mutationCount, 3);
assert.ok(transformExecution.execution.outputCss.includes("width: 1px"));
assert.ok(transformExecution.execution.outputCss.includes("result: --inner();"));
assert.ok(transformExecution.execution.outputCss.includes(".grid { display: grid; }"));
assert.ok(!transformExecution.execution.outputCss.includes(".fallback"));
assert.ok(!transformExecution.execution.outputCss.includes("display: if(supports"));
assert.ok(
  transformExecution.execution.outputCss.includes(
    "margin: if(media(width >= 1px): 1rem; else: 2rem)",
  ),
);

const defaultConsumerBuild = runRunner<ConsumerBuildSummaryV0>("consumer-build-style-source", {
  stylePath: "Native.module.css",
  styleSource: nativeSource,
  requestedPassIds: [],
});

assert.equal(defaultConsumerBuild.product, "omena-query.consumer-build-style-source");
assert.ok(
  !defaultConsumerBuild.execution.executedPassIds.includes("native-css-static-eval"),
  "native CSS static evaluation is an explicit opt-in pass while CSSWG ED behavior is still gated",
);
assert.ok(defaultConsumerBuild.execution.outputCss.includes("if(supports"));

const scssEvaluator = runRunner<NativeCssEvaluatorSummaryV0>("input-native-css-evaluator", {
  targetStylePath: "/tmp/Scss.module.scss",
  engineInput: engineInput("/tmp/Scss.module.scss", ".card { width: if(true, 1px, 2px); }"),
});

assert.equal(scssEvaluator.dialect, "scss");
assert.equal(scssEvaluator.supportedDialect, false);
assert.equal(scssEvaluator.nativeStaticEditCount, 0);
assert.equal(scssEvaluator.ifFunctionFoldableCount, 0);

console.log(
  "validated native CSS routing: opt-in static eval, prune-but-keep runtime preservation, function cycle errors, nested edit composition, and SCSS disjointness",
);
