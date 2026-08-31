import { unmigratedScanSurface } from "../../packages/check-orchestrator/src/evidence/scan-surface";

export default unmigratedScanSurface({
  scannerPath: "scripts/check-rust-omena-cli-bundle-origin-chain.ts",
  reason: "non-repo-temp-tree",
  evidenceNeedle: "mkdtempSync",
});
