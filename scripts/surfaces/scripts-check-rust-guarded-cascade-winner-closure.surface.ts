import { migratedScanSurface } from "../../packages/check-orchestrator/src/evidence/scan-surface";

export default migratedScanSurface({
  scannerPath: "scripts/check-rust-guarded-cascade-winner-closure.ts",
  mode: "index",
  pathspecs: ["**"],
  includeUntracked: false,
  excludes: ["personal-docs"],
});
