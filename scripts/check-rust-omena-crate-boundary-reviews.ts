import { strict as assert } from "node:assert";
import { execFileSync, spawnSync } from "node:child_process";
import { readdirSync, readFileSync, statSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

type Decision = "promote" | "retain" | "revisit";

interface MeasurementBase {
  readonly command: string;
  readonly response: string;
}

interface BoundaryReview {
  readonly id: string;
  readonly subject: {
    readonly kind: "rustCrate" | "operationalCli";
    readonly name: string;
    readonly sourceRoots: readonly string[];
    readonly measurementPaths: readonly string[];
  };
  readonly decision: Decision;
  readonly decisionRationale: string;
  readonly revisitWhen: readonly string[];
  readonly measurements: {
    readonly apiSurfaceStability: MeasurementBase & {
      readonly value: number;
      readonly unit: "commits";
    };
    readonly dependencyDirection: MeasurementBase & {
      readonly directDependencies: readonly string[];
      readonly directConsumers: readonly string[];
      readonly cycleCount: number;
    };
    readonly buildCost: MeasurementBase & {
      readonly warmSeconds: readonly number[];
      readonly medianWarmSeconds: number;
    };
    readonly consumerCount: MeasurementBase & {
      readonly value: number;
      readonly unit: "directWorkspaceConsumers" | "sourceConsumers";
      readonly sourceLines: number;
    };
  };
}

interface BoundaryReviewManifest {
  readonly schemaVersion: "0";
  readonly product: "omena.product-surface-boundary-reviews";
  readonly measurementBase: string;
  readonly ciEnvelope: {
    readonly runId: number;
    readonly rustWorkspaceSeconds: number;
    readonly closureDiffSeconds: number;
    readonly benchmarkSeconds: number;
    readonly conclusion: "success";
  };
  readonly criteria: readonly string[];
  readonly reviews: BoundaryReview[];
}

interface CargoMetadata {
  readonly packages: readonly {
    readonly name: string;
    readonly dependencies: readonly { readonly name: string }[];
  }[];
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const manifestPath = path.join(repoRoot, "rust/product-surface-boundary-reviews.json");
const manifest = JSON.parse(readFileSync(manifestPath, "utf8")) as BoundaryReviewManifest;

if (process.argv.includes("--measure")) {
  process.stdout.write(`${JSON.stringify(measureBuildCommands(), null, 2)}\n`);
  process.exit(0);
}

if (process.env.OMENA_BOUNDARY_REVIEW_TEST_REMOVE_MEASUREMENT === "1") {
  Reflect.deleteProperty(manifest.reviews[0]?.measurements ?? {}, "buildCost");
}
if (process.env.OMENA_BOUNDARY_REVIEW_TEST_CLEAR_RESPONSE === "1") {
  const first = manifest.reviews[0];
  if (first) {
    (first.measurements.consumerCount as { response: string }).response = "";
  }
}

assert.equal(manifest.schemaVersion, "0");
assert.equal(manifest.product, "omena.product-surface-boundary-reviews");
assert.deepEqual(manifest.criteria, [
  "apiSurfaceStability",
  "dependencyDirection",
  "buildCost",
  "consumerCount",
]);
assert.match(manifest.measurementBase, /^[0-9a-f]{9,40}$/u);
assert.ok(manifest.ciEnvelope.runId > 0);
assert.equal(manifest.ciEnvelope.conclusion, "success");
for (const seconds of [
  manifest.ciEnvelope.rustWorkspaceSeconds,
  manifest.ciEnvelope.closureDiffSeconds,
  manifest.ciEnvelope.benchmarkSeconds,
]) {
  assert.ok(seconds > 0, "CI envelope durations must be measured positive values");
}

assert.deepEqual(
  manifest.reviews.map(({ id }) => id),
  ["css-bundler-boundary", "scss-evaluator-boundary", "checker-cli-boundary"],
);

const metadata = cargoMetadata();
const packageNames = new Set(metadata.packages.map(({ name }) => name));

for (const review of manifest.reviews) {
  validateReviewContract(review);
  validateSourceMeasurements(review);
  validateApiChurn(review, manifest.measurementBase);
  if (review.subject.kind === "rustCrate") {
    validateCargoBoundary(review, metadata, packageNames);
  } else {
    validateCheckerCliBoundary(review);
  }
}

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.omena-crate-boundary-reviews",
      criteria: manifest.criteria.length,
      reviews: manifest.reviews.length,
      decisions: Object.fromEntries(manifest.reviews.map(({ id, decision }) => [id, decision])),
    },
    null,
    2,
  )}\n`,
);

function validateReviewContract(review: BoundaryReview): void {
  assert.ok(["promote", "retain", "revisit"].includes(review.decision));
  assert.ok(review.decisionRationale.trim().length >= 80);
  assert.ok(
    review.revisitWhen.length > 0,
    `${review.id} must declare measurable re-review conditions`,
  );

  for (const criterion of manifest.criteria) {
    const measurement = review.measurements[criterion as keyof BoundaryReview["measurements"]];
    assert.ok(measurement, `${review.id} is missing ${criterion}`);
    assert.ok(measurement.command.trim().length > 0, `${review.id}.${criterion} needs a command`);
    assert.ok(
      measurement.response.trim().length >= 40,
      `${review.id}.${criterion} must respond to the measured result`,
    );
  }

  const samples = review.measurements.buildCost.warmSeconds;
  assert.equal(samples.length, 3, `${review.id} must record three warm samples`);
  assert.ok(samples.every((sample) => Number.isFinite(sample) && sample > 0));
  const median = [...samples].sort((left, right) => left - right)[1];
  assert.equal(review.measurements.buildCost.medianWarmSeconds, median);
  assert.equal(review.measurements.dependencyDirection.cycleCount, 0);
}

function validateSourceMeasurements(review: BoundaryReview): void {
  const lines = review.subject.sourceRoots.reduce(
    (total, root) => total + countSourceLines(path.join(repoRoot, root)),
    0,
  );
  assert.equal(
    review.measurements.consumerCount.sourceLines,
    lines,
    `${review.id} source line measurement drifted`,
  );
}

function validateApiChurn(review: BoundaryReview, base: string): void {
  const output = execFileSync(
    "git",
    ["rev-list", "--count", `${base}..HEAD`, "--", ...review.subject.measurementPaths],
    { cwd: repoRoot, encoding: "utf8" },
  ).trim();
  assert.equal(
    review.measurements.apiSurfaceStability.value,
    Number.parseInt(output, 10),
    `${review.id} API churn measurement drifted`,
  );
}

function validateCargoBoundary(
  review: BoundaryReview,
  metadata: CargoMetadata,
  packageNames: ReadonlySet<string>,
): void {
  const subject = metadata.packages.find(({ name }) => name === review.subject.name);
  assert.ok(subject, `missing workspace crate ${review.subject.name}`);
  const directDependencies = subject.dependencies
    .map(({ name }) => name)
    .filter((name) => packageNames.has(name))
    .toSorted();
  const directConsumers = metadata.packages
    .filter(({ dependencies }) => dependencies.some(({ name }) => name === review.subject.name))
    .map(({ name }) => name)
    .toSorted();
  assert.deepEqual(review.measurements.dependencyDirection.directDependencies, directDependencies);
  assert.deepEqual(review.measurements.dependencyDirection.directConsumers, directConsumers);
  assert.equal(review.measurements.consumerCount.value, directConsumers.length);
  assert.ok(
    !directDependencies.includes("omena-query"),
    `${review.subject.name} must not depend back on the product facade`,
  );
}

function validateCheckerCliBoundary(review: BoundaryReview): void {
  const consumers = checkerCliConsumers();
  assert.deepEqual(review.measurements.dependencyDirection.directConsumers, consumers);
  assert.equal(review.measurements.consumerCount.value, consumers.length);
}

function cargoMetadata(): CargoMetadata {
  return JSON.parse(
    execFileSync(
      "cargo",
      ["metadata", "--manifest-path", "rust/Cargo.toml", "--no-deps", "--format-version", "1"],
      { cwd: repoRoot, encoding: "utf8" },
    ),
  ) as CargoMetadata;
}

function checkerCliConsumers(): readonly string[] {
  const output = execFileSync(
    "rg",
    ["-l", "server/checker-cli/src", "scripts", "test", "packages", "server"],
    { cwd: repoRoot, encoding: "utf8" },
  );
  return output
    .trim()
    .split("\n")
    .filter(Boolean)
    .filter((entry) => entry !== "scripts/check-rust-omena-crate-boundary-reviews.ts")
    .toSorted();
}

function countSourceLines(root: string): number {
  if (statSync(root).isFile()) return lineCount(root);
  let count = 0;
  for (const entry of readdirSync(root, { withFileTypes: true })) {
    const candidate = path.join(root, entry.name);
    if (entry.isDirectory()) {
      count += countSourceLines(candidate);
    } else if (/\.(?:rs|ts)$/u.test(entry.name)) {
      count += lineCount(candidate);
    }
  }
  return count;
}

function lineCount(filePath: string): number {
  const source = readFileSync(filePath, "utf8");
  return source.length === 0 ? 0 : source.split("\n").length - Number(source.endsWith("\n"));
}

function measureBuildCommands(): Record<string, readonly number[]> {
  return {
    "omena-bundler": measure([
      "cargo",
      "check",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-bundler",
      "--quiet",
    ]),
    "omena-scss-eval": measure([
      "cargo",
      "check",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-scss-eval",
      "--quiet",
    ]),
    "checker-cli": measure(["node", "--import", "tsx", "./scripts/check-workspace.ts", "--help"]),
  };
}

function measure(command: readonly string[]): readonly number[] {
  const samples: number[] = [];
  for (let index = 0; index < 3; index += 1) {
    const startedAt = performance.now();
    const result = spawnSync(command[0], command.slice(1), {
      cwd: repoRoot,
      encoding: "utf8",
      stdio: "ignore",
    });
    assert.equal(result.status, 0, `${command.join(" ")} must remain executable`);
    samples.push(Number(((performance.now() - startedAt) / 1000).toFixed(2)));
  }
  return samples;
}
