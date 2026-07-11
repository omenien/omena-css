import { spawnSync } from "node:child_process";

const PROFILES = new Set([
  "orchestrator",
  "rust-cli",
  "cross-platform-cli",
  "rust-workspace",
  "closure-diff",
  "linux-benchmark",
  "verify",
]);
const SCRATCH_REF = "codex/ci-probe";

const args = process.argv.slice(2);
const profile = args.find((arg) => !arg.startsWith("-"));
const watch = args.includes("--watch");

if (!profile || !PROFILES.has(profile)) {
  console.error(
    `Usage: pnpm ci:probe -- <profile> [--watch]\nProfiles: ${[...PROFILES].join(", ")}`,
  );
  process.exit(1);
}

const headSha = run("git", ["rev-parse", "HEAD"], { capture: true }).trim();
const dirty = run("git", ["status", "--porcelain"], { capture: true }).trim();
if (dirty) {
  console.warn("Working-tree changes are not included; the probe runs the committed HEAD only.");
}

run("git", ["push", "--force-with-lease", "origin", `HEAD:refs/heads/${SCRATCH_REF}`]);
run("gh", ["workflow", "run", "ci-probe.yml", "--ref", SCRATCH_REF, "-f", `profile=${profile}`]);

console.log(`Triggered ${profile} at ${headSha.slice(0, 12)} on ${SCRATCH_REF}.`);
console.log(`Inspect later with: gh run list --workflow ci-probe.yml --branch ${SCRATCH_REF}`);

if (watch) {
  const runId = waitForRun(headSha);
  run("gh", ["run", "watch", runId, "--exit-status"]);
}

function waitForRun(expectedSha) {
  for (let attempt = 0; attempt < 20; attempt += 1) {
    const output = run(
      "gh",
      [
        "run",
        "list",
        "--workflow",
        "ci-probe.yml",
        "--branch",
        SCRATCH_REF,
        "--event",
        "workflow_dispatch",
        "--limit",
        "10",
        "--json",
        "databaseId,headSha",
      ],
      { capture: true },
    );
    const match = JSON.parse(output).find((entry) => entry.headSha === expectedSha);
    if (match) return String(match.databaseId);
    Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 1_000);
  }
  console.error("Timed out while locating the dispatched workflow run.");
  process.exit(1);
}

function run(command, commandArgs, options = {}) {
  const result = spawnSync(command, commandArgs, {
    encoding: "utf8",
    shell: false,
    stdio: options.capture ? "pipe" : "inherit",
  });
  if (result.error) {
    console.error(`Failed to start ${command}: ${result.error.message}`);
    process.exit(1);
  }
  if ((result.status ?? 1) !== 0) {
    if (options.capture) process.stderr.write(result.stderr ?? "");
    process.exit(result.status ?? 1);
  }
  return result.stdout ?? "";
}
