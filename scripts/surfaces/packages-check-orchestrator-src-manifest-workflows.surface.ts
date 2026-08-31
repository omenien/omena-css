import { migratedScanSurface } from "../../packages/check-orchestrator/src/evidence/scan-surface";

export default migratedScanSurface({
  scannerPath: "packages/check-orchestrator/src/manifest/workflows.ts",
  mode: "workingTree",
  pathspecs: [".github/workflows/**"],
  includeUntracked: false,
  excludes: ["git-metadata", "personal-docs", "node-modules", "rust-build-output"],
});
