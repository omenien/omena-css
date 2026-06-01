import path from "node:path";
import type { Range } from "@omena/shared";
import type {
  CustomPropertyDeclHIR,
  CustomPropertyRefHIR,
  SassModuleUseHIR,
  SassSymbolKind,
  StyleDocumentHIR,
  ValueImportHIR,
} from "../hir/style-types";
import { findSassSymbolDeclForSymbol } from "../query/find-style-selector";

export type StyleDependencyReason = "localComposes" | "crossFileComposes";

export interface StyleDependencySelectorRef {
  readonly filePath: string;
  readonly canonicalName: string;
  readonly reason: StyleDependencyReason;
  readonly range?: Range;
}

interface StyleDependencyEdge {
  readonly fromFilePath: string;
  readonly fromCanonicalName: string;
  readonly toFilePath: string;
  readonly toCanonicalName: string;
  readonly reason: StyleDependencyReason;
  readonly range?: Range;
}

export interface SassModuleMemberDependencyRef {
  readonly filePath: string;
  readonly namespace: string;
  readonly symbolKind: SassSymbolKind;
  readonly name: string;
  readonly range: Range;
}

interface SassModuleMemberDependencyEdge extends SassModuleMemberDependencyRef {
  readonly toFilePath: string;
}

export interface ValueImportDependencyRef extends Pick<
  ValueImportHIR,
  "name" | "importedName" | "importedNameRange" | "range" | "ruleRange"
> {
  readonly filePath: string;
  readonly from: string;
}

interface ValueImportDependencyEdge extends ValueImportDependencyRef {
  readonly toFilePath: string;
}

export interface CustomPropertyDependencyDecl extends Pick<
  CustomPropertyDeclHIR,
  "name" | "value" | "range" | "ruleRange" | "context"
> {
  readonly filePath: string;
}

export interface CustomPropertyDependencyRef extends Pick<
  CustomPropertyRefHIR,
  "name" | "range" | "context"
> {
  readonly filePath: string;
}

export interface SassModuleExportedSymbolDependencyTarget {
  readonly filePath: string;
  readonly name: string;
}

export interface StyleDependencyRecordOptions {
  readonly resolveSassModuleUseTargetFilePath?: (moduleUse: SassModuleUseHIR) => string | null;
  readonly resolveSassModuleExportedSymbolTargets?: (
    moduleUse: SassModuleUseHIR,
    symbolKind: SassSymbolKind,
    name: string,
  ) => readonly SassModuleExportedSymbolDependencyTarget[];
  readonly resolveSassModuleExportedSymbolTargetFilePaths?: (
    moduleUse: SassModuleUseHIR,
    symbolKind: SassSymbolKind,
    name: string,
  ) => readonly string[];
}

export interface StyleDependencyGraph {
  record(
    filePath: string,
    styleDocument: StyleDocumentHIR,
    options?: StyleDependencyRecordOptions,
  ): void;
  forget(filePath: string): void;
  forgetWithinRoot(rootPath: string): void;
  getIncoming(filePath: string, canonicalName: string): readonly StyleDependencySelectorRef[];
  getOutgoing(filePath: string, canonicalName: string): readonly StyleDependencySelectorRef[];
  getIncomingSassModuleMemberRefs(
    filePath: string,
    symbolKind: SassSymbolKind,
    name: string,
  ): readonly SassModuleMemberDependencyRef[];
  getIncomingValueImports(filePath: string, name: string): readonly ValueImportDependencyRef[];
  getAllCustomPropertyDecls(): readonly CustomPropertyDependencyDecl[];
  getCustomPropertyDecls(name: string): readonly CustomPropertyDependencyDecl[];
  getCustomPropertyRefs(name: string): readonly CustomPropertyDependencyRef[];
}

export class WorkspaceStyleDependencyGraph implements StyleDependencyGraph {
  private readonly moduleEdges = new Map<
    string,
    {
      readonly selectorEdges: readonly StyleDependencyEdge[];
      readonly sassModuleMemberEdges: readonly SassModuleMemberDependencyEdge[];
      readonly valueImportEdges: readonly ValueImportDependencyEdge[];
      readonly customPropertyDecls: readonly CustomPropertyDependencyDecl[];
      readonly customPropertyRefs: readonly CustomPropertyDependencyRef[];
    }
  >();
  private readonly incoming = new Map<string, readonly StyleDependencySelectorRef[]>();
  private readonly outgoing = new Map<string, readonly StyleDependencySelectorRef[]>();
  private readonly incomingSassModuleMembers = new Map<
    string,
    readonly SassModuleMemberDependencyRef[]
  >();
  private readonly incomingValueImports = new Map<string, readonly ValueImportDependencyRef[]>();
  private readonly customPropertyDecls = new Map<string, readonly CustomPropertyDependencyDecl[]>();
  private readonly customPropertyRefs = new Map<string, readonly CustomPropertyDependencyRef[]>();

  record(
    filePath: string,
    styleDocument: StyleDocumentHIR,
    options: StyleDependencyRecordOptions = {},
  ): void {
    this.moduleEdges.set(filePath, {
      selectorEdges: collectEdges(filePath, styleDocument),
      sassModuleMemberEdges: collectSassModuleMemberEdges(filePath, styleDocument, options),
      valueImportEdges: collectValueImportEdges(filePath, styleDocument),
      customPropertyDecls: collectCustomPropertyDecls(filePath, styleDocument),
      customPropertyRefs: collectCustomPropertyRefs(filePath, styleDocument),
    });
    this.rebuild();
  }

  forget(filePath: string): void {
    if (!this.moduleEdges.delete(filePath)) return;
    this.rebuild();
  }

  forgetWithinRoot(rootPath: string): void {
    let changed = false;
    for (const filePath of this.moduleEdges.keys()) {
      if (!isWithinRoot(rootPath, filePath)) continue;
      this.moduleEdges.delete(filePath);
      changed = true;
    }
    if (changed) this.rebuild();
  }

  getIncoming(filePath: string, canonicalName: string): readonly StyleDependencySelectorRef[] {
    return this.incoming.get(selectorKey(filePath, canonicalName)) ?? [];
  }

  getOutgoing(filePath: string, canonicalName: string): readonly StyleDependencySelectorRef[] {
    return this.outgoing.get(selectorKey(filePath, canonicalName)) ?? [];
  }

  getIncomingSassModuleMemberRefs(
    filePath: string,
    symbolKind: SassSymbolKind,
    name: string,
  ): readonly SassModuleMemberDependencyRef[] {
    return this.incomingSassModuleMembers.get(sassMemberKey(filePath, symbolKind, name)) ?? [];
  }

  getIncomingValueImports(filePath: string, name: string): readonly ValueImportDependencyRef[] {
    return this.incomingValueImports.get(valueImportKey(filePath, name)) ?? [];
  }

  getAllCustomPropertyDecls(): readonly CustomPropertyDependencyDecl[] {
    return [...this.customPropertyDecls.values()]
      .flat()
      .toSorted(compareCustomPropertyDependencyDecls);
  }

  getCustomPropertyDecls(name: string): readonly CustomPropertyDependencyDecl[] {
    return this.customPropertyDecls.get(name) ?? [];
  }

  getCustomPropertyRefs(name: string): readonly CustomPropertyDependencyRef[] {
    return this.customPropertyRefs.get(name) ?? [];
  }

  private rebuild(): void {
    this.incoming.clear();
    this.outgoing.clear();
    this.incomingSassModuleMembers.clear();
    this.incomingValueImports.clear();
    this.customPropertyDecls.clear();
    this.customPropertyRefs.clear();
    for (const edges of this.moduleEdges.values()) {
      for (const edge of edges.selectorEdges) {
        push(this.outgoing, selectorKey(edge.fromFilePath, edge.fromCanonicalName), {
          filePath: edge.toFilePath,
          canonicalName: edge.toCanonicalName,
          reason: edge.reason,
          ...(edge.range ? { range: edge.range } : {}),
        });
        push(this.incoming, selectorKey(edge.toFilePath, edge.toCanonicalName), {
          filePath: edge.fromFilePath,
          canonicalName: edge.fromCanonicalName,
          reason: edge.reason,
          ...(edge.range ? { range: edge.range } : {}),
        });
      }
      for (const edge of edges.sassModuleMemberEdges) {
        push(
          this.incomingSassModuleMembers,
          sassMemberKey(edge.toFilePath, edge.symbolKind, edge.name),
          {
            filePath: edge.filePath,
            namespace: edge.namespace,
            symbolKind: edge.symbolKind,
            name: edge.name,
            range: edge.range,
          },
        );
      }
      for (const edge of edges.valueImportEdges) {
        push(this.incomingValueImports, valueImportKey(edge.toFilePath, edge.importedName), {
          filePath: edge.filePath,
          name: edge.name,
          importedName: edge.importedName,
          ...(edge.importedNameRange ? { importedNameRange: edge.importedNameRange } : {}),
          from: edge.from,
          range: edge.range,
          ruleRange: edge.ruleRange,
        });
      }
      for (const decl of edges.customPropertyDecls) {
        push(this.customPropertyDecls, decl.name, decl);
      }
      for (const ref of edges.customPropertyRefs) {
        push(this.customPropertyRefs, ref.name, ref);
      }
    }
  }
}

function collectValueImportEdges(
  filePath: string,
  styleDocument: StyleDocumentHIR,
): readonly ValueImportDependencyEdge[] {
  return styleDocument.valueImports
    .filter((valueImport) => isRelativeSpecifier(valueImport.from))
    .map((valueImport) => {
      const edge: ValueImportDependencyEdge = {
        filePath,
        toFilePath: path.resolve(path.dirname(filePath), valueImport.from),
        name: valueImport.name,
        importedName: valueImport.importedName,
        from: valueImport.from,
        range: valueImport.range,
        ruleRange: valueImport.ruleRange,
      };
      if (valueImport.importedNameRange) {
        return Object.assign(edge, { importedNameRange: valueImport.importedNameRange });
      }
      return edge;
    });
}

function collectEdges(
  filePath: string,
  styleDocument: StyleDocumentHIR,
): readonly StyleDependencyEdge[] {
  const canonicalNames = new Set(
    styleDocument.selectors
      .filter((selector) => selector.viewKind === "canonical")
      .map((selector) => selector.canonicalName),
  );
  const edges: StyleDependencyEdge[] = [];

  for (const selector of styleDocument.selectors) {
    if (selector.viewKind !== "canonical") continue;
    for (const ref of selector.composes) {
      if (ref.fromGlobal) continue;
      if (!ref.from) {
        for (const className of ref.classNames) {
          if (!canonicalNames.has(className)) continue;
          const edge: StyleDependencyEdge = {
            fromFilePath: filePath,
            fromCanonicalName: selector.canonicalName,
            toFilePath: filePath,
            toCanonicalName: className,
            reason: "localComposes",
          };
          const range = composesClassRange(ref, className);
          edges.push(range ? { ...edge, range } : edge);
        }
        continue;
      }
      if (!isRelativeSpecifier(ref.from)) continue;
      const targetPath = path.resolve(path.dirname(filePath), ref.from);
      for (const className of ref.classNames) {
        const edge: StyleDependencyEdge = {
          fromFilePath: filePath,
          fromCanonicalName: selector.canonicalName,
          toFilePath: targetPath,
          toCanonicalName: className,
          reason: "crossFileComposes",
        };
        const range = composesClassRange(ref, className);
        edges.push(range ? { ...edge, range } : edge);
      }
    }
  }

  return edges;
}

function composesClassRange(
  ref: StyleDocumentHIR["selectors"][number]["composes"][number],
  className: string,
): Range | null {
  return ref.classTokens?.find((token) => token.className === className)?.range ?? null;
}

function collectSassModuleMemberEdges(
  filePath: string,
  styleDocument: StyleDocumentHIR,
  options: StyleDependencyRecordOptions,
): readonly SassModuleMemberDependencyEdge[] {
  if (
    !options.resolveSassModuleUseTargetFilePath &&
    !options.resolveSassModuleExportedSymbolTargets &&
    !options.resolveSassModuleExportedSymbolTargetFilePaths
  ) {
    return [];
  }

  const moduleUsesByNamespace = new Map(
    styleDocument.sassModuleUses
      .filter((moduleUse) => moduleUse.namespaceKind !== "wildcard" && moduleUse.namespace)
      .map((moduleUse) => [moduleUse.namespace!, moduleUse]),
  );
  const edges: SassModuleMemberDependencyEdge[] = [];

  for (const memberRef of styleDocument.sassModuleMemberRefs) {
    const moduleUse = moduleUsesByNamespace.get(memberRef.namespace);
    if (!moduleUse) continue;
    const targets = resolveSassModuleMemberTargets(
      moduleUse,
      memberRef.symbolKind,
      memberRef.name,
      options,
    );
    for (const target of targets) {
      edges.push({
        filePath,
        toFilePath: target.filePath,
        namespace: memberRef.namespace,
        symbolKind: memberRef.symbolKind,
        name: target.name,
        range: memberRef.range,
      });
    }
  }

  const wildcardModuleUses = styleDocument.sassModuleUses.filter(
    (moduleUse) => moduleUse.namespaceKind === "wildcard",
  );
  for (const symbol of styleDocument.sassSymbols) {
    if (findSassSymbolDeclForSymbol(styleDocument, symbol)) continue;
    for (const moduleUse of wildcardModuleUses) {
      const targets = resolveSassModuleMemberTargets(
        moduleUse,
        symbol.symbolKind,
        symbol.name,
        options,
      );
      for (const target of targets) {
        const edge = {
          filePath,
          toFilePath: target.filePath,
          namespace: "*",
          symbolKind: symbol.symbolKind,
          name: target.name,
          range: symbol.range,
        };
        if (edges.some((candidate) => sassModuleMemberEdgeEquals(candidate, edge))) continue;
        edges.push(edge);
      }
    }
  }

  return edges;
}

function collectCustomPropertyDecls(
  filePath: string,
  styleDocument: StyleDocumentHIR,
): readonly CustomPropertyDependencyDecl[] {
  return styleDocument.customPropertyDecls.map((decl) => ({
    filePath,
    name: decl.name,
    value: decl.value,
    range: decl.range,
    ruleRange: decl.ruleRange,
    context: decl.context,
  }));
}

function compareCustomPropertyDependencyDecls(
  left: CustomPropertyDependencyDecl,
  right: CustomPropertyDependencyDecl,
): number {
  const nameCompare = left.name.localeCompare(right.name);
  if (nameCompare !== 0) return nameCompare;
  const fileCompare = left.filePath.localeCompare(right.filePath);
  if (fileCompare !== 0) return fileCompare;
  if (left.range.start.line !== right.range.start.line) {
    return left.range.start.line - right.range.start.line;
  }
  return left.range.start.character - right.range.start.character;
}

function collectCustomPropertyRefs(
  filePath: string,
  styleDocument: StyleDocumentHIR,
): readonly CustomPropertyDependencyRef[] {
  return styleDocument.customPropertyRefs.map((ref) => ({
    filePath,
    name: ref.name,
    range: ref.range,
    context: ref.context,
  }));
}

function resolveSassModuleMemberTargets(
  moduleUse: SassModuleUseHIR,
  symbolKind: SassSymbolKind,
  name: string,
  options: StyleDependencyRecordOptions,
): readonly SassModuleExportedSymbolDependencyTarget[] {
  if (options.resolveSassModuleExportedSymbolTargets) {
    return options.resolveSassModuleExportedSymbolTargets(moduleUse, symbolKind, name);
  }
  if (options.resolveSassModuleExportedSymbolTargetFilePaths) {
    return options
      .resolveSassModuleExportedSymbolTargetFilePaths(moduleUse, symbolKind, name)
      .map((filePath) => ({ filePath, name }));
  }
  const targetFilePath = options.resolveSassModuleUseTargetFilePath?.(moduleUse);
  return targetFilePath ? [{ filePath: targetFilePath, name }] : [];
}

function sassModuleMemberEdgeEquals(
  a: SassModuleMemberDependencyEdge,
  b: SassModuleMemberDependencyEdge,
): boolean {
  return (
    a.filePath === b.filePath &&
    a.toFilePath === b.toFilePath &&
    a.namespace === b.namespace &&
    a.symbolKind === b.symbolKind &&
    a.name === b.name &&
    a.range.start.line === b.range.start.line &&
    a.range.start.character === b.range.start.character &&
    a.range.end.line === b.range.end.line &&
    a.range.end.character === b.range.end.character
  );
}

function selectorKey(filePath: string, canonicalName: string): string {
  return `${filePath}\u0000${canonicalName}`;
}

function sassMemberKey(filePath: string, symbolKind: SassSymbolKind, name: string): string {
  return `${filePath}\u0000${symbolKind}\u0000${name}`;
}

function valueImportKey(filePath: string, name: string): string {
  return `${filePath}\u0000${name}`;
}

function isRelativeSpecifier(specifier: string): boolean {
  return specifier.startsWith("./") || specifier.startsWith("../");
}

function push<T>(map: Map<string, readonly T[]>, key: string, value: T): void {
  const existing = map.get(key) ?? [];
  map.set(key, [...existing, value]);
}

function isWithinRoot(rootPath: string, candidatePath: string): boolean {
  const rel = path.relative(rootPath, candidatePath);
  return rel === "" || (!rel.startsWith("..") && !path.isAbsolute(rel));
}
