import { strict as assert } from "node:assert";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { assertPinnedDartSassVersion, runPinnedDartSass } from "./lib/dart-sass-cli";

const repoRoot = process.cwd();
const entryArgument = process.argv[2];
assert.ok(entryArgument, "a Sass entry path is required");
const entry = path.resolve(entryArgument);
const compilerVersion = assertPinnedDartSassVersion(repoRoot).split(" ")[0];
const outputRoot = mkdtempSync(path.join(tmpdir(), "omena-sass-compile-"));
const outputPath = path.join(outputRoot, "compiled.css");

try {
  const result = runPinnedDartSass(
    ["--no-source-map", "--style", "expanded", entry, outputPath],
    repoRoot,
  );
  const exitStatus = result.status ?? -1;
  process.stdout.write(
    `${JSON.stringify(
      {
        schemaVersion: "0",
        product: "omena-cli.sass-compile-bridge-result",
        compiler: { name: "dart-sass", package: "sass", version: compilerVersion },
        entry,
        exitStatus,
        ...(exitStatus === 0 ? { css: readFileSync(outputPath, "utf8") } : {}),
        stderr: result.stderr,
      },
      null,
      2,
    )}\n`,
  );
} finally {
  rmSync(outputRoot, { force: true, recursive: true });
}
