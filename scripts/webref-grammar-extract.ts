import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";

import { classifyCssSpecEntry, loadCssSpecBoundaryContext } from "./css-spec-boundary";

// Shared extraction for the vendored `@webref/css` value-definition-syntax
// grammar snapshot. Both the generator (writes/checks the vendored snapshot) and
// the drift fence (re-extracts and compares) consume this module so the two gates
// can never disagree on what "the grammar" is.

export const WEBREF_PACKAGE = "@webref/css";
export const WEBREF_CSS_JSON = "node_modules/@webref/css/css.json";
export const WEBREF_PACKAGE_JSON = "node_modules/@webref/css/package.json";
export const SPEC_SOURCES_JSON = "rust/crates/omena-spec-audit/data/spec-sources.json";
export const WEBREF_GRAMMAR_SNAPSHOT = "rust/crates/omena-spec-audit/data/webref-grammar.json";
export const GENERATOR_TOOL = "scripts/generate-rust-omena-spec-audit-webref-grammar.ts";

// Categories carrying CSS value-definition syntax in `css.json`, in a fixed order
// so the snapshot is deterministic regardless of object-key iteration.
const CATEGORY_ORDER = ["atrules", "functions", "properties", "selectors", "types"] as const;

export interface WebrefGrammarEntry {
  readonly name: string;
  readonly href: string;
  readonly sourceOrdinal: number;
  readonly syntax: string | null;
  readonly boundary: {
    readonly classification: "in-boundary" | "forward-tier" | "excluded-with-reason";
    readonly reason: string;
    readonly ruleId: string;
    readonly browserSpecShortname: string | null;
  };
  readonly propertyMetadata?: WebrefPropertyMetadata;
}

export interface WebrefPropertyMetadata {
  readonly inherited: string | null;
  readonly initial: string | null;
  readonly appliesTo: string | null;
  readonly percentages: string | null;
  readonly computedValue: string | null;
  readonly animationType: string | null;
  readonly longhands: readonly string[] | null;
  readonly legacyAliasOf: string | null;
}

export interface WebrefGrammarSnapshot {
  readonly schemaVersion: "1";
  readonly product: "omena-spec-audit.webref-grammar";
  readonly source: { readonly package: string; readonly version: string; readonly gitHead: string };
  readonly generation: { readonly tool: string };
  readonly entryCount: number;
  readonly categories: Record<string, readonly WebrefGrammarEntry[]>;
}

interface SpecSourcePin {
  readonly package: string;
  readonly version: string;
  readonly gitHead: string;
}

function readJson<T>(filePath: string): T {
  return JSON.parse(readFileSync(filePath, "utf8")) as T;
}

// Deterministic UTF-16 code-unit comparison (NOT locale-dependent localeCompare),
// so the vendored snapshot is byte-stable across machines and CI runners.
function compareStrings(left: string, right: string): number {
  if (left < right) {
    return -1;
  }
  if (left > right) {
    return 1;
  }
  return 0;
}

/**
 * Extract the value-definition-syntax grammar from the installed, pinned
 * `@webref/css` package into a deterministic snapshot. Stamps the snapshot with
 * the pin's version + gitHead (npm tarballs strip `gitHead`, so it is carried
 * from the manifest pin) and asserts the installed version equals the pin.
 */
export function extractWebrefGrammarSnapshot(repoRoot: string): WebrefGrammarSnapshot {
  const cssJson = readJson<Record<string, unknown>>(path.join(repoRoot, WEBREF_CSS_JSON));
  const installed = readJson<{ version?: string }>(path.join(repoRoot, WEBREF_PACKAGE_JSON));
  const pins = readJson<{ sources?: readonly SpecSourcePin[] }>(
    path.join(repoRoot, SPEC_SOURCES_JSON),
  );
  const pin = (pins.sources ?? []).find((source) => source.package === WEBREF_PACKAGE);
  assert.ok(pin, `${SPEC_SOURCES_JSON} must pin ${WEBREF_PACKAGE}`);
  assert.equal(pin.gitHead.length, 40, `${WEBREF_PACKAGE} pin gitHead must be a 40-char SHA`);
  assert.equal(
    installed.version,
    pin.version,
    `installed ${WEBREF_PACKAGE} ${installed.version} does not match pinned ${pin.version}`,
  );
  const boundary = loadCssSpecBoundaryContext(repoRoot);

  const categories: Record<string, readonly WebrefGrammarEntry[]> = {};
  let entryCount = 0;
  for (const category of CATEGORY_ORDER) {
    const raw = cssJson[category];
    assert.ok(Array.isArray(raw), `css.json.${category} must be an array`);
    const entries: WebrefGrammarEntry[] = [];
    for (const [sourceOrdinal, item] of (
      raw as readonly {
        name?: unknown;
        href?: unknown;
        syntax?: unknown;
        inherited?: unknown;
        initial?: unknown;
        appliesTo?: unknown;
        percentages?: unknown;
        computedValue?: unknown;
        animationType?: unknown;
        longhands?: unknown;
        legacyAliasOf?: unknown;
      }[]
    ).entries()) {
      assert.equal(typeof item.name, "string", `css.json.${category} row name must be a string`);
      assert.equal(typeof item.href, "string", `css.json.${category} row href must be a string`);
      assert.ok(
        item.syntax === undefined || typeof item.syntax === "string",
        `css.json.${category} row syntax must be a string when present`,
      );
      const propertyMetadata =
        category === "properties"
          ? {
              inherited: optionalString(item.inherited, category, item.name as string, "inherited"),
              initial: optionalString(item.initial, category, item.name as string, "initial"),
              appliesTo: optionalString(item.appliesTo, category, item.name as string, "appliesTo"),
              percentages: optionalString(
                item.percentages,
                category,
                item.name as string,
                "percentages",
              ),
              computedValue: optionalString(
                item.computedValue,
                category,
                item.name as string,
                "computedValue",
              ),
              animationType: optionalString(
                item.animationType,
                category,
                item.name as string,
                "animationType",
              ),
              longhands: optionalStringArray(
                item.longhands,
                category,
                item.name as string,
                "longhands",
              ),
              legacyAliasOf: optionalString(
                item.legacyAliasOf,
                category,
                item.name as string,
                "legacyAliasOf",
              ),
            }
          : undefined;
      entries.push({
        name: item.name as string,
        href: item.href as string,
        sourceOrdinal,
        syntax: typeof item.syntax === "string" && item.syntax.length > 0 ? item.syntax : null,
        boundary: classifyCssSpecEntry(boundary, item.href as string),
        ...(propertyMetadata ? { propertyMetadata } : {}),
      });
    }
    entries.sort(
      (left, right) =>
        compareStrings(left.name, right.name) ||
        compareStrings(left.href, right.href) ||
        compareStrings(left.syntax ?? "", right.syntax ?? "") ||
        left.sourceOrdinal - right.sourceOrdinal,
    );
    categories[category] = entries;
    entryCount += entries.length;
  }

  return {
    schemaVersion: "1",
    product: "omena-spec-audit.webref-grammar",
    source: { package: pin.package, version: pin.version, gitHead: pin.gitHead },
    generation: { tool: GENERATOR_TOOL },
    entryCount,
    categories,
  };
}

function optionalString(
  value: unknown,
  category: string,
  name: string,
  field: string,
): string | null {
  assert.ok(
    value === undefined || typeof value === "string",
    `css.json.${category} ${name}.${field} must be a string when present`,
  );
  return typeof value === "string" ? value : null;
}

function optionalStringArray(
  value: unknown,
  category: string,
  name: string,
  field: string,
): readonly string[] | null {
  assert.ok(
    value === undefined ||
      (Array.isArray(value) && value.every((item) => typeof item === "string")),
    `css.json.${category} ${name}.${field} must be a string array when present`,
  );
  return Array.isArray(value) ? [...(value as readonly string[])] : null;
}

export function serializeWebrefGrammarSnapshot(snapshot: WebrefGrammarSnapshot): string {
  return `${JSON.stringify(snapshot, null, 2)}\n`;
}
