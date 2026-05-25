import type { StyleImport } from "@css-module-explainer/shared";
import { findImportDeclId } from "../../binder/import-decls";
import type { SourceBinderResult } from "../../binder/scope-types";
import type { ResolvedCxBinding } from "../../cx/resolved-bindings";
import {
  makeClassUtilBinding,
  makeSourceDocumentHIR,
  makeStyleImportBinding,
  type ClassExpressionHIR,
  type DomainClassReferenceHIR,
  type SourceDocumentHIR,
  type UtilityBindingHIR,
} from "../source-types";
import type { SourceLanguage } from "../shared-types";

export interface BuildSourceDocumentArgs {
  readonly filePath: string;
  readonly cxBindings: readonly ResolvedCxBinding[];
  readonly stylesBindings: ReadonlyMap<string, StyleImport>;
  readonly classUtilNames: readonly string[];
  readonly classExpressions: readonly ClassExpressionHIR[];
  readonly domainClassReferences?: readonly DomainClassReferenceHIR[];
  readonly sourceBinder?: SourceBinderResult;
}

export function buildSourceDocument(args: BuildSourceDocumentArgs): SourceDocumentHIR {
  const styleImports = Array.from(args.stylesBindings.entries(), ([localName, resolved], index) =>
    makeStyleImportBinding(
      `style-import:${index}`,
      localName,
      findImportDeclId(args.sourceBinder, localName) ??
        `synthetic-import-decl:${localName}:${index}`,
      resolved,
    ),
  );
  const utilityBindings: UtilityBindingHIR[] = [
    ...args.cxBindings.map((binding, index) => ({
      kind: "classnamesBind" as const,
      id: `utility-binding:cx:${index}`,
      localName: binding.cxVarName,
      stylesLocalName: binding.stylesVarName,
      scssModulePath: binding.scssModulePath,
      classNamesImportName: binding.classNamesImportName,
      bindingDeclId: binding.bindingDeclId,
    })),
    ...args.classUtilNames.map((localName, index) =>
      makeClassUtilBinding(
        `utility-binding:util:${index}`,
        localName,
        findImportDeclId(
          args.sourceBinder,
          localName,
          new Set(["clsx", "clsx/lite", "classnames"]),
        ) ?? `synthetic-import-decl:${localName}:${index}`,
      ),
    ),
  ];

  return makeSourceDocumentHIR({
    filePath: args.filePath,
    language: inferSourceLanguage(args.filePath),
    styleImports,
    utilityBindings,
    classExpressions: args.classExpressions,
    ...(args.domainClassReferences ? { domainClassReferences: args.domainClassReferences } : {}),
  });
}

function inferSourceLanguage(filePath: string): SourceLanguage {
  if (filePath.endsWith(".vue")) return "vue";
  if (filePath.endsWith(".tsx")) return "typescriptreact";
  if (filePath.endsWith(".ts") || filePath.endsWith(".mts") || filePath.endsWith(".cts")) {
    return "typescript";
  }
  if (filePath.endsWith(".jsx")) return "javascriptreact";
  if (filePath.endsWith(".js") || filePath.endsWith(".mjs") || filePath.endsWith(".cjs")) {
    return "javascript";
  }
  return "unknown";
}
