import { migratedScanSurface } from "../../packages/check-orchestrator/src/evidence/scan-surface";

export default migratedScanSurface({
  scannerPath: "scripts/lib/oss-corpus-local-workspace-files.ts",
  mode: "workingTree",
  pathspecs: ["**"],
  includeUntracked: false,
  excludes: ["git-metadata", "node-modules", "rust-build-output"],
});
