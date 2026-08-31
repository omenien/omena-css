import { migratedScanSurface } from "../../packages/check-orchestrator/src/evidence/scan-surface";

export default migratedScanSurface({
  scannerPath: "scripts/check-rust-omena-style-resolution-authority.ts",
  mode: "index",
  pathspecs: ["rust/**"],
  includeUntracked: false,
  excludes: ["personal-docs"],
});
