import { migratedScanSurface } from "../../packages/check-orchestrator/src/evidence/scan-surface";

export default migratedScanSurface({
  scannerPath: "scripts/check-rust-omena-resolver-identity-index-callers.ts",
  mode: "index",
  pathspecs: ["rust/**", "tools/**"],
  includeUntracked: false,
  excludes: ["personal-docs"],
});
