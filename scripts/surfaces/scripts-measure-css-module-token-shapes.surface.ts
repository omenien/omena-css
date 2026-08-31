import { unmigratedScanSurface } from "../../packages/check-orchestrator/src/evidence/scan-surface";

export default unmigratedScanSurface({
  scannerPath: "scripts/measure-css-module-token-shapes.ts",
  reason: "external-checkout",
  inRepoSpec: {
    scannerPath: "scripts/measure-css-module-token-shapes.ts",
    mode: "index",
    pathspecs: ["*.module.css", "*.module.scss", "*.module.sass", "*.module.less"],
    includeUntracked: false,
    excludes: ["git-metadata", "rust-build-output", "test-only-rust"],
  },
});
