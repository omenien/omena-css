import { execFileSync } from "node:child_process";
import process from "node:process";

const expectedSha = process.env.OMENA_PROVENANCE_SOURCE_SHA ?? process.env.GITHUB_SHA;

if (!expectedSha || !/^[0-9a-f]{40}$/u.test(expectedSha)) {
  console.error(
    "publish provenance source check requires a full OMENA_PROVENANCE_SOURCE_SHA or GITHUB_SHA",
  );
  process.exit(1);
}

const checkedOutSha = execFileSync("git", ["rev-parse", "HEAD"], {
  encoding: "utf8",
}).trim();

if (checkedOutSha !== expectedSha) {
  console.error(
    [
      "publish provenance source mismatch",
      `workflow provenance source: ${expectedSha}`,
      `checked-out publication source: ${checkedOutSha}`,
      "Dispatch the workflow from the same immutable ref that is being checked out.",
    ].join("\n"),
  );
  process.exit(1);
}

console.log(`publish provenance source verified: ${checkedOutSha}`);
