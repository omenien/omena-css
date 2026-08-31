import type { ScanSurfaceExcludePredicate } from "./types.ts";

const excludesGitMetadata: ScanSurfaceExcludePredicate = (candidate) =>
  candidate === ".git" || candidate.startsWith(".git/");

export default excludesGitMetadata;
