// The product-test classguard's CI-structure assertions, re-anchored from
// raw ci.yml text scans to the governed job registry (g131-S0). The registry
// is authoritative: ci.yml is byte-generated from it and drift-gated, so a
// structure read here is a read of what CI actually runs.
//
// STRUCTURE-PROOF requirement (g131 REV4, final-confirm N1): these checks
// must survive the exact restructures g131-S3 will perform — deleting the
// aggregator-only `rust-product-tests` job and rewiring needs — while still
// going RED on the mutations that matter (a deleted product-test job, a
// needs graph that no longer carries a product-test result to ci-required,
// a dropped matrix leg). The aggregation invariant is therefore expressed
// over the TRANSITIVE needs closure of `ci-required`, not over any named
// intermediate aggregator.

export interface CiStructureJobView {
  readonly name: string;
  readonly block: readonly string[];
  readonly needs: readonly string[];
}

export interface ProductTestCiStructureExpectation {
  readonly crateShardIds: readonly string[];
  readonly contractShardNames: readonly string[];
}

const CRATE_JOB = "rust-product-test-crates";
const CONTRACT_JOB = "rust-product-test-contracts";
const PRODUCT_TEST_JOBS = [CRATE_JOB, CONTRACT_JOB] as const;
// The duty is satisfied only by an EXECUTING run step — a comment naming
// the invocation is not a step (the same fail-open species the g130 judge
// rules closed; stage-5 lens reproduced the comment evasion end-to-end).
const CLASSGUARD_LINE = /^\s*- run: .*pnpm omena-check run rust\/product-test-coverage-classguard/u;

function parseInlineMatrix(blockLines: readonly string[], key: string): readonly string[] | null {
  const escapedKey = key.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
  const pattern = new RegExp(`^\\s+${escapedKey}:\\s*\\[([^\\]]+)\\]\\s*$`, "u");
  for (const line of blockLines) {
    const values = pattern.exec(line)?.[1];
    if (values) {
      return values
        .split(",")
        .map((value) => value.trim().replace(/^["']|["']$/gu, ""))
        .filter(Boolean);
    }
  }
  return null;
}

function ciRequiredNeedsClosure(jobs: readonly CiStructureJobView[]): Set<string> {
  const needsByName = new Map(jobs.map((job) => [job.name, job.needs]));
  const closure = new Set<string>();
  const queue = [...(needsByName.get("ci-required") ?? [])];
  while (queue.length > 0) {
    const name = queue.pop();
    if (!name || closure.has(name)) continue;
    closure.add(name);
    queue.push(...(needsByName.get(name) ?? []));
  }
  return closure;
}

export function findProductTestCiStructureErrors(
  jobs: readonly CiStructureJobView[],
  expected: ProductTestCiStructureExpectation,
): string[] {
  const errors: string[] = [];
  const byName = new Map(jobs.map((job) => [job.name, job]));

  // g131-S3: the classguard's HOME is not pinned to a named job (the plan job
  // was legally merged into the crates matrix as a ride-along) — the duty is
  // that SOME job executes it and that job's result reaches ci-required.
  const classguardHomes = jobs.filter((job) =>
    job.block.some((line) => CLASSGUARD_LINE.test(line)),
  );
  if (classguardHomes.length === 0) {
    errors.push(
      "no CI job executes the product-test classguard; the coverage contract is unenforced",
    );
  }

  const crateJob = byName.get(CRATE_JOB);
  if (!crateJob) {
    errors.push(`registry must declare job "${CRATE_JOB}" (the Cargo product-test matrix)`);
  } else {
    const matrix = parseInlineMatrix(crateJob.block, "product-shard");
    if (!matrix) {
      errors.push(`job "${CRATE_JOB}" must declare an explicit inline product-shard matrix`);
    } else if (
      JSON.stringify([...matrix].toSorted()) !==
      JSON.stringify([...expected.crateShardIds].toSorted())
    ) {
      errors.push(
        `job "${CRATE_JOB}" matrix [${[...matrix].toSorted().join(", ")}] must equal the declared ` +
          `product-test shards [${[...expected.crateShardIds].toSorted().join(", ")}]`,
      );
    }
    if (
      !crateJob.block.some((line) =>
        // The canonical invocation, in the g131-S0 summary form: the flag
        // sits before the `--` slice so parseArgs keeps it CLI-level.
        /pnpm omena-check run rust\/product-test-execution --summary -- \$\{\{ matrix\.product-shard \}\}/u.test(
          line,
        ),
      )
    ) {
      errors.push(`job "${CRATE_JOB}" must pass its shard through the canonical product-test gate`);
    }
  }

  const contractJob = byName.get(CONTRACT_JOB);
  if (!contractJob) {
    errors.push(`registry must declare job "${CONTRACT_JOB}" (the contract matrix)`);
  } else {
    const matrix = parseInlineMatrix(contractJob.block, "contract-shard");
    if (!matrix) {
      errors.push(`job "${CONTRACT_JOB}" must declare an explicit inline contract-shard matrix`);
    } else if (
      JSON.stringify([...matrix].toSorted()) !==
      JSON.stringify([...expected.contractShardNames].toSorted())
    ) {
      errors.push(
        `job "${CONTRACT_JOB}" matrix [${[...matrix].toSorted().join(", ")}] must equal the ` +
          `orchestrator shard table [${[...expected.contractShardNames].toSorted().join(", ")}]`,
      );
    }
    if (
      !contractJob.block.some((line) =>
        /pnpm omena-check bundle rust\/product-test-contracts --summary --shard=\$\{\{ matrix\.contract-shard \}\}/u.test(
          line,
        ),
      )
    ) {
      errors.push(`job "${CONTRACT_JOB}" must use the orchestrator shard table`);
    }
    if (
      !contractJob.block.some((line) =>
        /taiki-e\/install-action@7f4eb899022d8fe70b20c4f3de697aa85c309026/u.test(line),
      )
    ) {
      errors.push(`job "${CONTRACT_JOB}" must retain the pinned prebuilt tool installer`);
    }
    if (
      !contractJob.block.some((line) =>
        /key: rust-api-tools-\$\{\{ runner\.os \}\}-\$\{\{ runner\.arch \}\}-public-api-0\.52\.0-semver-checks-0\.48\.0/u.test(
          line,
        ),
      )
    ) {
      errors.push(`job "${CONTRACT_JOB}" must cache its versioned Rust API tools`);
    }
  }

  // Aggregation, structure-proof: every product-test job's RESULT must reach
  // ci-required through the needs graph — through any intermediate shape or
  // none at all. (GitHub's skip-cascade makes every needs ancestor
  // failure-propagating; the judge rules in the orchestrator govern the
  // if:/judge semantics of whatever aggregator carries them.)
  const closure = ciRequiredNeedsClosure(jobs);
  for (const jobName of PRODUCT_TEST_JOBS) {
    if (!byName.has(jobName)) continue; // already reported above
    if (!closure.has(jobName)) {
      errors.push(
        `job "${jobName}" no longer reaches ci-required through the needs graph; ` +
          "a product-test failure could stop blocking the merge",
      );
    }
  }
  for (const home of classguardHomes) {
    if (!closure.has(home.name)) {
      errors.push(
        `job "${home.name}" executes the classguard but does not reach ci-required; ` +
          "a coverage violation could stop blocking the merge",
      );
    }
  }

  return errors;
}
