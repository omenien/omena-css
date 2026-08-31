import { migratedScanSurface } from "../../packages/check-orchestrator/src/evidence/scan-surface";

export default migratedScanSurface({
  scannerPath: "scripts/check-packaged-omena-lsp-server-type-fact-protocol.ts",
  mode: "workingTree",
  pathspecs: ["*.vsix"],
  includeUntracked: false,
  excludes: ["git-metadata", "personal-docs", "node-modules", "rust-build-output"],
});
