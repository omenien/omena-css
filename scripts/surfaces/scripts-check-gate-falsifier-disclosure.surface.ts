import { migratedScanSurface } from "../../packages/check-orchestrator/src/evidence/scan-surface";

export default migratedScanSurface({
  scannerPath: "scripts/check-gate-falsifier-disclosure.ts",
  mode: "index",
  pathspecs: ["rust/**"],
  includeUntracked: false,
  excludes: ["personal-docs"],
});
