import { strict as assert } from "node:assert";
import { execFileSync } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import os from "node:os";
import path from "node:path";

const repoRoot = process.cwd();
for (const credential of ["NPM_TOKEN", "NODE_AUTH_TOKEN"] as const) {
  assert.equal(
    process.env[credential],
    undefined,
    `${credential} must not enter npm pack rehearsal`,
  );
}
const tempDir = mkdtempSync(path.join(os.tmpdir(), "omena-npm-pack-rehearsal-"));
try {
  for (const gate of [
    "core/build/omena-wasm",
    "core/build/omena-napi",
    "release/check/packaged-omena-napi-crossplatform",
  ]) {
    execFileSync("pnpm", ["omena-check", "run", gate], { cwd: repoRoot, stdio: "inherit" });
  }
  const pluginRoot = path.join(tempDir, "plugins");
  execFileSync(
    "pnpm",
    ["omena-check", "run", "core/build/omena-plugin-packages", "--", "--out", pluginRoot],
    { cwd: repoRoot, stdio: "inherit" },
  );
  const packageDirs = [
    path.join(repoRoot, "rust/crates/omena-wasm/pkg"),
    path.join(repoRoot, "rust/crates/omena-napi/pkg"),
    ...["css-build-adapter", "vite-plugin", "postcss-plugin"].map((name) =>
      path.join(pluginRoot, name),
    ),
  ];
  for (const packageDir of packageDirs) {
    const report = JSON.parse(
      execFileSync("npm", ["pack", "--dry-run", "--json"], { cwd: packageDir, encoding: "utf8" }),
    ) as readonly {
      readonly name?: string;
      readonly version?: string;
      readonly files?: readonly unknown[];
    }[];
    assert.equal(report.length, 1, `${packageDir} must produce one pack report`);
    assert.ok(report[0]?.name && report[0]?.version && (report[0]?.files?.length ?? 0) > 0);
  }
  process.stdout.write(
    `${JSON.stringify(
      {
        schemaVersion: "0",
        product: "release.rehearsal.npm-pack",
        packageCount: packageDirs.length,
        uploadAttempted: false,
      },
      null,
      2,
    )}\n`,
  );
} finally {
  rmSync(tempDir, { recursive: true, force: true });
}
