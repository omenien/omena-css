import { spawnSync } from "node:child_process";
import { strict as assert } from "node:assert";

interface TransformContextSummaryV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly targetStylePath: string;
  readonly styleCount: number;
  readonly importInlineCount: number;
  readonly classNameRewriteCount: number;
  readonly cssModuleComposesResolutionCount: number;
  readonly reachableClassNameCount: number;
  readonly reachableKeyframeNameCount: number;
  readonly reachableValueNameCount: number;
  readonly reachableCustomPropertyNameCount: number;
  readonly context: {
    readonly closedStyleWorld: boolean;
    readonly reachableClassNames: readonly string[];
    readonly reachableKeyframeNames: readonly string[];
    readonly reachableValueNames: readonly string[];
    readonly reachableCustomPropertyNames: readonly string[];
    readonly importInlines: readonly {
      readonly importSource: string;
      readonly replacementCss: string;
    }[];
    readonly cssModuleComposesResolutions: readonly {
      readonly localClassName: string;
      readonly exportedClassNames: readonly string[];
    }[];
    readonly classNameRewrites: readonly {
      readonly originalName: string;
      readonly rewrittenName: string;
    }[];
    readonly designTokenRoutes: readonly {
      readonly tokenName: string;
      readonly routedValue: string;
    }[];
  };
  readonly readySurfaces: readonly string[];
}

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
    "transform-context",
  ],
  {
    cwd: process.cwd(),
    encoding: "utf8",
    input: JSON.stringify({
      targetStylePath: "src/Button.module.css",
      styles: [
        {
          stylePath: "src/Button.module.css",
          styleSource:
            '@import "./tokens.css"; .button { composes: base utility; color: var(--brand); } .base { color: blue; } .button-primary { color: red; }',
        },
        {
          stylePath: "src/tokens.css",
          styleSource: ":root { --brand: red; }",
        },
      ],
    }),
    maxBuffer: 8 * 1024 * 1024,
  },
);

assert.equal(result.status, 0, result.stderr);
assert.equal(result.error, undefined);

const summary = JSON.parse(result.stdout) as TransformContextSummaryV0;

assert.equal(summary.schemaVersion, "0");
assert.equal(summary.product, "omena-query.transform-context");
assert.equal(summary.targetStylePath, "src/Button.module.css");
assert.equal(summary.styleCount, 2);
assert.equal(summary.importInlineCount, 1);
assert.equal(summary.classNameRewriteCount, 3);
assert.equal(summary.cssModuleComposesResolutionCount, 1);
assert.equal(summary.reachableClassNameCount, 3);
assert.equal(summary.reachableKeyframeNameCount, 0);
assert.equal(summary.reachableValueNameCount, 0);
assert.equal(summary.reachableCustomPropertyNameCount, 1);
assert.equal(summary.context.closedStyleWorld, false);
assert.deepEqual(summary.context.reachableClassNames, ["base", "button", "button-primary"]);
assert.deepEqual(summary.context.reachableKeyframeNames, []);
assert.deepEqual(summary.context.reachableValueNames, []);
assert.deepEqual(summary.context.reachableCustomPropertyNames, ["--brand"]);
assert.deepEqual(summary.context.importInlines, [
  {
    importSource: "./tokens.css",
    replacementCss: ":root { --brand: red; }",
  },
]);
assert.deepEqual(summary.context.cssModuleComposesResolutions, [
  {
    localClassName: "button",
    exportedClassNames: ["base", "button", "utility"],
  },
]);
assertIncludesAll(
  summary.context.classNameRewrites.map((rewrite) => rewrite.originalName),
  ["button", "base", "button-primary"],
  "class rewrite originals",
);
assertIncludesAll(
  summary.context.classNameRewrites.map((rewrite) => rewrite.rewrittenName),
  ["_button_0", "_base_1", "_button-primary_2"],
  "class rewrite outputs",
);
assert.deepEqual(summary.context.designTokenRoutes, []);
assertIncludesAll(
  summary.readySurfaces,
  [
    "transformContextProducer",
    "reachableNameSeedProducer",
    "cssModuleClassRewriteProducer",
    "cssModuleComposesResolutionProducer",
    "directImportInlineProducer",
  ],
  "transform context ready surfaces",
);

process.stdout.write(
  [
    "validated omena-query transform-context runtime:",
    `styles=${summary.styleCount}`,
    `imports=${summary.importInlineCount}`,
    `rewrites=${summary.classNameRewriteCount}`,
    `composes=${summary.cssModuleComposesResolutionCount}`,
    `reachableClasses=${summary.reachableClassNameCount}`,
  ].join(" "),
);
process.stdout.write("\n");

function assertIncludesAll(actual: readonly string[], expected: readonly string[], label: string) {
  for (const value of expected) {
    assert.ok(actual.includes(value), `${label} must include ${value}`);
  }
}
