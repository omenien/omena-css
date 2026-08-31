import type { ScanSurfaceExcludePredicate } from "./types.ts";

const excludesPersonalDocs: ScanSurfaceExcludePredicate = (candidate) =>
  candidate === ".personal_docs" || candidate.startsWith(".personal_docs/");

export default excludesPersonalDocs;
