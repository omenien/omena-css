import { migratedScanSurface } from "../../packages/check-orchestrator/src/evidence/scan-surface";

export default migratedScanSurface({
  scannerPath: "scripts/check-docs-version-governance.ts",
  mode: "workingTree",
  pathspecs: ["packages/**"],
  includeUntracked: false,
  excludes: ["git-metadata", "personal-docs", "node-modules", "rust-build-output"],
});
