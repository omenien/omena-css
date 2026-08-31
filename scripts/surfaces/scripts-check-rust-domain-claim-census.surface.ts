import { migratedScanSurface } from "../../packages/check-orchestrator/src/evidence/scan-surface";

export default migratedScanSurface({
  scannerPath: "scripts/check-rust-domain-claim-census.ts",
  mode: "index",
  pathspecs: ["**"],
  includeUntracked: true,
  excludes: ["personal-docs"],
});
