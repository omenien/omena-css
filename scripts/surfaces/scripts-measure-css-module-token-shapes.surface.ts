import { unmigratedScanSurface } from "../../packages/check-orchestrator/src/evidence/scan-surface";

export default unmigratedScanSurface({
  scannerPath: "scripts/measure-css-module-token-shapes.ts",
  reason: "external-checkout",
  evidenceNeedle: "args.corpusRoot",
});
