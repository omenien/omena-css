import type { Range } from "@omena/shared";
import type { StyleDocumentBuilder } from "../../engine-core-ts/src/core/scss/scss-index";
import {
  makeStyleDocumentHIR,
  type CustomPropertyDeclContextHIR,
  type CustomPropertyRefContextHIR,
  type NestedSelectorSafety,
  type SassModuleUseNamespaceKind,
  type SassSymbolKind,
  type SassSymbolResolution,
  type SassSymbolRole,
  type StyleDocumentHIR,
  type StylePreprocessorSymbolSyntax,
} from "../../engine-core-ts/src/core/hir/style-types";
import {
  SELECTED_QUERY_RUNNER_COMMANDS,
  isPackagedExtensionRuntime,
  runRustSelectedQueryBackendJson,
} from "./selected-query-backend";

export type OmenaParserStyleDocumentRunner = <T>(command: string, input: unknown) => T;

export function resolveRuntimeStyleDocumentBuilder(
  env: NodeJS.ProcessEnv = process.env,
  fileExists?: (filePath: string) => boolean,
): StyleDocumentBuilder | undefined {
  const value = env.OMENA_STYLE_DOCUMENT_BUILDER?.trim();
  if (value === "typescript-current") return undefined;
  if (value === "omena-parser") return buildStyleDocumentWithOmenaParser;
  if (!value && isPackagedExtensionRuntime(env, fileExists))
    return buildStyleDocumentWithOmenaParser;
  if (!value) return undefined;
  throw new Error(`Unknown OMENA_STYLE_DOCUMENT_BUILDER: ${value}`);
}

interface ParserPositionV0 {
  readonly line: number;
  readonly character: number;
}

interface ParserRangeV0 {
  readonly start: ParserPositionV0;
  readonly end: ParserPositionV0;
}

interface ParserAtRuleContextV0 {
  readonly name: string;
  readonly params: string;
  readonly range: ParserRangeV0;
}

interface ParserSelectorDefinitionFactV0 {
  readonly name: string;
  readonly sourceOrder: number;
  readonly range: ParserRangeV0;
  readonly ruleRange?: ParserRangeV0;
  readonly fullSelector?: string;
  readonly declarations?: string;
  readonly nestedSafetyKind: string;
  readonly bemSuffixParentName?: string;
}

interface ParserCustomPropertyDeclFactV0 {
  readonly name: string;
  readonly value: string;
  readonly sourceOrder: number;
  readonly range: ParserRangeV0;
  readonly ruleRange: ParserRangeV0;
  readonly selectorContexts: readonly string[];
  readonly wrapperAtRules?: readonly ParserAtRuleContextV0[];
  readonly underMedia: boolean;
  readonly underSupports: boolean;
  readonly underLayer: boolean;
}

interface ParserCustomPropertyRefFactV0 {
  readonly name: string;
  readonly sourceOrder: number;
  readonly range: ParserRangeV0;
  readonly selectorContexts: readonly string[];
  readonly wrapperAtRules?: readonly ParserAtRuleContextV0[];
  readonly underMedia: boolean;
  readonly underSupports: boolean;
  readonly underLayer: boolean;
}

interface ParserValueDeclFactV0 {
  readonly name: string;
  readonly value: string;
  readonly sourceOrder: number;
  readonly range: ParserRangeV0;
  readonly ruleRange: ParserRangeV0;
}

interface ParserValueImportFactV0 {
  readonly name: string;
  readonly importedName: string;
  readonly from: string;
  readonly sourceOrder: number;
  readonly range: ParserRangeV0;
  readonly importedNameRange?: ParserRangeV0;
  readonly ruleRange: ParserRangeV0;
}

interface ParserValueRefFactV0 {
  readonly name: string;
  readonly source: "declaration" | "valueDecl";
  readonly sourceOrder: number;
  readonly range: ParserRangeV0;
}

interface ParserKeyframesDeclFactV0 {
  readonly name: string;
  readonly sourceOrder: number;
  readonly range: ParserRangeV0;
  readonly ruleRange: ParserRangeV0;
}

interface ParserAnimationNameRefFactV0 {
  readonly name: string;
  readonly property: "animation" | "animation-name";
  readonly sourceOrder: number;
  readonly range: ParserRangeV0;
}

interface ParserSassSymbolDeclFactV0 {
  readonly symbolKind: string;
  readonly name: string;
  readonly range: ParserRangeV0;
}

interface ParserSassSelectorSymbolFactV0 {
  readonly selectorName: string;
  readonly symbolKind: string;
  readonly name: string;
  readonly namespace?: string | null;
  readonly role: string;
  readonly resolution: string;
  readonly range: ParserRangeV0;
}

interface ParserSassModuleUseFactV0 {
  readonly source: string;
  readonly namespaceKind: string;
  readonly namespace: string | null;
  readonly range: ParserRangeV0;
}

interface ParserSassModuleForwardFactV0 {
  readonly source: string;
  readonly prefix: string;
  readonly visibilityKind: "all" | "show" | "hide";
  readonly visibilityMembers: readonly {
    readonly name: string;
    readonly symbolKind: "variable" | null;
  }[];
  readonly range: ParserRangeV0;
  readonly ruleRange: ParserRangeV0;
}

interface ParserComposesEdgeFactV0 {
  readonly kind: "local" | "external" | "global";
  readonly ownerSelectorNames: readonly string[];
  readonly targetNames: readonly string[];
  readonly importSource?: string | null;
  readonly classTokens?: readonly {
    readonly className: string;
    readonly range: ParserRangeV0;
  }[];
  readonly range?: ParserRangeV0;
}

interface ParserCssModulesIntermediateV0 {
  readonly schemaVersion: "0";
  readonly language: "css" | "scss" | "sass" | "less";
  readonly selectors: {
    readonly definitionFacts: readonly ParserSelectorDefinitionFactV0[];
  };
  readonly values: {
    readonly declFacts: readonly ParserValueDeclFactV0[];
    readonly importFacts: readonly ParserValueImportFactV0[];
    readonly refFacts: readonly ParserValueRefFactV0[];
  };
  readonly customProperties: {
    readonly declFacts: readonly ParserCustomPropertyDeclFactV0[];
    readonly refFacts: readonly ParserCustomPropertyRefFactV0[];
  };
  readonly keyframes: {
    readonly declFacts: readonly ParserKeyframesDeclFactV0[];
    readonly refFacts: readonly ParserAnimationNameRefFactV0[];
  };
  readonly sass: {
    readonly symbolDeclFacts: readonly ParserSassSymbolDeclFactV0[];
    readonly selectorSymbolFacts: readonly ParserSassSelectorSymbolFactV0[];
    readonly moduleUseEdges: readonly ParserSassModuleUseFactV0[];
    readonly moduleForwardSources: readonly string[];
    readonly moduleForwardEdges?: readonly ParserSassModuleForwardFactV0[];
  };
  readonly composes: {
    readonly edges?: readonly ParserComposesEdgeFactV0[];
  };
}

export function buildStyleDocumentWithOmenaParser(
  filePath: string,
  content: string,
  runner: OmenaParserStyleDocumentRunner = runRustSelectedQueryBackendJson,
): StyleDocumentHIR {
  const intermediate = runner<ParserCssModulesIntermediateV0>(
    SELECTED_QUERY_RUNNER_COMMANDS.omenaParserCssModulesIntermediate,
    {
      styleSource: content,
      dialect: dialectForStylePath(filePath),
    },
  );
  const composesByOwner = collectComposesByOwner(intermediate.composes.edges ?? []);
  const syntax = syntaxForDialect(intermediate.language);

  return makeStyleDocumentHIR(
    filePath,
    intermediate.selectors.definitionFacts.map((fact) => {
      const bemSuffix =
        fact.nestedSafetyKind === "bemSuffixSafe" && fact.bemSuffixParentName
          ? {
              rawTokenRange: toRange(fact.range),
              rawToken: `&${fact.name.slice(fact.bemSuffixParentName.length)}`,
              parentResolvedName: fact.bemSuffixParentName,
            }
          : undefined;
      return {
        id: `selector:${fact.sourceOrder}:${fact.name}`,
        kind: "selector",
        range: toRange(fact.range),
        name: fact.name,
        canonicalName: fact.name,
        viewKind: "canonical",
        fullSelector: fact.fullSelector ?? `.${fact.name}`,
        declarations: fact.declarations ?? "",
        ruleRange: toRange(fact.ruleRange ?? fact.range),
        composes: composesByOwner.get(fact.name) ?? [],
        nestedSafety: nestedSafety(fact.nestedSafetyKind),
        ...(bemSuffix ? { bemSuffix } : {}),
      };
    }),
    intermediate.keyframes.declFacts.map((fact) => ({
      id: `keyframes:${fact.sourceOrder}:${fact.name}`,
      kind: "keyframes",
      range: toRange(fact.range),
      name: fact.name,
      ruleRange: toRange(fact.ruleRange),
    })),
    intermediate.keyframes.refFacts.map((fact) => ({
      id: `animation-name-ref:${fact.sourceOrder}:${fact.name}`,
      kind: "animationNameRef",
      range: toRange(fact.range),
      name: fact.name,
      property: fact.property,
    })),
    intermediate.values.declFacts.map((fact) => ({
      id: `value:${fact.sourceOrder}:${fact.name}`,
      kind: "valueDecl",
      range: toRange(fact.range),
      name: fact.name,
      value: fact.value,
      ruleRange: toRange(fact.ruleRange),
    })),
    intermediate.values.importFacts.map((fact) => ({
      id: `value-import:${fact.sourceOrder}:${fact.name}`,
      kind: "valueImport",
      range: toRange(fact.range),
      name: fact.name,
      importedName: fact.importedName,
      ...(fact.importedNameRange ? { importedNameRange: toRange(fact.importedNameRange) } : {}),
      from: fact.from,
      ruleRange: toRange(fact.ruleRange),
    })),
    intermediate.values.refFacts.map((fact) => ({
      id: `value-ref:${fact.source}:${fact.sourceOrder}:${fact.name}`,
      kind: "valueRef",
      range: toRange(fact.range),
      name: fact.name,
      source: fact.source,
    })),
    intermediate.customProperties.declFacts.map((fact) => ({
      id: `custom-property-decl:${fact.sourceOrder}:${fact.name}`,
      kind: "customPropertyDecl",
      range: toRange(fact.range),
      name: fact.name,
      value: fact.value,
      ruleRange: toRange(fact.ruleRange),
      context: customPropertyContext(fact),
    })),
    intermediate.customProperties.refFacts.map((fact) => ({
      id: `custom-property-ref:${fact.sourceOrder}:${fact.name}`,
      kind: "customPropertyRef",
      range: toRange(fact.range),
      name: fact.name,
      context: customPropertyContext(fact),
    })),
    intermediate.sass.selectorSymbolFacts
      .filter((fact) => !fact.namespace)
      .map((fact, index) =>
        withSyntax(
          {
            id: `sass-symbol:${index}:${fact.selectorName}:${fact.name}`,
            kind: "sassSymbol",
            selectorName: fact.selectorName,
            symbolKind: sassSymbolKind(fact.symbolKind),
            name: fact.name,
            role: sassSymbolRole(fact.role),
            resolution: sassSymbolResolution(fact.resolution),
            range: toRange(fact.range),
            ruleRange: toRange(fact.range),
          },
          syntax,
        ),
      ),
    intermediate.sass.symbolDeclFacts.map((fact, index) =>
      withSyntax(
        {
          id: `sass-symbol-decl:${index}:${fact.name}`,
          kind: "sassSymbolDecl",
          symbolKind: sassSymbolKind(fact.symbolKind),
          name: fact.name,
          range: toRange(fact.range),
          ruleRange: toRange(fact.range),
        },
        syntax,
      ),
    ),
    intermediate.sass.moduleUseEdges.map((fact, index) => ({
      id: `sass-module-use:${index}:${fact.source}`,
      kind: "sassModuleUse",
      source: fact.source,
      namespaceKind: sassModuleUseNamespaceKind(fact.namespaceKind),
      namespace: fact.namespace,
      range: toRange(fact.range),
      ruleRange: toRange(fact.range),
    })),
    intermediate.sass.selectorSymbolFacts
      .filter((fact): fact is ParserSassSelectorSymbolFactV0 & { readonly namespace: string } =>
        Boolean(fact.namespace),
      )
      .map((fact, index) => ({
        id: `sass-module-member-ref:${index}:${fact.selectorName}:${fact.namespace}.${fact.name}`,
        kind: "sassModuleMemberRef",
        selectorName: fact.selectorName,
        namespace: fact.namespace,
        symbolKind: sassSymbolKind(fact.symbolKind),
        name: fact.name,
        role: sassSymbolRole(fact.role),
        range: toRange(fact.range),
        ruleRange: toRange(fact.range),
      })),
    sassModuleForwardFacts(intermediate),
  );
}

function sassModuleForwardFacts(intermediate: ParserCssModulesIntermediateV0) {
  if (intermediate.sass.moduleForwardEdges) {
    return intermediate.sass.moduleForwardEdges.map((fact, index) => ({
      id: `sass-module-forward:${index}:${fact.source}`,
      kind: "sassModuleForward" as const,
      source: fact.source,
      prefix: fact.prefix,
      visibilityKind: fact.visibilityKind,
      visibilityMembers: [...fact.visibilityMembers],
      range: toRange(fact.range),
      ruleRange: toRange(fact.ruleRange),
    }));
  }

  return intermediate.sass.moduleForwardSources.map((source, index) => ({
    id: `sass-module-forward:${index}:${source}`,
    kind: "sassModuleForward" as const,
    source,
    prefix: "",
    visibilityKind: "all" as const,
    visibilityMembers: [],
    range: zeroRange(),
    ruleRange: zeroRange(),
  }));
}

function collectComposesByOwner(
  edges: readonly ParserComposesEdgeFactV0[],
): Map<string, StyleDocumentHIR["selectors"][number]["composes"]> {
  const byOwner = new Map<string, StyleDocumentHIR["selectors"][number]["composes"]>();
  for (const edge of edges) {
    for (const owner of edge.ownerSelectorNames) {
      const refs = byOwner.get(owner) ?? [];
      byOwner.set(owner, [
        ...refs,
        {
          classNames: [...edge.targetNames],
          ...(edge.range ? { range: toRange(edge.range) } : {}),
          ...(edge.classTokens && edge.classTokens.length > 0
            ? {
                classTokens: edge.classTokens.map((token) => ({
                  className: token.className,
                  range: toRange(token.range),
                })),
              }
            : {}),
          ...(edge.kind === "external" && edge.importSource ? { from: edge.importSource } : {}),
          ...(edge.kind === "global" ? { fromGlobal: true } : {}),
        },
      ]);
    }
  }
  return byOwner;
}

function customPropertyContext(
  fact: Pick<
    ParserCustomPropertyDeclFactV0,
    "selectorContexts" | "wrapperAtRules" | "underMedia" | "underSupports" | "underLayer"
  >,
): CustomPropertyDeclContextHIR & CustomPropertyRefContextHIR {
  const selectorText = fact.selectorContexts[0] ?? null;
  const wrapperAtRules =
    fact.wrapperAtRules?.map((wrapper) => ({
      name: wrapper.name,
      params: wrapper.params,
      range: toRange(wrapper.range),
    })) ??
    [
      fact.underMedia ? "media" : null,
      fact.underSupports ? "supports" : null,
      fact.underLayer ? "layer" : null,
    ]
      .filter((name): name is string => name !== null)
      .map((name) => ({ name, params: "", range: zeroRange() }));

  return {
    containerKind: selectorText ? "rule" : wrapperAtRules.length > 0 ? "atrule" : "root",
    selectorText,
    atRuleName: wrapperAtRules[0]?.name ?? null,
    atRuleParams: wrapperAtRules[0]?.params ?? null,
    wrapperAtRules,
  };
}

function dialectForStylePath(filePath: string): ParserCssModulesIntermediateV0["language"] {
  if (filePath.endsWith(".sass")) return "sass";
  if (filePath.endsWith(".scss")) return "scss";
  if (filePath.endsWith(".less")) return "less";
  return "css";
}

function syntaxForDialect(
  dialect: ParserCssModulesIntermediateV0["language"],
): StylePreprocessorSymbolSyntax | undefined {
  if (dialect === "less") return "less";
  if (dialect === "scss" || dialect === "sass") return "sass";
  return undefined;
}

function withSyntax<T extends object>(
  value: T,
  syntax: StylePreprocessorSymbolSyntax | undefined,
): T & { syntax?: StylePreprocessorSymbolSyntax } {
  if (!syntax) return value;
  return Object.assign(value, { syntax });
}

function nestedSafety(value: string): NestedSelectorSafety {
  if (value === "bemSuffixSafe" || value === "nestedUnsafe") return value;
  return "flat";
}

function sassSymbolKind(value: string): SassSymbolKind {
  if (value === "mixin" || value === "function") return value;
  return "variable";
}

function sassSymbolRole(value: string): SassSymbolRole {
  if (value === "include" || value === "call") return value;
  return "reference";
}

function sassSymbolResolution(value: string): SassSymbolResolution {
  if (value === "resolved" || value === "unresolved") return value;
  return "unresolved";
}

function sassModuleUseNamespaceKind(value: string): SassModuleUseNamespaceKind {
  if (value === "alias" || value === "wildcard") return value;
  return "default";
}

function toRange(range: ParserRangeV0): Range {
  return {
    start: { line: range.start.line, character: range.start.character },
    end: { line: range.end.line, character: range.end.character },
  };
}

function zeroRange(): Range {
  return {
    start: { line: 0, character: 0 },
    end: { line: 0, character: 0 },
  };
}
