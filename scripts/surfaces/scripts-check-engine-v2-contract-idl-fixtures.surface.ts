import { unmigratedScanSurface } from "../../packages/check-orchestrator/src/evidence/scan-surface";

export default unmigratedScanSurface({
  scannerPath: "scripts/check-engine-v2-contract-idl-fixtures.ts",
  reason: "non-repo-temp-tree",
  evidenceNeedle: "mkdtempSync",
});
