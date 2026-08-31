import { migratedScanSurface } from "../../packages/check-orchestrator/src/evidence/scan-surface";

export default migratedScanSurface({
  scannerPath: "scripts/check-rust-omena-syntax-authority-raw-scan-census.ts",
  mode: "index",
  pathspecs: ["**"],
  includeUntracked: false,
  excludes: ["personal-docs"],
});
