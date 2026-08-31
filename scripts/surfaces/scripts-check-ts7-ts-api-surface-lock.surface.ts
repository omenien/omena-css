import { migratedScanSurface } from "../../packages/check-orchestrator/src/evidence/scan-surface";

export default migratedScanSurface({
  scannerPath: "scripts/check-ts7-ts-api-surface-lock.ts",
  mode: "workingTree",
  pathspecs: ["server/**", "packages/**", "scripts/**"],
  includeUntracked: false,
  excludes: ["git-metadata", "personal-docs", "node-modules", "rust-build-output"],
});
