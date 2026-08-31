import type { ScanSurfaceExcludePredicate } from "./types.ts";

const excludesTestOnlyRust: ScanSurfaceExcludePredicate = (candidate) =>
  /(?:^|\/)(?:tests?|benches|examples|fuzz)(?:\/|$)/u.test(candidate);

export default excludesTestOnlyRust;
