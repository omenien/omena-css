#!/usr/bin/env node
import { resolveScanSurfaceForScanner } from "../packages/check-orchestrator/src/evidence/scan-surface-manifest.ts";
import { existsSync, mkdirSync } from "node:fs";
import { execFileSync } from "node:child_process";
import path from "node:path";

const evidenceScanSurface = resolveScanSurfaceForScanner(import.meta.url);

const repoRoot = path.resolve(import.meta.dirname, "..");
const artifactsRoot = path.join(repoRoot, ".runner-artifacts");
const outputRoot = path.join(repoRoot, "dist", "bin");

if (!existsSync(artifactsRoot)) {
  console.log("No .runner-artifacts directory found; skipping runner artifact merge.");
  process.exit(0);
}

const archives = findArchives(artifactsRoot);
if (archives.length === 0) {
  console.log("No engine-shadow-runner artifact archives found; skipping runner artifact merge.");
  process.exit(0);
}

mkdirSync(outputRoot, { recursive: true });
for (const archive of archives) {
  execFileSync("tar", ["-xzf", archive, "-C", outputRoot], {
    cwd: repoRoot,
    stdio: "inherit",
  });
}

console.log(`Merged ${archives.length} engine-shadow-runner artifact archive(s).`);

function findArchives(root) {
  const results = [];
  for (const entry of evidenceScanSurface.readdirSync(root, { withFileTypes: true })) {
    const entryPath = path.join(root, entry.name);
    if (entry.isDirectory()) {
      results.push(...findArchives(entryPath));
      continue;
    }
    if (entry.isFile() && /^engine-shadow-runner-.+\.tgz$/u.test(entry.name)) {
      results.push(entryPath);
    }
  }
  return results.toSorted();
}
