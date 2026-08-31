import { migratedScanSurface } from "../../packages/check-orchestrator/src/evidence/scan-surface";

export default migratedScanSurface({
  scannerPath: "scripts/check-rust-omena-query-cascade-input-authority.ts",
  mode: "index",
  pathspecs: [
    ":(glob)rust/crates/omena-query/src/*.rs",
    ":(glob)rust/crates/omena-query/src/**/*.rs",
  ],
  includeUntracked: false,
  excludes: ["personal-docs"],
});
