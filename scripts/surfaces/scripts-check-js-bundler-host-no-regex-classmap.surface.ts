import { migratedScanSurface } from "../../packages/check-orchestrator/src/evidence/scan-surface";

export default migratedScanSurface({
  scannerPath: "scripts/check-js-bundler-host-no-regex-classmap.ts",
  mode: "index",
  pathspecs: ["**"],
  includeUntracked: false,
  excludes: ["personal-docs"],
});
