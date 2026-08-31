import { migratedScanSurface } from "../../packages/check-orchestrator/src/evidence/scan-surface";

export default migratedScanSurface({
  scannerPath: "scripts/check-rust-omena-sif-shard-trust-census.ts",
  mode: "index",
  pathspecs: ["*.rs"],
  includeUntracked: false,
  excludes: ["personal-docs"],
});
