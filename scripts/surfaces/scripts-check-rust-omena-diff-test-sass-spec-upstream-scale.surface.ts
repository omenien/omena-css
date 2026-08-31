import { unmigratedScanSurface } from "../../packages/check-orchestrator/src/evidence/scan-surface";

export default unmigratedScanSurface({
  scannerPath: "scripts/check-rust-omena-diff-test-sass-spec-upstream-scale.ts",
  reason: "external-checkout",
  inRepoSpec: {
    scannerPath: "scripts/check-rust-omena-diff-test-sass-spec-upstream-scale.ts",
    mode: "workingTree",
    pathspecs: [
      "rust/crates/omena-diff-test/fixtures/sass-spec-conformance/**/*.hrx",
      "rust/crates/omena-diff-test/fixtures/sass-spec-import/**/*.hrx",
    ],
    includeUntracked: false,
    excludes: ["git-metadata"],
  },
});
