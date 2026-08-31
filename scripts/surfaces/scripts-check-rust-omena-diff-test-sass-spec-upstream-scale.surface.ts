import { unmigratedScanSurface } from "../../packages/check-orchestrator/src/evidence/scan-surface";

export default unmigratedScanSurface({
  scannerPath: "scripts/check-rust-omena-diff-test-sass-spec-upstream-scale.ts",
  reason: "external-checkout",
  evidenceNeedle: "OMENA_SASS_SPEC_UPSTREAM_ROOT",
});
