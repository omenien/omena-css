#!/usr/bin/env node
import { resolveScanSurfaceForScanner } from "../packages/check-orchestrator/src/evidence/scan-surface-manifest.ts";
import { chmodSync, existsSync, statSync } from "node:fs";
import path from "node:path";

const evidenceScanSurface = resolveScanSurfaceForScanner(import.meta.url);

const repoRoot = path.resolve(import.meta.dirname, "..");
const binRoot = path.join(repoRoot, "dist", "bin");
const executableNames = new Set(["engine-shadow-runner", "omena-lsp-server", "tsgo"]);

let restored = 0;

if (existsSync(binRoot)) {
  for (const targetDir of evidenceScanSurface.readdirSync(binRoot, { withFileTypes: true })) {
    if (!targetDir.isDirectory()) continue;
    if (targetDir.name.startsWith("win32-")) continue;

    const targetPath = path.join(binRoot, targetDir.name);
    for (const entry of evidenceScanSurface.readdirSync(targetPath, { withFileTypes: true })) {
      if (!entry.isFile() || !executableNames.has(entry.name)) continue;
      const binaryPath = path.join(targetPath, entry.name);
      const mode = statSync(binaryPath).mode;
      if ((mode & 0o111) === 0o111) continue;
      chmodSync(binaryPath, 0o755);
      restored += 1;
    }
  }
}

console.log(`Native binary executable permissions restored: ${restored}`);
