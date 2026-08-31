import { migratedScanSurface } from "../../packages/check-orchestrator/src/evidence/scan-surface";

export default migratedScanSurface(
  {
    scannerPath: "scripts/check-docs-reference-surface.ts",
    mode: "index",
    pathspecs: ["**"],
    includeUntracked: true,
    excludes: ["git-metadata", "personal-docs", "rust-build-output"],
  },
  {
    narrowingReason:
      "match the scanner's tracked-plus-nonignored index contract; ignored worktree-only files were never scanner inputs",
  },
);
