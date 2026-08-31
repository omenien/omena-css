import { unmigratedScanSurface } from "../../packages/check-orchestrator/src/evidence/scan-surface";

export default unmigratedScanSurface({
  scannerPath: "scripts/rename-selector.ts",
  reason: "external-checkout",
  evidenceNeedle: "--root",
});
