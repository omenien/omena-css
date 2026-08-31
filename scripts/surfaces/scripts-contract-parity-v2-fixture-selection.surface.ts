import { migratedScanSurface } from "../../packages/check-orchestrator/src/evidence/scan-surface";

export default migratedScanSurface({
  scannerPath: "scripts/contract-parity-v2-fixture-selection.ts",
  mode: "workingTree",
  pathspecs: ["test/_fixtures/contract-parity-v2/**"],
  includeUntracked: false,
  excludes: ["git-metadata", "personal-docs", "node-modules", "rust-build-output"],
});
