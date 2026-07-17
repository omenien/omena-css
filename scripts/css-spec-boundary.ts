import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";

export const CSS_SPEC_BOUNDARY_PATH = "rust/crates/omena-spec-audit/data/css-spec-boundary.json";
export const BROWSER_SPECS_PATH = "node_modules/browser-specs/index.json";

export type CssSpecBoundaryClassification = "in-boundary" | "forward-tier" | "excluded-with-reason";

export interface CssSpecBoundaryVerdict {
  readonly classification: CssSpecBoundaryClassification;
  readonly reason: string;
  readonly ruleId: string;
  readonly browserSpecShortname: string | null;
}

interface FixedSourceRule {
  readonly id: string;
  readonly origin: string;
  readonly pathPrefix: string;
  readonly strategy: "fixed";
  readonly classification: CssSpecBoundaryClassification;
  readonly reason: string;
}

interface BrowserSpecsSourceRule {
  readonly id: string;
  readonly origin: string;
  readonly pathPrefix: string;
  readonly strategy: "browser-specs";
}

type CssSpecBoundarySourceRule = FixedSourceRule | BrowserSpecsSourceRule;

export interface CssSpecBoundaryData {
  readonly schemaVersion: "0";
  readonly product: "omena-spec-audit.css-spec-boundary";
  readonly snapshot: {
    readonly year: number;
    readonly url: string;
    readonly stableSections: readonly string[];
    readonly forwardSections: readonly string[];
  };
  readonly browserSpecsPolicy: {
    readonly package: "browser-specs";
    readonly acceptedStanding: readonly string[];
    readonly forwardSeriesComposition: readonly string[];
    readonly unacceptedStandingReason: string;
  };
  readonly sourceRules: readonly CssSpecBoundarySourceRule[];
  readonly statusBucketPolicy: Readonly<
    Record<
      "vendorPrefixed" | "deprecated" | "experimental" | "forwardSpecification",
      { readonly classification: CssSpecBoundaryClassification; readonly reason: string }
    >
  >;
  readonly reasonTaxonomy: readonly string[];
}

interface BrowserSpecRecord {
  readonly url?: string;
  readonly shortname?: string;
  readonly standing?: string;
  readonly seriesComposition?: string;
  readonly series?: { readonly nightlyUrl?: string };
  readonly nightly?: { readonly url?: string; readonly alternateUrls?: readonly string[] };
}

export interface CssSpecBoundaryContext {
  readonly boundary: CssSpecBoundaryData;
  readonly browserSpecsByRoot: ReadonlyMap<string, BrowserSpecRecord>;
}

export function loadCssSpecBoundaryContext(repoRoot = process.cwd()): CssSpecBoundaryContext {
  const boundary = readJson<CssSpecBoundaryData>(path.join(repoRoot, CSS_SPEC_BOUNDARY_PATH));
  const browserSpecs = readJson<readonly BrowserSpecRecord[]>(
    path.join(repoRoot, BROWSER_SPECS_PATH),
  );
  validateBoundary(boundary);
  return { boundary, browserSpecsByRoot: indexBrowserSpecs(browserSpecs) };
}

export function classifyCssSpecEntry(
  context: CssSpecBoundaryContext,
  href: string,
): CssSpecBoundaryVerdict {
  const url = new URL(href);
  const rule = context.boundary.sourceRules.find(
    (candidate) => candidate.origin === url.origin && url.pathname.startsWith(candidate.pathPrefix),
  );
  assert.ok(rule, `no CSS specification boundary rule classifies ${href}`);

  if (rule.strategy === "fixed") {
    return {
      classification: rule.classification,
      reason: rule.reason,
      ruleId: rule.id,
      browserSpecShortname: null,
    };
  }

  const root = specificationRoot(url);
  assert.ok(root, `browser-specs boundary URL has no path segment: ${href}`);
  const browserSpec = context.browserSpecsByRoot.get(root);
  assert.ok(browserSpec, `browser-specs has no record for ${root} (from ${href})`);
  const policy = context.boundary.browserSpecsPolicy;
  if (!policy.acceptedStanding.includes(browserSpec.standing ?? "")) {
    return {
      classification: "excluded-with-reason",
      reason: policy.unacceptedStandingReason,
      ruleId: rule.id,
      browserSpecShortname: browserSpec.shortname ?? null,
    };
  }
  if (policy.forwardSeriesComposition.includes(browserSpec.seriesComposition ?? "")) {
    return {
      classification: "forward-tier",
      reason: "forward-specification",
      ruleId: rule.id,
      browserSpecShortname: browserSpec.shortname ?? null,
    };
  }
  return {
    classification: "in-boundary",
    reason: "stable-css-snapshot",
    ruleId: rule.id,
    browserSpecShortname: browserSpec.shortname ?? null,
  };
}

function validateBoundary(boundary: CssSpecBoundaryData): void {
  assert.equal(boundary.schemaVersion, "0");
  assert.equal(boundary.product, "omena-spec-audit.css-spec-boundary");
  assert.deepEqual(boundary.snapshot.stableSections, ["2.1", "2.2"]);
  assert.deepEqual(boundary.snapshot.forwardSections, ["2.3", "2.4"]);
  assert.ok(boundary.snapshot.year >= 2021, "CSS snapshot year must be explicit and current");
  assert.ok(boundary.sourceRules.length > 0, "CSS specification boundary needs source rules");
  const reasons = new Set(boundary.reasonTaxonomy);
  assert.equal(reasons.size, boundary.reasonTaxonomy.length, "boundary reasons must be unique");
  for (const rule of boundary.sourceRules) {
    assert.ok(rule.id.length > 0, "boundary source rule id must be present");
    if (rule.strategy === "fixed") {
      assert.ok(reasons.has(rule.reason), `boundary source rule ${rule.id} uses unknown reason`);
    }
  }
  for (const [bucket, policy] of Object.entries(boundary.statusBucketPolicy)) {
    assert.ok(reasons.has(policy.reason), `boundary status bucket ${bucket} uses unknown reason`);
  }
}

function indexBrowserSpecs(
  browserSpecs: readonly BrowserSpecRecord[],
): ReadonlyMap<string, BrowserSpecRecord> {
  const byRoot = new Map<string, BrowserSpecRecord>();
  for (const spec of browserSpecs) {
    const urls = [
      spec.url,
      spec.series?.nightlyUrl,
      spec.nightly?.url,
      ...(spec.nightly?.alternateUrls ?? []),
    ];
    for (const candidate of urls) {
      if (!candidate) continue;
      const url = new URL(candidate);
      const root = specificationRoot(url);
      if (root) {
        byRoot.set(root, spec);
      }
    }
  }
  return byRoot;
}

function specificationRoot(url: URL): string | null {
  const firstSegment = url.pathname.split("/").filter(Boolean)[0];
  return firstSegment ? `${url.origin}/${firstSegment}` : null;
}

function readJson<T>(filePath: string): T {
  return JSON.parse(readFileSync(filePath, "utf8")) as T;
}
