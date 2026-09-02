import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import { parse as parseYaml } from "yaml";

import {
  RELEASE_REHEARSAL_ENVIRONMENT_STEP,
  RELEASE_REHEARSAL_ESCALATION_STEP,
  RELEASE_REHEARSAL_ESCALATION_TITLE,
  RELEASE_REHEARSAL_JOB_ID,
  RELEASE_REHEARSAL_JOB_NAME,
  RELEASE_REHEARSAL_PATH_STEPS,
  RELEASE_REHEARSAL_RETIREMENT_STEP,
} from "./lib/release-rehearsal-contract.ts";

type UnknownRecord = Record<string, unknown>;

let source = readFileSync(".github/workflows/release-rehearsal.yml", "utf8");
const mutation = process.argv.find((arg) => arg.startsWith("--inject-"));
if (mutation === "--inject-registry-credential") {
  source = source.replace(
    "    runs-on: ubuntu-latest\n",
    "    runs-on: ubuntu-latest\n    env:\n      NPM_TOKEN: injected\n",
  );
}
if (mutation === "--inject-environment") {
  source = source.replace(
    "    runs-on: ubuntu-latest\n",
    "    runs-on: ubuntu-latest\n    environment: release\n",
  );
}
if (mutation === "--inject-drop-notes-step") {
  source = source.replace(
    "      - name: Render latest published release notes",
    "      - name: Removed notes rehearsal",
  );
}
if (mutation === "--inject-drop-environment-step") {
  source = source.replace(
    `      - name: ${RELEASE_REHEARSAL_ENVIRONMENT_STEP}`,
    "      - name: Removed posture check",
  );
}
if (mutation === "--inject-drop-escalation-step") {
  source = source.replace(
    `      - name: ${RELEASE_REHEARSAL_ESCALATION_STEP}`,
    "      - name: Removed escalation",
  );
}
if (mutation === "--inject-drop-retirement-step") {
  source = source.replace(
    `      - name: ${RELEASE_REHEARSAL_RETIREMENT_STEP}`,
    "      - name: Removed retirement ladder",
  );
}
if (mutation === "--inject-cached-notes") {
  source = source.replace(
    "pnpm omena-check run release/rehearsal/notes",
    "pnpm omena-check run release/release/notes -- export-github --tag cached",
  );
}
if (mutation === "--inject-retitle-escalation") {
  source = source.replace(
    RELEASE_REHEARSAL_ESCALATION_TITLE,
    `${RELEASE_REHEARSAL_ESCALATION_TITLE} renamed`,
  );
}

const workflow = asRecord(parseYaml(source));
const events = asRecord(workflow.on);
assert.deepEqual(Object.keys(events).toSorted(), ["schedule", "workflow_dispatch"]);
assert.ok(
  Array.isArray(events.schedule) && events.schedule.length === 1,
  "rehearsal must run weekly",
);
assert.deepEqual(workflow.permissions, { actions: "read", contents: "read", issues: "write" });
assert.equal(
  asRecord(workflow.env).RELEASE_REHEARSAL_ESCALATION_TITLE,
  RELEASE_REHEARSAL_ESCALATION_TITLE,
);

const jobs = asRecord(workflow.jobs);
assert.deepEqual(Object.keys(jobs), [RELEASE_REHEARSAL_JOB_ID]);
const job = asRecord(jobs[RELEASE_REHEARSAL_JOB_ID]);
assert.equal(job.name, RELEASE_REHEARSAL_JOB_NAME);
assert.equal(job.environment, undefined, "rehearsal must never wait on a publication environment");
assert.equal(job["timeout-minutes"], 120);
const jobSource = JSON.stringify(job);
for (const forbidden of [
  "CRATES_IO_TOKEN",
  "CARGO_REGISTRY_TOKEN",
  "NPM_TOKEN",
  "NODE_AUTH_TOKEN",
  "softprops/action-gh-release",
  "deploy-pages",
  "npm publish",
  "cargo publish",
]) {
  assert.ok(
    !jobSource.includes(forbidden),
    `release rehearsal contains forbidden capability ${forbidden}`,
  );
}

const steps = Array.isArray(job.steps) ? job.steps.map(asRecord) : [];
const stepByName = new Map(
  steps.filter((step) => typeof step.name === "string").map((step) => [step.name as string, step]),
);
const pathGateIds = [
  "release/rehearsal/crate-dry-run",
  "release/rehearsal/notes",
  "release/rehearsal/npm-pack",
  "release/rehearsal/lifecycle",
];
for (const [index, name] of RELEASE_REHEARSAL_PATH_STEPS.entries()) {
  const step = stepByName.get(name);
  assert(step, `rehearsal path step is missing: ${name}`);
  assert.equal(step.run, `pnpm omena-check run ${pathGateIds[index]}`);
  if (index > 0) assert.equal(step.if, "always()", `${name} must run after an earlier failure`);
}
const posture = stepByName.get(RELEASE_REHEARSAL_ENVIRONMENT_STEP);
assert(posture);
assert.equal(posture.run, "pnpm omena-check run release/rehearsal/environment-protection");
assert.equal(posture.if, "always()");
const escalation = stepByName.get(RELEASE_REHEARSAL_ESCALATION_STEP);
assert(escalation);
assert.equal(escalation.if, "failure()");
assert.equal(escalation.uses, "./.github/actions/escalate-ci-failure");
assert.equal(asRecord(escalation.with).title, "${{ env.RELEASE_REHEARSAL_ESCALATION_TITLE }}");
const retirement = stepByName.get(RELEASE_REHEARSAL_RETIREMENT_STEP);
assert(retirement);
assert.equal(retirement.if, "always()");
assert.equal(retirement.run, "pnpm omena-check run release/rehearsal/retirement -- --live");
assert.equal(
  steps.at(-1)?.name,
  RELEASE_REHEARSAL_RETIREMENT_STEP,
  "retirement self-check must be final",
);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "release.rehearsal-workflow",
      rehearsalPathSteps: RELEASE_REHEARSAL_PATH_STEPS,
      permittedPermissions: workflow.permissions,
      timeoutMinutes: job["timeout-minutes"],
      retirementStepIsFinal: true,
    },
    null,
    2,
  )}\n`,
);

function asRecord(value: unknown): UnknownRecord {
  return value !== null && typeof value === "object" && !Array.isArray(value)
    ? (value as UnknownRecord)
    : {};
}
