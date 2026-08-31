import { migratedScanSurface } from "../../packages/check-orchestrator/src/evidence/scan-surface";

export default migratedScanSurface({
  scannerPath: "scripts/check-rust-frame-refresh-runtime-disposition.ts",
  mode: "index",
  pathspecs: ["rust/**"],
  includeUntracked: false,
  excludes: ["personal-docs"],
});
