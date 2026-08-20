// g131-S2: the re-homed shell postcondition. The old ci.yml step
// `public-surface:update && git diff --exit-code -- …/public-api.txt` was the
// ONLY consumer that compared the writer's output to the committed snapshot
// at RAW BYTES — the plain public-surface member normalizes BOTH sides
// (sorted, trimmed, io-path-rewritten), so normalization-masked drift in the
// committed file passes it. This gate keeps that byte postcondition alive as
// a real, shard-able gate: writer output (via out-override, tree untouched)
// must equal the committed snapshot byte-for-byte. No normalization.
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const snapshotPath = path.join(
  repoRoot,
  "rust/crates/omena-reactive/tests/snapshots/public-api.txt",
);

const scratch = mkdtempSync(path.join(os.tmpdir(), "omena-public-surface-clean-"));
const outPath = path.join(scratch, "public-api.txt");
try {
  const writer = spawnSync(
    "node",
    ["--import", "tsx", "./scripts/check-rust-omena-reactive-public-surface.ts", "--write"],
    {
      cwd: repoRoot,
      encoding: "utf8",
      stdio: "inherit",
      env: {
        ...process.env,
        OMENA_REACTIVE_PUBLIC_SURFACE_OUT: outPath,
        OMENA_REACTIVE_PUBLIC_SURFACE_SKIP_SEMVER: "1",
      },
    },
  );
  if (writer.status !== 0) {
    throw new Error(`public-surface writer failed with status ${writer.status}`);
  }

  const emitted = readFileSync(outPath);
  const committed = readFileSync(snapshotPath);
  if (!emitted.equals(committed)) {
    const emittedLines = emitted.toString("utf8").split("\n");
    const committedLines = committed.toString("utf8").split("\n");
    let index = 0;
    while (
      index < Math.max(emittedLines.length, committedLines.length) &&
      emittedLines[index] === committedLines[index]
    ) {
      index += 1;
    }
    throw new Error(
      "omena-reactive committed public-api snapshot differs from the writer's output at raw " +
        `bytes (line ${index + 1}: committed=${JSON.stringify(committedLines[index] ?? "<eof>")} ` +
        `emitted=${JSON.stringify(emittedLines[index] ?? "<eof>")}). ` +
        "Run `pnpm run update:rust-omena-reactive-public-surface` and review the diff — " +
        "normalization-masked drift in the committed file is exactly what this gate exists to catch.",
    );
  }

  process.stdout.write(
    `${JSON.stringify({
      schemaVersion: "0",
      product: "rust.omena-reactive.public-surface-clean",
      comparison: "raw-bytes",
      snapshot: path.relative(repoRoot, snapshotPath),
      bytes: committed.length,
    })}\n`,
  );
} finally {
  rmSync(scratch, { recursive: true, force: true });
}
