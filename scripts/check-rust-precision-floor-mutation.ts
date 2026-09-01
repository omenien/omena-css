import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const authority = JSON.parse(
  fs.readFileSync(path.join(repoRoot, "rust/omena-precision-floor-authority.json"), "utf8"),
) as {
  readonly mutationProbes: readonly {
    readonly id: string;
    readonly sourcePath: string;
    readonly from: string;
    readonly to: string;
    readonly command: readonly string[];
    readonly expectedFailure: string;
  }[];
};

const requestedId = process.argv[2];
assert.ok(requestedId !== undefined, "usage: check-rust-precision-floor-mutation.ts <probe-id>");
const probe = authority.mutationProbes.find((candidate) => candidate.id === requestedId);
assert.ok(probe !== undefined, `unknown precision mutation probe: ${requestedId}`);
assert.ok(probe.command[0] !== undefined, `${probe.id} must declare a command`);

const sourcePath = path.join(repoRoot, probe.sourcePath);
const original = fs.readFileSync(sourcePath, "utf8");
assert.equal(
  original.split(probe.from).length - 1,
  1,
  `${probe.id} source must match exactly once before mutation`,
);
const mutated = original.replace(probe.from, probe.to);
assert.notEqual(mutated, original, `${probe.id} must change the source`);
const mutationPatch = unifiedPatch(probe.sourcePath, original, mutated);

let transcript = "";
let status: number | null = null;
try {
  const apply = spawnSync("git", ["apply", "--whitespace=nowarn", "-"], {
    cwd: repoRoot,
    encoding: "utf8",
    input: mutationPatch,
  });
  assert.equal(apply.status, 0, `${probe.id} mutation apply failed:\n${apply.stderr}`);
  const run = spawnSync(probe.command[0], probe.command.slice(1), {
    cwd: repoRoot,
    encoding: "utf8",
    maxBuffer: 32 * 1024 * 1024,
    env: { ...process.env, OMENA_PRECISION_MUTATION_PROBE: probe.id },
  });
  status = run.status;
  transcript = `${run.stdout ?? ""}\n${run.stderr ?? ""}`;
} finally {
  const restore = spawnSync("git", ["apply", "--reverse", "--whitespace=nowarn", "-"], {
    cwd: repoRoot,
    encoding: "utf8",
    input: mutationPatch,
  });
  assert.equal(restore.status, 0, `${probe.id} mutation restore failed:\n${restore.stderr}`);
}

assert.notEqual(status, 0, `${probe.id} unexpectedly stayed GREEN:\n${transcript}`);
assert.ok(
  transcript.includes(probe.expectedFailure),
  `${probe.id} failed for the wrong reason; expected ${probe.expectedFailure}:\n${transcript}`,
);
assert.equal(fs.readFileSync(sourcePath, "utf8"), original, `${probe.id} source restore failed`);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "1",
      product: "omena-transform.precision-floor-source-mutation",
      probeId: probe.id,
      sourcePath: probe.sourcePath,
      command: probe.command,
      observedExit: status,
      expectedFailure: probe.expectedFailure,
      restored: true,
      red: true,
    },
    null,
    2,
  )}\n`,
);

function unifiedPatch(relativePath: string, before: string, after: string): string {
  const beforeLines = before.split("\n");
  const afterLines = after.split("\n");
  let prefixLength = 0;
  while (
    prefixLength < beforeLines.length &&
    prefixLength < afterLines.length &&
    beforeLines[prefixLength] === afterLines[prefixLength]
  ) {
    prefixLength += 1;
  }
  let suffixLength = 0;
  while (
    suffixLength < beforeLines.length - prefixLength &&
    suffixLength < afterLines.length - prefixLength &&
    beforeLines[beforeLines.length - 1 - suffixLength] ===
      afterLines[afterLines.length - 1 - suffixLength]
  ) {
    suffixLength += 1;
  }

  const context = 3;
  const beforeChangeEnd = beforeLines.length - suffixLength;
  const afterChangeEnd = afterLines.length - suffixLength;
  const beforeStart = Math.max(0, prefixLength - context);
  const afterStart = beforeStart;
  const beforeEnd = Math.min(beforeLines.length, beforeChangeEnd + context);
  const afterEnd = Math.min(afterLines.length, afterChangeEnd + context);
  const lines = [
    `diff --git a/${relativePath} b/${relativePath}`,
    `--- a/${relativePath}`,
    `+++ b/${relativePath}`,
    `@@ -${beforeStart + 1},${beforeEnd - beforeStart} +${afterStart + 1},${afterEnd - afterStart} @@`,
    ...beforeLines.slice(beforeStart, prefixLength).map((line) => ` ${line}`),
    ...beforeLines.slice(prefixLength, beforeChangeEnd).map((line) => `-${line}`),
    ...afterLines.slice(prefixLength, afterChangeEnd).map((line) => `+${line}`),
    ...beforeLines.slice(beforeChangeEnd, beforeEnd).map((line) => ` ${line}`),
  ];
  return `${lines.join("\n")}\n`;
}
