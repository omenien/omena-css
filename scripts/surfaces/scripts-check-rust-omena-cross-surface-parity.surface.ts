import { migratedScanSurface } from "../../packages/check-orchestrator/src/evidence/scan-surface";

export default migratedScanSurface({
  scannerPath: "scripts/check-rust-omena-cross-surface-parity.ts",
  mode: "workingTree",
  pathspecs: ["test/**", "contracts/**", "packages/**", "rust/**"],
  includeUntracked: false,
  excludes: ["git-metadata", "personal-docs", "node-modules", "rust-build-output"],
});
