import excludesGitMetadata from "./git-metadata.ts";
import excludesNodeModules from "./node-modules.ts";
import excludesPersonalDocs from "./personal-docs.ts";
import excludesRustBuildOutput from "./rust-build-output.ts";
import excludesTestOnlyRust from "./test-only-rust.ts";
import type { ScanSurfaceExcludePredicate } from "./types.ts";

export const SCAN_SURFACE_EXCLUDE_PREDICATES = {
  "git-metadata": excludesGitMetadata,
  "node-modules": excludesNodeModules,
  "personal-docs": excludesPersonalDocs,
  "rust-build-output": excludesRustBuildOutput,
  "test-only-rust": excludesTestOnlyRust,
} as const satisfies Readonly<Record<string, ScanSurfaceExcludePredicate>>;

export const SCAN_SURFACE_PREDICATE_MODULE_PATHS = {
  "git-metadata": "packages/check-orchestrator/src/evidence/predicates/git-metadata.ts",
  "node-modules": "packages/check-orchestrator/src/evidence/predicates/node-modules.ts",
  "personal-docs": "packages/check-orchestrator/src/evidence/predicates/personal-docs.ts",
  "rust-build-output": "packages/check-orchestrator/src/evidence/predicates/rust-build-output.ts",
  "test-only-rust": "packages/check-orchestrator/src/evidence/predicates/test-only-rust.ts",
} as const;
