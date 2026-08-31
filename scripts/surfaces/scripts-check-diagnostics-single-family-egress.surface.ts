import { migratedScanSurface } from "../../packages/check-orchestrator/src/evidence/scan-surface";

export default migratedScanSurface({
  scannerPath: "scripts/check-diagnostics-single-family-egress.ts",
  mode: "workingTree",
  pathspecs: ["server/**", "packages/**", "rust/**", "scripts/**"],
  includeUntracked: false,
  excludes: ["git-metadata", "personal-docs", "node-modules", "rust-build-output"],
});
