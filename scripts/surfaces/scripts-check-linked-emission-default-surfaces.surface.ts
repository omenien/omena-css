import { migratedScanSurface } from "../../packages/check-orchestrator/src/evidence/scan-surface";

export default migratedScanSurface({
  scannerPath: "scripts/check-linked-emission-default-surfaces.ts",
  mode: "index",
  pathspecs: ["packages/**", "rust/**"],
  includeUntracked: false,
  excludes: ["personal-docs"],
});
