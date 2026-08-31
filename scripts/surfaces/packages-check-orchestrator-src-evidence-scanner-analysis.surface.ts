import { migratedScanSurface } from "../../packages/check-orchestrator/src/evidence/scan-surface";

export default migratedScanSurface({
  scannerPath: "packages/check-orchestrator/src/evidence/scanner-analysis.ts",
  mode: "index",
  pathspecs: ["scripts/**", "packages/check-orchestrator/src/**"],
  includeUntracked: true,
  excludes: [],
});
