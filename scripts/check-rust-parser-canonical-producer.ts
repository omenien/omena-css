import { spawn } from "node:child_process";
import { strict as assert } from "node:assert";

import type { ParserSassSeedFactsV0 } from "./rust-parser-sass-facts";

interface ParserParityLiteSummaryV0 {
  readonly schemaVersion: "0";
  readonly language: "css" | "scss" | "less";
}

interface ParserIndexSummaryV0 {
  readonly schemaVersion: "0";
  readonly language: "css" | "scss" | "less";
  readonly sass: ParserSassSeedFactsV0;
}

interface ParserCanonicalCandidateBundleV0 {
  readonly schemaVersion: "0";
  readonly language: "css" | "scss" | "less";
  readonly parityLite: ParserParityLiteSummaryV0;
  readonly cssModulesIntermediate: ParserIndexSummaryV0;
}

interface ParserEvaluatorCandidateV0 {
  readonly kind: "selector-index-facts";
  readonly selectorName: string;
}

interface ParserEvaluatorCandidatesV0 {
  readonly schemaVersion: "0";
  readonly language: "css" | "scss" | "less";
  readonly results: readonly ParserEvaluatorCandidateV0[];
}

interface ParserCanonicalProducerSignalV0 {
  readonly schemaVersion: "0";
  readonly language: "css" | "scss" | "less";
  readonly canonicalCandidate: ParserCanonicalCandidateBundleV0;
  readonly evaluatorCandidates: ParserEvaluatorCandidatesV0;
  readonly publicProductGate: {
    readonly canonicalCandidateCommand: "pnpm check:rust-parser-canonical-candidate";
    readonly consumerBoundaryCommand: "pnpm check:rust-parser-consumer-boundary";
    readonly publicProductGateCommand: "pnpm check:rust-parser-public-product";
    readonly includedInParserLane: true;
    readonly includedInRustLaneBundle: true;
    readonly includedInRustReleaseBundle: true;
  };
}

const CORPUS = [
  {
    label: "css-basic-parser-canonical-producer",
    filePath: "/f.module.css",
    source: `.btn { color: red; }`,
  },
  {
    label: "less-basic-parser-canonical-producer",
    filePath: "/f.module.less",
    source: `.btn { color: red; }`,
  },
  {
    label: "scss-media-keyframes-parser-canonical-producer",
    filePath: "/f.module.scss",
    source: `@media (min-width: 1px) { @keyframes pulse { from { opacity: 0; } } .btn { animation: pulse 1s linear; } }`,
  },
  {
    label: "scss-mixed-value-refs-parser-canonical-producer",
    filePath: "/f.module.scss",
    source: `@supports (display: grid) { @layer ui { @value brand from "./tokens.module.scss"; @value accent: red; .btn { color: brand; background: accent; } } }`,
  },
  {
    label: "scss-mixed-composes-parser-canonical-producer",
    filePath: "/f.module.scss",
    source: `@supports (display: grid) { @layer ui { .card { composes: base utility; composes: shell from global; composes: tone from "./base.module.scss"; } } }`,
  },
  {
    label: "scss-sass-symbol-parser-canonical-producer",
    filePath: "/f.module.scss",
    source: `@use "./plain";\n@use "./reset" as *;\n@use "./tokens" as tokens;\n@use "sass:color";\n@forward "./theme";\n@import "./legacy";\n$gap: 1rem;\n@mixin raised($depth) { box-shadow: 0 0 $depth black; }\n@function tone($value) { @return $value; }\n.btn { color: $gap; @include raised($gap); border-color: tone($gap); }\n.ghost { color: $missing; @include absent($gap); }`,
  },
] as const;

async function runRustJson<T>(bin: string, filePath: string, source: string): Promise<T> {
  return new Promise((resolve, reject) => {
    const child = spawn(
      "cargo",
      [
        "run",
        "--quiet",
        "--manifest-path",
        "rust/Cargo.toml",
        "-p",
        "omena-parser",
        "--bin",
        bin,
        "--",
        filePath,
      ],
      {
        cwd: process.cwd(),
        stdio: ["pipe", "pipe", "pipe"],
      },
    );

    let stdout = "";
    let stderr = "";
    child.stdout.on("data", (chunk) => {
      stdout += String(chunk);
    });
    child.stderr.on("data", (chunk) => {
      stderr += String(chunk);
    });
    child.on("error", reject);
    child.on("close", (code) => {
      if (code !== 0) {
        reject(new Error(`${bin} exited with ${code}\n${stderr}`));
        return;
      }
      resolve(JSON.parse(stdout) as T);
    });

    child.stdin.end(source);
  });
}

void (async () => {
  for (const entry of CORPUS) {
    process.stdout.write(`== rust-parser-canonical-producer:${entry.label} ==\n`);

    // oxlint-disable-next-line eslint/no-await-in-loop
    const canonicalCandidate = await runRustJson<ParserCanonicalCandidateBundleV0>(
      "omena-parser-canonical-candidate",
      entry.filePath,
      entry.source,
    );
    // oxlint-disable-next-line eslint/no-await-in-loop
    const evaluatorCandidates = await runRustJson<ParserEvaluatorCandidatesV0>(
      "omena-parser-evaluator-candidates",
      entry.filePath,
      entry.source,
    );
    // oxlint-disable-next-line eslint/no-await-in-loop
    const actual = await runRustJson<ParserCanonicalProducerSignalV0>(
      "omena-parser-canonical-producer",
      entry.filePath,
      entry.source,
    );

    const expected: ParserCanonicalProducerSignalV0 = {
      schemaVersion: "0",
      language: canonicalCandidate.language,
      canonicalCandidate,
      evaluatorCandidates,
      publicProductGate: {
        canonicalCandidateCommand: "pnpm check:rust-parser-canonical-candidate",
        consumerBoundaryCommand: "pnpm check:rust-parser-consumer-boundary",
        publicProductGateCommand: "pnpm check:rust-parser-public-product",
        includedInParserLane: true,
        includedInRustLaneBundle: true,
        includedInRustReleaseBundle: true,
      },
    };

    assert.deepEqual(
      actual,
      expected,
      [
        `parser canonical-producer mismatch for ${entry.label}`,
        `expected: ${JSON.stringify(expected, null, 2)}`,
        `actual: ${JSON.stringify(actual, null, 2)}`,
      ].join("\n"),
    );

    process.stdout.write(
      `validated parser canonical-producer: language=${actual.language} lane=${actual.publicProductGate.includedInParserLane} sassVars=${actual.canonicalCandidate.cssModulesIntermediate.sass.variableDeclNames.length}\n\n`,
    );
  }
})().catch((error: unknown) => {
  console.error(error);
  process.exit(1);
});
