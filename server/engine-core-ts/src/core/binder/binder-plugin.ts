import type ts from "typescript";
import type { StyleImport } from "@css-module-explainer/shared";
import type { AliasResolver } from "../cx/alias-resolver";
import { detectClassUtilImports, scanCxImports, type CxScanResult } from "../cx/binding-detector";
import { parseClassExpressions } from "../cx/class-ref-parser";
import type { CxBinding } from "../cx/cx-types";
import { resolveCxBindings, type ResolvedCxBinding } from "../cx/resolved-bindings";
import type { ClassExpressionHIR } from "../hir/source-types";
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
}

export interface BinderPluginV0 {
  readonly id: string;
  readonly version: "0";
  readonly stability: "builtIn";
  readonly domains: readonly string[];
  readonly importTargets: readonly string[];
  readonly utilityTargets: readonly string[];
  analyzeSource(input: BinderPluginAnalyzeInputV0): BinderPluginAnalyzeResultV0;
}

export const cssModulesClassnamesBinderPluginV0: BinderPluginV0 = {
  id: "css-modules-classnames-bind",
  version: "0",
  stability: "builtIn",
  domains: ["css-modules"],
  importTargets: ["*.module.css", "*.module.scss", "*.module.less"],
  utilityTargets: ["classnames/bind", "classnames", "clsx", "clsx/lite"],
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
  };
}
