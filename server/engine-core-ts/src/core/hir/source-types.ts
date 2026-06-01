import type { ClassRefOrigin, Range, StyleImport } from "@omena/shared";
import type { HirDocumentBase, HirNodeBase, SourceLanguage } from "./shared-types";

export interface SourceDocumentHIR extends HirDocumentBase {
  readonly kind: "source";
  readonly language: SourceLanguage;
  readonly styleImports: readonly StyleImportBindingHIR[];
  readonly utilityBindings: readonly UtilityBindingHIR[];
  readonly classExpressions: readonly ClassExpressionHIR[];
  readonly domainClassReferences: readonly DomainClassReferenceHIR[];
}

export interface StyleImportBindingHIR extends HirNodeBase {
  readonly kind: "styleImport";
  readonly localName: string;
  readonly bindingDeclId: string;
  readonly resolved: StyleImport;
}

export interface ClassnamesBindUtilityBindingHIR extends HirNodeBase {
  readonly kind: "classnamesBind";
  readonly localName: string;
  readonly stylesLocalName: string;
  readonly scssModulePath: string;
  readonly classNamesImportName: string;
  readonly bindingDeclId: string;
}

export interface ClassUtilBindingHIR extends HirNodeBase {
  readonly kind: "classUtil";
  readonly localName: string;
  readonly bindingDeclId: string;
}

export type UtilityBindingHIR = ClassnamesBindUtilityBindingHIR | ClassUtilBindingHIR;

interface ClassExpressionBase extends HirNodeBase {
  readonly origin: ClassRefOrigin;
  readonly scssModulePath: string;
  readonly range: Range;
}

export interface LiteralClassExpressionHIR extends ClassExpressionBase {
  readonly kind: "literal";
  readonly className: string;
}

export interface TemplateClassExpressionHIR extends ClassExpressionBase {
  readonly kind: "template";
  readonly rawTemplate: string;
  readonly staticPrefix: string;
}

export interface SymbolRefClassExpressionHIR extends ClassExpressionBase {
  readonly kind: "symbolRef";
  readonly rawReference: string;
  readonly rootName: string;
  readonly pathSegments: readonly string[];
  readonly rootBindingDeclId?: string;
}

/**
 * Source HIR for direct `styles.x` / `styles["x"]` access.
 *
 * This preserves the resolved class token and access path so later
 * queries do not need to re-parse property access syntax at the
 * provider boundary.
 */
export interface StyleAccessClassExpressionHIR extends ClassExpressionBase {
  readonly kind: "styleAccess";
  readonly bindingDeclId: string;
  readonly className: string;
  readonly accessPath: readonly string[];
}

export type ClassExpressionHIR =
  | LiteralClassExpressionHIR
  | TemplateClassExpressionHIR
  | SymbolRefClassExpressionHIR
  | StyleAccessClassExpressionHIR;

export type SourceExpressionKind = ClassExpressionHIR["kind"];

interface DomainClassReferenceBase extends HirNodeBase {
  readonly kind: "domainClassReference";
  readonly pluginId: string;
  readonly domain: string;
  readonly origin: "jsxClassAttribute" | "classUtilityCall" | "styleAccess";
  readonly range: Range;
}

export interface DomainLiteralClassReferenceHIR extends DomainClassReferenceBase {
  readonly matchKind: "literal";
  readonly className: string;
}

export interface DomainTemplateClassReferenceHIR extends DomainClassReferenceBase {
  readonly matchKind: "templatePrefix";
  readonly rawTemplate: string;
  readonly staticPrefix: string;
}

export type DomainClassReferenceHIR =
  | DomainLiteralClassReferenceHIR
  | DomainTemplateClassReferenceHIR;

export interface BuildSourceDocumentHIRArgs {
  readonly filePath: string;
  readonly language: SourceLanguage;
  readonly styleImports: readonly StyleImportBindingHIR[];
  readonly utilityBindings: readonly UtilityBindingHIR[];
  readonly classExpressions: readonly ClassExpressionHIR[];
  readonly domainClassReferences?: readonly DomainClassReferenceHIR[];
}

export function makeStyleImportBinding(
  id: string,
  localName: string,
  bindingDeclId: string,
  resolved: StyleImport,
): StyleImportBindingHIR {
  return resolved.kind === "missing"
    ? { kind: "styleImport", id, localName, bindingDeclId, resolved, range: resolved.range }
    : { kind: "styleImport", id, localName, bindingDeclId, resolved };
}

export function makeClassUtilBinding(
  id: string,
  localName: string,
  bindingDeclId: string,
): ClassUtilBindingHIR {
  return { kind: "classUtil", id, localName, bindingDeclId };
}

export function makeSourceDocumentHIR(args: BuildSourceDocumentHIRArgs): SourceDocumentHIR {
  return {
    kind: "source",
    filePath: args.filePath,
    language: args.language,
    styleImports: args.styleImports,
    utilityBindings: args.utilityBindings,
    classExpressions: args.classExpressions,
    domainClassReferences: args.domainClassReferences ?? [],
  };
}

export function makeLiteralClassExpression(
  id: string,
  origin: ClassRefOrigin,
  scssModulePath: string,
  className: string,
  range: Range,
): LiteralClassExpressionHIR {
  return { kind: "literal", id, origin, scssModulePath, className, range };
}

export function makeTemplateClassExpression(
  id: string,
  origin: ClassRefOrigin,
  scssModulePath: string,
  rawTemplate: string,
  staticPrefix: string,
  range: Range,
): TemplateClassExpressionHIR {
  return { kind: "template", id, origin, scssModulePath, rawTemplate, staticPrefix, range };
}

export function makeSymbolRefClassExpression(
  id: string,
  origin: ClassRefOrigin,
  scssModulePath: string,
  rawReference: string,
  rootName: string,
  pathSegments: readonly string[],
  range: Range,
  rootBindingDeclId?: string,
): SymbolRefClassExpressionHIR {
  return {
    kind: "symbolRef",
    id,
    origin,
    scssModulePath,
    rawReference,
    rootName,
    pathSegments,
    range,
    ...(rootBindingDeclId ? { rootBindingDeclId } : {}),
  };
}

export function makeStyleAccessClassExpression(
  id: string,
  scssModulePath: string,
  bindingDeclId: string,
  className: string,
  accessPath: readonly string[],
  range: Range,
): StyleAccessClassExpressionHIR {
  return {
    kind: "styleAccess",
    id,
    origin: "styleAccess",
    scssModulePath,
    bindingDeclId,
    className,
    accessPath,
    range,
  };
}

export function makeDomainLiteralClassReference(
  id: string,
  pluginId: string,
  domain: string,
  origin: DomainClassReferenceHIR["origin"],
  className: string,
  range: Range,
): DomainLiteralClassReferenceHIR {
  return {
    kind: "domainClassReference",
    matchKind: "literal",
    id,
    pluginId,
    domain,
    origin,
    className,
    range,
  };
}

export function makeDomainTemplateClassReference(
  id: string,
  pluginId: string,
  domain: string,
  origin: DomainClassReferenceHIR["origin"],
  rawTemplate: string,
  staticPrefix: string,
  range: Range,
): DomainTemplateClassReferenceHIR {
  return {
    kind: "domainClassReference",
    matchKind: "templatePrefix",
    id,
    pluginId,
    domain,
    origin,
    rawTemplate,
    staticPrefix,
    range,
  };
}
