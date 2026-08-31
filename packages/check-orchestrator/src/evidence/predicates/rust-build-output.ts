import type { ScanSurfaceExcludePredicate } from "./types.ts";

const excludesRustBuildOutput: ScanSurfaceExcludePredicate = (candidate) =>
  candidate === "rust/target" || candidate.startsWith("rust/target/");

export default excludesRustBuildOutput;
