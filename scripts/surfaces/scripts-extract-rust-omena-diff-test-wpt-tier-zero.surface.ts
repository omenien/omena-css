import { unmigratedScanSurface } from "../../packages/check-orchestrator/src/evidence/scan-surface";

export default unmigratedScanSurface({
  scannerPath: "scripts/extract-rust-omena-diff-test-wpt-tier-zero.ts",
  reason: "external-checkout",
});
