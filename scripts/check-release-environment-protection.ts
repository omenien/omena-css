import { strict as assert } from "node:assert";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";

interface EnvironmentResponse {
  readonly protection_rules?: readonly unknown[];
}

interface PostureCarrier {
  readonly expectedEnvironmentProtection: boolean;
}

const carrier = JSON.parse(
  readFileSync("docs/releases/release-workflow-job-allowlist.json", "utf8"),
) as PostureCarrier;
const fixture = process.argv.find((arg) => arg.startsWith("--fixture-"));
let tracked = carrier.expectedEnvironmentProtection;
let live: boolean;

switch (fixture) {
  case "--fixture-tracked-configured-live-unconfigured":
    tracked = true;
    live = false;
    break;
  case "--fixture-both-unconfigured":
    tracked = false;
    live = false;
    break;
  case "--fixture-live-configured-tracked-unconfigured":
    tracked = false;
    live = true;
    break;
  case undefined: {
    const repository = process.env.GITHUB_REPOSITORY ?? "omenien/omena-css";
    const response = JSON.parse(
      execFileSync("gh", ["api", `repos/${repository}/environments/release`], {
        encoding: "utf8",
      }),
    ) as EnvironmentResponse;
    assert.ok(
      Array.isArray(response.protection_rules),
      "release environment API omitted protection_rules",
    );
    live = response.protection_rules.length > 0;
    break;
  }
  default:
    throw new Error(`unknown environment-protection fixture ${fixture}`);
}

if (tracked && !live) {
  throw new Error("release environment lost its expected protection rules");
}
const disposition =
  !tracked && !live
    ? "NOTICE: release environment has not yet been configured with protection rules"
    : !tracked && live
      ? "NOTICE: protection is live; flip expectedEnvironmentProtection in the reviewed allowlist"
      : "release environment protection matches the tracked configured posture";

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "release.environment-protection",
      expectedEnvironmentProtection: tracked,
      liveEnvironmentProtection: live,
      disposition,
    },
    null,
    2,
  )}\n`,
);
