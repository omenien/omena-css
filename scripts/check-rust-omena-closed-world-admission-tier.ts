import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";

const repositoryRoot = process.cwd();
const injectRecordedAbsenceLoss = process.argv.includes(
  "--inject-recorded-module-reachability-absence-loss",
);
const testName = injectRecordedAbsenceLoss
  ? "injected_missing_module_reachability_absence_test"
  : "tests::empty_semantic_reachability_records_module_input_absence_without_narrowing";
const run = spawnSync(
  "cargo",
  ["test", "-p", "omena-bundler", testName, "--", "--exact"],
  {
    cwd: `${repositoryRoot}/rust`,
    encoding: "utf8",
  },
);
const transcript = `${run.stdout ?? ""}\n${run.stderr ?? ""}`;

// FALSIFIER: id=closed-world-admission-reachability-absence class=liveness via=--inject-recorded-module-reachability-absence-loss producer=can-fail owner=closed-world-admission-tier entry=silent-full-module-retention
assert.equal(
  run.status === 0 &&
    transcript.includes(
      "test tests::empty_semantic_reachability_records_module_input_absence_without_narrowing ... ok",
    ),
  true,
  `module reachability absence was not recorded by the closed-world bundle:\n${transcript}`,
);

process.stdout.write(
  "Omena closed-world admission tier S1 evidence OK: moduleReachabilityInputAbsent recorded\n",
);
