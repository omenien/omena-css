import { migratedScanSurface } from "../../packages/check-orchestrator/src/evidence/scan-surface";

export default migratedScanSurface({
  scannerPath: "scripts/check-engine-v2-contract-idl-fixtures.ts",
  mode: "workingTree",
  pathspecs: ["**/*.json"],
  includeUntracked: false,
  excludes: ["git-metadata"],
});
