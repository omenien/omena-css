import { unmigratedScanSurface } from "../../packages/check-orchestrator/src/evidence/scan-surface";

export default unmigratedScanSurface({
  scannerPath: "scripts/check-dual-pin-product-bytes.ts",
  reason: "non-repo-temp-tree",
});
