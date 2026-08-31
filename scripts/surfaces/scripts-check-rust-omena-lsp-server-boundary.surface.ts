import { migratedScanSurface } from "../../packages/check-orchestrator/src/evidence/scan-surface";

export default migratedScanSurface({
  scannerPath: "scripts/check-rust-omena-lsp-server-boundary.ts",
  mode: "workingTree",
  pathspecs: ["rust/**", "server/**", "docs/**"],
  includeUntracked: false,
  excludes: ["git-metadata", "personal-docs", "node-modules", "rust-build-output"],
});
