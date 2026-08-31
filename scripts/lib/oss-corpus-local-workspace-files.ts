import { statSync } from "node:fs";
import path from "node:path";
import { resolveScanSurfaceForScanner } from "../../packages/check-orchestrator/src/evidence/scan-surface-manifest";

const SKIPPED_DIRECTORY_NAMES = new Set([
  ".cache",
  ".git",
  "node_modules",
  "target",
  "dist",
  "out",
]);

export function listLocalWorkspaceCorpusFiles(repoRoot: string, workspaceRoot: string): string[] {
  const surface = resolveScanSurfaceForScanner(import.meta.url, repoRoot);
  const files: string[] = [];
  const visit = (directory: string): void => {
    for (const entry of surface.readdirSync(directory, { withFileTypes: true })) {
      if (SKIPPED_DIRECTORY_NAMES.has(entry.name)) continue;
      const entryPath = path.join(directory, entry.name);
      if (entry.isDirectory()) {
        visit(entryPath);
      } else if (/\.(?:css|scss|sass|less|jsx?|tsx?|json|toml)$/u.test(entry.name)) {
        files.push(entryPath);
      }
    }
  };
  if (statSync(workspaceRoot).isDirectory()) visit(workspaceRoot);
  return files.sort((left, right) => left.localeCompare(right, "en"));
}
