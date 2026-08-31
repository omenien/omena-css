import { migratedScanSurface } from "../../packages/check-orchestrator/src/evidence/scan-surface";

export default migratedScanSurface({
  scannerPath: "packages/check-orchestrator/src/manifest/documented-commands.ts",
  mode: "workingTree",
  pathspecs: ["README.md", "CONTRIBUTING.md", "docs/**", "packages/**", "examples/**"],
  includeUntracked: false,
  excludes: ["git-metadata", "personal-docs", "node-modules", "rust-build-output"],
});
