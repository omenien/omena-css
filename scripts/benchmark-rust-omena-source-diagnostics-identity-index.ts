import { strict as assert } from "node:assert";
import { spawn, spawnSync } from "node:child_process";
import {
  existsSync,
  mkdtempSync,
  mkdirSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const sourceCount = positiveIntegerArgument("--source-count", 300);
const styleCount = positiveIntegerArgument("--style-count", 120);
const warmupCount = nonNegativeIntegerArgument("--warmups", 1);
const sampleCount = positiveIntegerArgument("--samples", 5);
const profilePath = valueAfter("--sample-profile");
const fixtureClass = fixtureClassArgument();
const omenaBinary = path.resolve(repoRoot, valueAfter("--binary") ?? "rust/target/release/omena");

assert.ok(
  existsSync(omenaBinary),
  `release CLI is missing at ${omenaBinary}; build it with cargo build --manifest-path rust/Cargo.toml --release -p omena-cli --bin omena`,
);
assert.ok(sourceCount >= 200, "source-diagnostics timing corpus must contain hundreds of sources");
assert.ok(styleCount >= 32, "identity-index timing corpus must contain a non-trivial style index");
assertWeightedSampleParser();

void main();

async function main(): Promise<void> {
  const fixtureRoot = mkdtempSync(
    path.join(os.tmpdir(), "omena-source-diagnostics-identity-index-"),
  );
  try {
    writeFixture(fixtureRoot, sourceCount, styleCount, fixtureClass);
    for (let index = 0; index < warmupCount; index += 1) runLint(fixtureRoot);

    const elapsedMilliseconds: number[] = [];
    for (let index = 0; index < sampleCount; index += 1) {
      const started = process.hrtime.bigint();
      runLint(fixtureRoot);
      elapsedMilliseconds.push(Number(process.hrtime.bigint() - started) / 1_000_000);
    }

    const profile =
      profilePath === undefined
        ? undefined
        : await captureSampleProfile(fixtureRoot, path.resolve(repoRoot, profilePath));
    const sorted = elapsedMilliseconds.toSorted((left, right) => left - right);
    const median = sorted[Math.floor(sorted.length / 2)]!;
    const minimum = sorted[0]!;
    const maximum = sorted.at(-1)!;
    const mean = sorted.reduce((sum, value) => sum + value, 0) / sorted.length;
    const noiseBandPercent = median === 0 ? 0 : ((maximum - minimum) / median) * 100;

    process.stdout.write(
      `${JSON.stringify(
        {
          schemaVersion: "0",
          product: "omena-cli.source-diagnostics-identity-index-benchmark",
          harness: {
            command: `${path.relative(repoRoot, omenaBinary)} lint <generated-workspace> --profile recommended --json`,
            fixtureClass,
            sourceCount,
            styleCount,
            warmupCount,
            sampleCount,
            timingQualification: sampleCount === 1 ? "n=1-advisory" : "multi-sample",
            releaseBinary: path.relative(repoRoot, omenaBinary),
            binaryPreparation: "prebuilt-required; harness-does-not-build-the-cli",
          },
          elapsedMilliseconds: elapsedMilliseconds.map(round),
          minimumMilliseconds: round(minimum),
          medianMilliseconds: round(median),
          maximumMilliseconds: round(maximum),
          meanMilliseconds: round(mean),
          noiseBandPercent: round(noiseBandPercent),
          profile,
        },
        null,
        2,
      )}\n`,
    );
  } finally {
    rmSync(fixtureRoot, { recursive: true, force: true });
  }
}

function writeFixture(
  root: string,
  sources: number,
  styles: number,
  fixtureKind: "exact-match" | "symlink-confirmation",
): void {
  const sourceRoot = path.join(root, "src");
  const styleRoot = path.join(sourceRoot, "styles");
  mkdirSync(styleRoot, { recursive: true });
  if (fixtureKind === "symlink-confirmation") {
    symlinkSync(styleRoot, path.join(sourceRoot, "linked-styles"), "dir");
  }
  writeFileSync(
    path.join(root, "package.json"),
    `${JSON.stringify({ name: "omena-source-diagnostics-identity-index-benchmark", private: true })}\n`,
  );
  for (let index = 0; index < styles; index += 1) {
    const suffix = padded(index);
    writeFileSync(
      path.join(styleRoot, `Style${suffix}.module.css`),
      `.item${suffix} { color: rgb(${index % 255} 0 0); }\n`,
    );
  }
  for (let index = 0; index < sources; index += 1) {
    const styleIndex = index % styles;
    const sourceSuffix = padded(index);
    const styleSuffix = padded(styleIndex);
    const importRoot = fixtureKind === "exact-match" ? "styles" : "linked-styles";
    writeFileSync(
      path.join(sourceRoot, `Component${sourceSuffix}.tsx`),
      `import styles from "./${importRoot}/Style${styleSuffix}.module.css";\nexport const Component${sourceSuffix} = () => <div className={styles.item${styleSuffix}} />;\n`,
    );
  }
}

function runLint(root: string): void {
  const result = spawnSync(omenaBinary, ["lint", root, "--profile", "recommended", "--json"], {
    cwd: repoRoot,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
    env: { ...process.env, NO_COLOR: "1" },
  });
  if (result.error) throw result.error;
  assert.equal(
    result.status,
    0,
    `product lint failed: status=${result.status}\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  const envelope = JSON.parse(result.stdout) as {
    readonly product: string;
    readonly payload: { readonly sourceFileCount: number; readonly styleFileCount: number };
  };
  assert.equal(envelope.product, "omena-cli.lint", "benchmark must execute the product lint path");
  assert.equal(envelope.payload.sourceFileCount, sourceCount, "source corpus drifted");
  assert.equal(envelope.payload.styleFileCount, styleCount, "style corpus drifted");
}

async function captureSampleProfile(
  root: string,
  outputPath: string,
): Promise<{
  readonly status: "captured" | "unavailable";
  readonly path: string;
  readonly metric?: "macos-sample-weight-sum";
  readonly captureCount?: 1;
  readonly timingQualification?: "n=1-advisory";
  readonly identityIndexBuilderWeightedSamples?: number;
  readonly canonicalizeIdentityWeightedSamples?: number;
  readonly detail?: string;
}> {
  if (process.platform !== "darwin" || !existsSync("/usr/bin/sample")) {
    return {
      status: "unavailable",
      path: outputPath,
      detail: "the macOS sample profiler is unavailable on this host",
    };
  }
  mkdirSync(path.dirname(outputPath), { recursive: true });
  const child = spawn(omenaBinary, ["lint", root, "--profile", "recommended", "--json"], {
    cwd: repoRoot,
    stdio: ["ignore", "ignore", "pipe"],
    env: { ...process.env, NO_COLOR: "1" },
  });
  assert.ok(child.pid !== undefined, "product lint profiler process must have a pid");
  const exitPromise = new Promise<number | null>((resolve, reject) => {
    child.once("error", reject);
    child.once("exit", resolve);
  });
  await new Promise((resolve) => setTimeout(resolve, 250));
  const sampled = spawnSync(
    "/usr/bin/sample",
    [String(child.pid), "5", "10", "-file", outputPath],
    { cwd: repoRoot, encoding: "utf8" },
  );
  const exit = await exitPromise;
  if (sampled.status !== 0 || exit !== 0 || !existsSync(outputPath)) {
    return {
      status: "unavailable",
      path: outputPath,
      detail: `sample_status=${sampled.status} lint_status=${exit} stderr=${sampled.stderr.trim()}`,
    };
  }
  const profile = readFileSync(outputPath, "utf8");
  return {
    status: "captured",
    path: outputPath,
    metric: "macos-sample-weight-sum",
    captureCount: 1,
    timingQualification: "n=1-advisory",
    identityIndexBuilderWeightedSamples: weightedSampleCount(
      profile,
      "build_omena_resolver_style_module_confirmation_identity_index",
    ),
    canonicalizeIdentityWeightedSamples: weightedSampleCount(
      profile,
      "canonicalize_omena_resolver_style_identity_path",
    ),
  };
}

function positiveIntegerArgument(flag: string, fallback: number): number {
  const value = valueAfter(flag);
  if (value === undefined) return fallback;
  const parsed = Number.parseInt(value, 10);
  assert.ok(Number.isSafeInteger(parsed) && parsed > 0, `${flag} must be a positive integer`);
  return parsed;
}

function nonNegativeIntegerArgument(flag: string, fallback: number): number {
  const value = valueAfter(flag);
  if (value === undefined) return fallback;
  const parsed = Number.parseInt(value, 10);
  assert.ok(Number.isSafeInteger(parsed) && parsed >= 0, `${flag} must be non-negative`);
  return parsed;
}

function valueAfter(flag: string): string | undefined {
  const index = process.argv.indexOf(flag);
  return index < 0 ? undefined : process.argv[index + 1];
}

function fixtureClassArgument(): "exact-match" | "symlink-confirmation" {
  const value = valueAfter("--fixture-class") ?? "exact-match";
  assert.ok(
    value === "exact-match" || value === "symlink-confirmation",
    "--fixture-class must be exact-match or symlink-confirmation",
  );
  return value;
}

function weightedSampleCount(source: string, symbol: string): number {
  let total = 0;
  for (const line of source.split(/\r?\n/u)) {
    const symbolAt = line.indexOf(symbol);
    if (symbolAt < 0) continue;
    const weight = line.slice(0, symbolAt).match(/(?:^|\s)(\d+)\s+\S*$/u)?.[1];
    if (weight !== undefined) total += Number.parseInt(weight, 10);
  }
  return total;
}

function assertWeightedSampleParser(): void {
  const symbol = "canonicalize_omena_resolver_style_identity_path";
  const fixture = [
    `  + ! : | 123 ${symbol}::h1 (in omena)`,
    `  + ! : |   2 ${symbol}::h1 (in omena)`,
    `        8       ${symbol}::h1 (in omena)`,
    `        5       unrelated_symbol::h2 (in omena)`,
  ].join("\n");
  assert.equal(
    weightedSampleCount(fixture, symbol),
    133,
    "sample parser must sum stack weights instead of counting report-text occurrences",
  );
}

function padded(value: number): string {
  return value.toString().padStart(4, "0");
}

function round(value: number): number {
  return Math.round(value * 1000) / 1000;
}
