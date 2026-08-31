import type { ScanSurfaceExcludePredicate } from "./types.ts";

const excludesNodeModules: ScanSurfaceExcludePredicate = (candidate) =>
  candidate.split("/").includes("node_modules");

export default excludesNodeModules;
