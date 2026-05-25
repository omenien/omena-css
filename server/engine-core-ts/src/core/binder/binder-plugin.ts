import type ts from "typescript";
import type { StyleImport } from "@css-module-explainer/shared";
import type { AliasResolver } from "../cx/alias-resolver";
import { detectClassUtilImports, scanCxImports, type CxScanResult } from "../cx/binding-detector";
import { parseClassExpressions } from "../cx/class-ref-parser";
import type { CxBinding } from "../cx/cx-types";
import { resolveCxBindings, type ResolvedCxBinding } from "../cx/resolved-bindings";
import type { ClassExpressionHIR, DomainClassReferenceHIR } from "../hir/source-types";
import type { ClassValueUniverseEntryV0 } from "./class-value-universe-provider";
import type { SourceBinderResult } from "./scope-types";

export interface BinderPluginAnalyzeInputV0 {
  readonly sourceFile: ts.SourceFile;
  readonly filePath: string;
  readonly sourceBinder: SourceBinderResult;
  readonly fileExists: (path: string) => boolean;
  readonly aliasResolver: AliasResolver;
}

export interface BinderPluginAnalyzeResultV0 {
  readonly pluginId: string;
  readonly stylesBindings: ReadonlyMap<string, StyleImport>;
  readonly rawCxBindings: readonly CxBinding[];
  readonly cxBindings: readonly ResolvedCxBinding[];
  readonly classUtilNames: readonly string[];
  readonly classExpressions: readonly ClassExpressionHIR[];
  readonly domainClassReferences: readonly DomainClassReferenceHIR[];
  readonly classValueUniverses: readonly ClassValueUniverseEntryV0[];
}

export interface BinderPluginV0 {
  readonly id: string;
  readonly version: "0";
  readonly stability: "builtIn";
  readonly domains: readonly string[];
  readonly importTargets: readonly string[];
  readonly utilityTargets: readonly string[];
  readonly ownsSurfaces: readonly string[];
  analyzeSource(input: BinderPluginAnalyzeInputV0): BinderPluginAnalyzeResultV0;
}

export const cssModulesClassnamesBinderPluginV0: BinderPluginV0 = {
  id: "css-modules-classnames-bind",
  version: "0",
  stability: "builtIn",
  domains: ["css-modules"],
  importTargets: ["*.module.css", "*.module.scss", "*.module.less"],
  utilityTargets: ["classnames/bind", "classnames", "clsx", "clsx/lite"],
  ownsSurfaces: [
    "styleImportRecognition",
    "classUtilityRecognition",
    "classReferenceExtraction",
    "sourceExpressionProjection",
  ],
  analyzeSource(input) {
    const scan = scanCxImports(
      input.sourceFile,
      input.filePath,
      input.fileExists,
      input.aliasResolver,
    );
    return analyzeCssModulesClassnamesScan(input.sourceFile, input.sourceBinder, scan);
  },
};

function analyzeCssModulesClassnamesScan(
  sourceFile: ts.SourceFile,
  sourceBinder: SourceBinderResult,
  scan: CxScanResult,
): BinderPluginAnalyzeResultV0 {
  const cxBindings = resolveCxBindings(scan.bindings, sourceBinder, sourceFile);
  const classUtilNames = detectClassUtilImports(sourceFile);
  const classExpressions = parseClassExpressions(
    sourceFile,
    cxBindings,
    scan.stylesBindings,
    sourceBinder,
  );

  return {
    pluginId: cssModulesClassnamesBinderPluginV0.id,
    stylesBindings: scan.stylesBindings,
    rawCxBindings: scan.bindings,
    cxBindings,
    classUtilNames,
    classExpressions,
    domainClassReferences: [],
    classValueUniverses: [],
  };
}

export function composeBinderPluginsV0(plugins: readonly BinderPluginV0[]): BinderPluginV0 {
  if (plugins.length === 0) {
    throw new Error("composeBinderPluginsV0 requires at least one plugin");
  }
  if (plugins.length === 1) {
    return plugins[0]!;
  }

  return {
    id: plugins.map((plugin) => plugin.id).join("+"),
    version: "0",
    stability: "builtIn",
    domains: uniqueFlatMap(plugins, (plugin) => plugin.domains),
    importTargets: uniqueFlatMap(plugins, (plugin) => plugin.importTargets),
    utilityTargets: uniqueFlatMap(plugins, (plugin) => plugin.utilityTargets),
    ownsSurfaces: uniqueFlatMap(plugins, (plugin) => plugin.ownsSurfaces),
    analyzeSource(input) {
      const analyses = plugins.map((plugin) => plugin.analyzeSource(input));
      return {
        pluginId: analyses.map((analysis) => analysis.pluginId).join("+"),
        stylesBindings: mergeStyleBindings(analyses.map((analysis) => analysis.stylesBindings)),
        rawCxBindings: analyses.flatMap((analysis) => analysis.rawCxBindings),
        cxBindings: analyses.flatMap((analysis) => analysis.cxBindings),
        classUtilNames: uniqueStrings(analyses.flatMap((analysis) => analysis.classUtilNames)),
        classExpressions: analyses.flatMap((analysis) => analysis.classExpressions),
        domainClassReferences: analyses.flatMap((analysis) => analysis.domainClassReferences),
        classValueUniverses: analyses.flatMap((analysis) => analysis.classValueUniverses),
      };
    },
  };
}

function mergeStyleBindings(
  bindings: readonly ReadonlyMap<string, StyleImport>[],
): ReadonlyMap<string, StyleImport> {
  const merged = new Map<string, StyleImport>();
  for (const bindingMap of bindings) {
    for (const [localName, styleImport] of bindingMap) {
      if (!merged.has(localName)) {
        merged.set(localName, styleImport);
      }
    }
  }
  return merged;
}

function uniqueFlatMap<T>(
  plugins: readonly BinderPluginV0[],
  select: (plugin: BinderPluginV0) => readonly T[],
): readonly T[] {
  return Array.from(new Set(plugins.flatMap((plugin) => [...select(plugin)])));
}

function uniqueStrings(values: readonly string[]): readonly string[] {
  return Array.from(new Set(values));
}
