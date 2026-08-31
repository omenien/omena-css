import { migratedScanSurface } from "../../packages/check-orchestrator/src/evidence/scan-surface";

export default migratedScanSurface({
  scannerPath: "scripts/prepare-tsgo-binary.mjs",
  mode: "workingTree",
  pathspecs: ["node_modules/**"],
  includeUntracked: false,
  excludes: ["git-metadata", "personal-docs", "rust-build-output"],
});
