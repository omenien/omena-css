import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import ts from "typescript";
import type { Range, StyleImport } from "@omena/shared";
import { buildSourceBinder } from "../../server/engine-core-ts/src/core/binder/binder-builder";
import {
  detectClassUtilImports,
  scanCxImports,
} from "../../server/engine-core-ts/src/core/cx/binding-detector";
import { parseClassExpressions } from "../../server/engine-core-ts/src/core/cx/class-ref-parser";
import { resolveCxBindings } from "../../server/engine-core-ts/src/core/cx/resolved-bindings";
import { buildSourceDocument } from "../../server/engine-core-ts/src/core/hir/builders/ts-source-adapter";
import type { SourceDocumentHIR } from "../../server/engine-core-ts/src/core/hir/source-types";
import type { StyleDocumentHIR } from "../../server/engine-core-ts/src/core/hir/style-types";
import {
  expandStyleDocumentWithTransform,
  type ClassnameTransformMode,
} from "../../server/engine-core-ts/src/core/scss/classname-transform";
import { parseStyleDocument } from "../../server/engine-core-ts/src/core/scss/scss-parser";
import { EMPTY_ALIAS_RESOLVER } from "./test-helpers";

const REPO_ROOT = process.cwd();
const EXAMPLES_ROOT = path.join(REPO_ROOT, "examples/src/scenarios");

export interface SourceScenarioDef {
  readonly id: string;
  readonly sourcePath: string;
}

export interface StyleScenarioDef {
  readonly id: string;
  readonly stylePath: string;
  readonly mode?: ClassnameTransformMode;
}

export const SOURCE_SCENARIOS: readonly SourceScenarioDef[] = [
  { id: "01-basic", sourcePath: "01-basic/BasicScenario.tsx" },
  { id: "02-multi-binding", sourcePath: "02-multi-binding/MultiBindingScenario.tsx" },
  { id: "02-style-access", sourcePath: "02-multi-binding/StyleAccessDemo.tsx" },
  { id: "04-dynamic", sourcePath: "04-dynamic/DynamicScenario.tsx" },
  { id: "10-clsx", sourcePath: "10-clsx/ClsxScenario.tsx" },
];

export const STYLE_SCENARIOS: readonly StyleScenarioDef[] = [
  { id: "01-basic-style", stylePath: "01-basic/Button.module.scss" },
  {
    id: "02-button-camel-case",
    stylePath: "02-multi-binding/Button.module.scss",
    mode: "camelCase",
  },
  { id: "04-dynamic-style", stylePath: "04-dynamic/DynamicKeys.module.scss" },
  { id: "10-clsx-style", stylePath: "10-clsx/Clsx.module.scss" },
];

export interface LoadedSourceScenario {
  readonly id: string;
  readonly filePath: string;
  readonly relativePath: string;
  readonly sourceDocument: SourceDocumentHIR;
}

export interface LoadedStyleScenario {
  readonly id: string;
  readonly filePath: string;
  readonly relativePath: string;
  readonly styleDocument: StyleDocumentHIR;
}

export function loadSourceScenario(def: SourceScenarioDef): LoadedSourceScenario {
  const filePath = path.join(EXAMPLES_ROOT, def.sourcePath);
  const relativePath = toRepoRelative(filePath);
  const content = readFileSync(filePath, "utf8");
  const sourceFile = ts.createSourceFile(
    filePath,
    content,
    ts.ScriptTarget.Latest,
    true,
    scriptKindFor(filePath),
  );

  const { stylesBindings, bindings } = scanCxImports(
    sourceFile,
    filePath,
    existsSync,
    EMPTY_ALIAS_RESOLVER,
  );
  const sourceBinder = buildSourceBinder(sourceFile);
  const cxBindings = resolveCxBindings(bindings, sourceBinder, sourceFile);
  const sourceDocument = buildSourceDocument({
    filePath,
    cxBindings,
    stylesBindings,
    classUtilNames: detectClassUtilImports(sourceFile),
    sourceBinder,
    classExpressions: parseClassExpressions(sourceFile, cxBindings, stylesBindings, sourceBinder),
  });

  return {
    id: def.id,
    filePath,
    relativePath,
    sourceDocument,
  };
}

export function loadStyleScenario(def: StyleScenarioDef): LoadedStyleScenario {
  const filePath = path.join(EXAMPLES_ROOT, def.stylePath);
  const relativePath = toRepoRelative(filePath);
  const content = readFileSync(filePath, "utf8");
  const styleDocument = expandStyleDocumentWithTransform(
    parseStyleDocument(content, filePath),
    def.mode ?? "asIs",
  );

  return {
    id: def.id,
    filePath,
    relativePath,
    styleDocument,
  };
}

export function normalizeSourceDocument(doc: SourceDocumentHIR): unknown {
  return {
    filePath: toRepoRelative(doc.filePath),
    language: doc.language,
    styleImports: doc.styleImports.map((binding) => ({
      kind: binding.kind,
      localName: binding.localName,
      bindingDeclId: binding.bindingDeclId,
      resolved: normalizeStyleImport(binding.resolved),
      ...(binding.range ? { range: normalizeRange(binding.range) } : {}),
    })),
    utilityBindings: doc.utilityBindings.map((binding) =>
      binding.kind === "classnamesBind"
        ? {
            kind: binding.kind,
            localName: binding.localName,
            stylesLocalName: binding.stylesLocalName,
            scssModulePath: toRepoRelative(binding.scssModulePath),
            classNamesImportName: binding.classNamesImportName,
            bindingDeclId: binding.bindingDeclId,
          }
        : {
            kind: binding.kind,
            localName: binding.localName,
            bindingDeclId: binding.bindingDeclId,
          },
    ),
    classExpressions: doc.classExpressions.map((expr) => ({
      kind: expr.kind,
      origin: expr.origin,
      scssModulePath: toRepoRelative(expr.scssModulePath),
      range: normalizeRange(expr.range),
      ...(expr.kind === "literal" ? { className: expr.className } : {}),
      ...(expr.kind === "template"
        ? { rawTemplate: expr.rawTemplate, staticPrefix: expr.staticPrefix }
        : {}),
      ...(expr.kind === "symbolRef"
        ? {
            rawReference: expr.rawReference,
            rootName: expr.rootName,
            pathSegments: [...expr.pathSegments],
            ...(expr.rootBindingDeclId ? { rootBindingDeclId: expr.rootBindingDeclId } : {}),
          }
        : {}),
      ...(expr.kind === "styleAccess"
        ? {
            bindingDeclId: expr.bindingDeclId,
            className: expr.className,
            accessPath: [...expr.accessPath],
          }
        : {}),
    })),
  };
}

export function normalizeStyleDocument(doc: StyleDocumentHIR): unknown {
  return {
    filePath: toRepoRelative(doc.filePath),
    selectors: doc.selectors.map((selector) => ({
      name: selector.name,
      canonicalName: selector.canonicalName,
      viewKind: selector.viewKind,
      nestedSafety: selector.nestedSafety,
      range: normalizeRange(selector.range),
      ruleRange: normalizeRange(selector.ruleRange),
      fullSelector: selector.fullSelector,
      ...(selector.originalName ? { originalName: selector.originalName } : {}),
      ...(selector.bemSuffix
        ? {
            bemSuffix: {
              rawToken: selector.bemSuffix.rawToken,
              rawTokenRange: normalizeRange(selector.bemSuffix.rawTokenRange),
              parentResolvedName: selector.bemSuffix.parentResolvedName,
            },
          }
        : {}),
      ...(selector.composes.length > 0 ? { composes: selector.composes } : {}),
    })),
    ...(doc.sassModuleUses.length > 0
      ? {
          sassModuleUses: doc.sassModuleUses.map((moduleUse) => ({
            source: moduleUse.source,
            namespaceKind: moduleUse.namespaceKind,
            namespace: moduleUse.namespace,
            range: normalizeRange(moduleUse.range),
            ruleRange: normalizeRange(moduleUse.ruleRange),
          })),
        }
      : {}),
    ...(doc.sassModuleForwards.length > 0
      ? {
          sassModuleForwards: doc.sassModuleForwards.map((moduleForward) => ({
            source: moduleForward.source,
            prefix: moduleForward.prefix,
            visibilityKind: moduleForward.visibilityKind,
            visibilityMembers: moduleForward.visibilityMembers,
            range: normalizeRange(moduleForward.range),
            ruleRange: normalizeRange(moduleForward.ruleRange),
          })),
        }
      : {}),
    ...(doc.sassModuleMemberRefs.length > 0
      ? {
          sassModuleMemberRefs: doc.sassModuleMemberRefs.map((memberRef) => ({
            selectorName: memberRef.selectorName,
            namespace: memberRef.namespace,
            symbolKind: memberRef.symbolKind,
            name: memberRef.name,
            role: memberRef.role,
            range: normalizeRange(memberRef.range),
            ruleRange: normalizeRange(memberRef.ruleRange),
          })),
        }
      : {}),
  };
}

function normalizeStyleImport(styleImport: StyleImport): unknown {
  return styleImport.kind === "resolved"
    ? {
        kind: "resolved",
        absolutePath: toRepoRelative(styleImport.absolutePath),
      }
    : {
        kind: "missing",
        absolutePath: toRepoRelative(styleImport.absolutePath),
        specifier: styleImport.specifier,
        range: normalizeRange(styleImport.range),
      };
}

function normalizeRange(range: Range): readonly [number, number, number, number] {
  return [range.start.line, range.start.character, range.end.line, range.end.character];
}

function toRepoRelative(filePath: string): string {
  return path.relative(REPO_ROOT, filePath).replaceAll(path.sep, "/");
}

function scriptKindFor(filePath: string): ts.ScriptKind {
  if (filePath.endsWith(".tsx")) return ts.ScriptKind.TSX;
  if (filePath.endsWith(".ts")) return ts.ScriptKind.TS;
  if (filePath.endsWith(".jsx")) return ts.ScriptKind.JSX;
  return ts.ScriptKind.JS;
}
