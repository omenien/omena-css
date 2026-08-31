import { migratedScanSurface } from "../../packages/check-orchestrator/src/evidence/scan-surface";

export default migratedScanSurface({
  scannerPath: "scripts/check-backend-typecheck-smoke.ts",
  mode: "workingTree",
  pathspecs: ["test/_fixtures/backend-typecheck-smoke/**"],
  includeUntracked: false,
  excludes: ["git-metadata", "personal-docs", "node-modules", "rust-build-output"],
});
