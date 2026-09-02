import { migratedScanSurface } from "../../packages/check-orchestrator/src/evidence/scan-surface";

export default migratedScanSurface({
  scannerPath: "scripts/check-release-print-sensitive-evidence.ts",
  mode: "index",
  pathspecs: ["**"],
  includeUntracked: true,
  excludes: ["personal-docs"],
});
