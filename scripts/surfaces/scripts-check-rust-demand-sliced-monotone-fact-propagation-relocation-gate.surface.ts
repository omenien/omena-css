import { migratedScanSurface } from "../../packages/check-orchestrator/src/evidence/scan-surface";

export default migratedScanSurface({
  scannerPath: "scripts/check-rust-demand-sliced-monotone-fact-propagation-relocation-gate.ts",
  mode: "workingTree",
  pathspecs: ["rust/**"],
  includeUntracked: false,
  excludes: ["git-metadata", "personal-docs", "node-modules", "rust-build-output"],
});
