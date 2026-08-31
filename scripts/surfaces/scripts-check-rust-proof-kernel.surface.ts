import { migratedScanSurface } from "../../packages/check-orchestrator/src/evidence/scan-surface";

export default migratedScanSurface({
  scannerPath: "scripts/check-rust-proof-kernel.ts",
  mode: "index",
  pathspecs: [":(glob)rust/crates/**/*.rs"],
  includeUntracked: false,
  excludes: ["personal-docs"],
});
