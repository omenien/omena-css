import { spawnSync } from "node:child_process";
import { strict as assert } from "node:assert";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { banGateArgv } from "./lib/rust-write-authority";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const { cargoArgs, scope } = banGateArgv(repoRoot);
const result = spawnSync("cargo", cargoArgs, {
  cwd: repoRoot,
  encoding: "utf8",
  maxBuffer: 128 * 1024 * 1024,
});
assert.equal(result.error, undefined, result.error?.message ?? "cargo clippy spawn failed");
if (result.stdout) process.stdout.write(result.stdout);
if (result.stderr) process.stderr.write(result.stderr);
if (result.status !== 0) process.exit(result.status ?? 1);
process.stdout.write(
  `${JSON.stringify({
    schemaVersion: "0",
    product: "rust.fs-capability-ban",
    scopePackageCount: scope.scope.length,
    refusalPackages: scope.refusals,
    cargoArgv: ["cargo", ...cargoArgs],
  })}\n`,
);
