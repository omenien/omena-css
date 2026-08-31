import { migratedScanSurface } from "../../packages/check-orchestrator/src/evidence/scan-surface";

export default migratedScanSurface({
  scannerPath: "scripts/measure-rust-omena-diagnostic-transition-attribution.ts",
  mode: "index",
  pathspecs: ["**/*.css", "**/*.scss", "**/*.less"],
  includeUntracked: false,
  excludes: ["personal-docs"],
});
