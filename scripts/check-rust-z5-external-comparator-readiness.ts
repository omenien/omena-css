import { spawnSync } from "node:child_process";
import { strict as assert } from "node:assert";
import { transform as lightningTransform } from "lightningcss";
import { parse as postcssParse, type Root } from "postcss";
import scssSyntax from "postcss-scss";

type StyleDialect = "css" | "scss" | "sass" | "less";

interface StyleCorpusSnapshotV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-benchmarks.style-corpus-snapshot";
  readonly benchmarkFamily: string;
  readonly corpusSampleCount: number;
  readonly samples: readonly StyleCorpusSampleSnapshotV0[];
}

interface StyleCorpusSampleSnapshotV0 {
  readonly name: string;
  readonly path: string;
  readonly dialect: StyleDialect;
  readonly byteLength: number;
  readonly lineCount: number;
  readonly source: string;
}

interface ComparatorResultV0 {
  readonly comparator: "lightningcss" | "postcss";
  readonly supportedDialects: readonly StyleDialect[];
  readonly parsedSampleCount: number;
  readonly parsedSampleNames: readonly string[];
  readonly unsupportedSampleNames: readonly string[];
}

const snapshot = runStyleCorpusSnapshot();

assert.equal(snapshot.schemaVersion, "0");
assert.equal(snapshot.product, "omena-benchmarks.style-corpus-snapshot");
assert.equal(snapshot.benchmarkFamily, "z5-performance-baseline");
assert.equal(snapshot.samples.length, snapshot.corpusSampleCount);
assert.ok(snapshot.corpusSampleCount >= 4, "Z5 corpus should keep M4 corpus expansion");
assert.ok(
  snapshot.samples.some((sample) => sample.name === "css-sizing-width-corpus"),
  "Z5 corpus should reflect M4 css-sizing WPT/spec coverage",
);
assert.ok(
  snapshot.samples.some((sample) => sample.name === "css-backgrounds-longhand-corpus"),
  "Z5 corpus should reflect M4 css-backgrounds WPT/spec coverage",
);

const lightningResult = validateComparator("lightningcss", ["css"], parseWithLightningCss);
const postcssResult = validateComparator("postcss", ["css", "scss"], parseWithPostcss);
const results = [lightningResult, postcssResult] as const;

assert.ok(
  lightningResult.parsedSampleCount >= 3,
  "lightningcss comparator must parse the CSS samples from the Z5 corpus",
);
assert.ok(
  postcssResult.parsedSampleCount >= 5,
  "postcss comparator must parse the CSS/SCSS subset of the Z5 corpus",
);
assert.ok(
  lightningResult.parsedSampleNames.includes("css-backgrounds-longhand-corpus"),
  "lightningcss comparator must parse the M4 css-backgrounds benchmark sample",
);
assert.ok(
  postcssResult.parsedSampleNames.includes("css-backgrounds-longhand-corpus"),
  "postcss comparator must parse the M4 css-backgrounds benchmark sample",
);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "omena-benchmarks.external-comparator-readiness",
      benchmarkFamily: snapshot.benchmarkFamily,
      sameCorpusSource: true,
      corpusSampleCount: snapshot.corpusSampleCount,
      comparatorCount: results.length,
      comparators: results,
      timingPolicy: "no-cross-tool-speed-claim-without-full-timing-run",
    },
    null,
    2,
  )}\n`,
);

function validateComparator(
  comparator: ComparatorResultV0["comparator"],
  supportedDialects: readonly StyleDialect[],
  parse: (sample: StyleCorpusSampleSnapshotV0) => void,
): ComparatorResultV0 {
  const parsedSampleNames: string[] = [];
  const unsupportedSampleNames: string[] = [];

  for (const sample of snapshot.samples) {
    if (!supportedDialects.includes(sample.dialect)) {
      unsupportedSampleNames.push(sample.name);
      continue;
    }
    assert.ok(sample.byteLength > 0, `${sample.name} must expose source bytes`);
    parse(sample);
    parsedSampleNames.push(sample.name);
  }

  return {
    comparator,
    supportedDialects,
    parsedSampleCount: parsedSampleNames.length,
    parsedSampleNames,
    unsupportedSampleNames,
  };
}

function parseWithLightningCss(sample: StyleCorpusSampleSnapshotV0): void {
  lightningTransform({
    filename: sample.path,
    code: Buffer.from(sample.source),
  });
}

function parseWithPostcss(sample: StyleCorpusSampleSnapshotV0): void {
  const root =
    sample.dialect === "scss"
      ? ((scssSyntax as { parse: typeof postcssParse }).parse(sample.source, {
          from: sample.path,
        }) as Root)
      : postcssParse(sample.source, { from: sample.path });
  assert.ok(root.nodes.length > 0, `${sample.name} should parse to a non-empty PostCSS tree`);
}

function runStyleCorpusSnapshot(): StyleCorpusSnapshotV0 {
  const result = spawnSync(
    "cargo",
    [
      "run",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-benchmarks",
      "--bin",
      "z5_style_corpus_snapshot",
      "--quiet",
    ],
    {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
  assert.equal(result.status, 0, result.stderr);
  return JSON.parse(result.stdout) as StyleCorpusSnapshotV0;
}
