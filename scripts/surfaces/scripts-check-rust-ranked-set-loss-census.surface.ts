import { migratedScanSurface } from "../../packages/check-orchestrator/src/evidence/scan-surface";

export default migratedScanSurface({
  scannerPath: "scripts/check-rust-ranked-set-loss-census.ts",
  mode: "index",
  pathspecs: ["**"],
  includeUntracked: true,
  excludes: ["personal-docs"],
});
