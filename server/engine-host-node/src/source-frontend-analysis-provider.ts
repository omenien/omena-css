import path from "node:path";
import { pathToFileURL } from "node:url";
import type { StyleImport } from "@omena/shared";
import { makeStyleImportBinding } from "../../engine-core-ts/src/core/hir/source-types";
import type {
  SourceFrontendAnalysisProviderInputV0,
  SourceFrontendAnalysisProviderResultV0,
} from "../../engine-core-ts/src/core/indexing/document-analysis-cache";
import type { AliasResolver } from "../../engine-core-ts/src/core/cx/alias-resolver";
import type { SourceLanguage } from "../../engine-core-ts/src/core/hir/shared-types";
import { getAllStyleExtensions } from "../../engine-core-ts/src/core/scss/lang-registry";
import {
  projectRustSourceBindingIndexV0,
  projectRustSourceSyntaxExtrasV0,
  type RustSourceBindingIndexV0,
  type RustSourceSyntaxIndexV0,
} from "../../engine-core-ts/src/core/source-frontend/rust-binding-index-projection";
import {
  loadDefaultOmenaNapiSourceFrontendBinding,
  type OmenaNapiSourceFrontendBinding,
} from "./omena-napi-source-frontend-binding";

export type SourceFrontendBackendKind = "rust-source-frontend";

export interface RustSourceFrontendAnalysisProviderOptions {
  readonly aliasResolver: () => AliasResolver;
  readonly fileExists: (path: string) => boolean;
  readonly loadBinding?: () => OmenaNapiSourceFrontendBinding | null | undefined;
}

interface SourceFrontendImportInputsV0 {
  readonly importedStyleBindings: readonly {
    readonly binding: string;
    readonly styleUri: string;
  }[];
  readonly styleImportFallbacks: readonly SourceFrontendStyleImportFallbackV0[];
  readonly classUtilFallbacks: readonly SourceFrontendClassUtilFallbackV0[];
  readonly classnamesBindUtilityFallbacks: readonly SourceFrontendClassnamesBindUtilityFallbackV0[];
  readonly missingStyleImports: readonly {
    readonly binding: string;
    readonly resolved: StyleImport;
  }[];
  readonly classnamesBindBindings: readonly string[];
}

interface SourceFrontendStyleImportFallbackV0 {
  readonly binding: string;
  readonly styleUri: string;
  readonly importPath: string;
  readonly byteSpan: RustSourceBindingIndexV0["bindingDecls"][number]["byteSpan"];
}

interface SourceFrontendClassUtilFallbackV0 {
  readonly localName: string;
  readonly importPath: string;
  readonly byteSpan: RustSourceBindingIndexV0["bindingDecls"][number]["byteSpan"];
}

interface SourceFrontendClassnamesBindUtilityFallbackV0 {
  readonly localName: string;
  readonly stylesLocalName: string;
  readonly styleUri: string;
  readonly classnamesImportName: string;
  readonly byteSpan: RustSourceBindingIndexV0["bindingDecls"][number]["byteSpan"];
}

const IMPORT_FROM_PATTERN =
  /\bimport\s+(?:type\s+)?(?:(?<defaultName>[A-Za-z_$][\w$]*)\s*,?\s*)?(?:\*\s+as\s+(?<namespaceName>[A-Za-z_$][\w$]*)\s*)?(?:\{[^}]*\}\s*)?from\s*["'](?<specifier>[^"']+)["']/g;
const CLASSNAMES_BIND_INITIALIZER_PATTERN =
  /\b(?:const|let|var)\s+(?<localName>[A-Za-z_$][\w$]*)\s*=\s*(?<classnamesImportName>[A-Za-z_$][\w$]*)\.bind\(\s*(?<stylesLocalName>[A-Za-z_$][\w$]*)\s*\)/g;

export function resolveSourceFrontendBackendKind(
  env: NodeJS.ProcessEnv = process.env,
): SourceFrontendBackendKind {
  const value = env.OMENA_SOURCE_FRONTEND_BACKEND?.trim();
  if (!value || value === "rust-source-frontend") return "rust-source-frontend";
  throw new Error(`Unknown source frontend backend: ${value}`);
}

export function createRequiredRustSourceFrontendAnalysisProvider(
  options: RustSourceFrontendAnalysisProviderOptions,
): (input: SourceFrontendAnalysisProviderInputV0) => SourceFrontendAnalysisProviderResultV0 | null {
  const optionalProvider = createDefaultRustSourceFrontendAnalysisProvider(options);
  return (input) => {
    if (!sourceLanguageForPath(input.filePath)) return null;
    const result = optionalProvider(input);
    if (result) return result;
    throw new Error(
      `Rust source frontend analysis is required for ${input.filePath}. Build or install @omena/napi before analyzing supported source files.`,
    );
  };
}

export function createDefaultRustSourceFrontendAnalysisProvider(
  options: RustSourceFrontendAnalysisProviderOptions,
): (input: SourceFrontendAnalysisProviderInputV0) => SourceFrontendAnalysisProviderResultV0 | null {
  const loadBinding = options.loadBinding ?? loadDefaultOmenaNapiSourceFrontendBinding;
  return (input) => {
    const sourceLanguage = sourceLanguageForPath(input.filePath);
    if (!sourceLanguage) return null;
    const binding = loadBinding();
    if (!binding) return null;
    const readBinding = binding.readSourceBindingIndexJson;
    if (typeof readBinding !== "function") return null;

    const importInputs = collectSourceFrontendImportInputs({
      content: input.content,
      filePath: input.filePath,
      aliasResolver: options.aliasResolver(),
      fileExists: options.fileExists,
    });
    const importedStyleBindingsJson = JSON.stringify(importInputs.importedStyleBindings);
    const classnamesBindBindingsJson = JSON.stringify(importInputs.classnamesBindBindings);
    try {
      const raw = readBinding(
        input.filePath,
        input.content,
        sourceLanguage,
        importedStyleBindingsJson,
        classnamesBindBindingsJson,
      );
      if (!raw) return null;
      const bindingIndex = bindingIndexWithImportFallbacks(
        JSON.parse(raw) as RustSourceBindingIndexV0,
        input.content,
        importInputs,
      );
      const projected = projectRustSourceBindingIndexV0({
        filePath: input.filePath,
        source: input.content,
        language: sourceLanguage,
        index: bindingIndex,
      });
      if (projected.sourceBinder.scopes.length === 0 && projected.sourceBinder.decls.length === 0) {
        return null;
      }
      const extras = readRustSourceSyntaxExtras({
        binding,
        filePath: input.filePath,
        content: input.content,
        sourceLanguage,
        importedStyleBindingsJson,
        classnamesBindBindingsJson,
      });
      const sourceDocument = sourceDocumentWithMissingStyleImports(
        {
          ...projected.sourceDocument,
          ...(extras ? { domainClassReferences: extras.domainClassReferences } : {}),
        },
        importInputs.missingStyleImports,
      );
      return {
        ...projected,
        sourceDocument,
        ...(extras ? { classValueUniverses: extras.classValueUniverses } : {}),
      };
    } catch {
      return null;
    }
  };
}

function readRustSourceSyntaxExtras(args: {
  readonly binding: OmenaNapiSourceFrontendBinding;
  readonly filePath: string;
  readonly content: string;
  readonly sourceLanguage: SourceLanguage;
  readonly importedStyleBindingsJson: string;
  readonly classnamesBindBindingsJson: string;
}) {
  const readSyntax = args.binding.readSourceSyntaxIndexJson;
  if (typeof readSyntax !== "function") return null;
  const raw = readSyntax(
    args.filePath,
    args.content,
    args.sourceLanguage,
    args.importedStyleBindingsJson,
    args.classnamesBindBindingsJson,
  );
  if (!raw) return null;
  return projectRustSourceSyntaxExtrasV0({
    filePath: args.filePath,
    source: args.content,
    index: JSON.parse(raw) as RustSourceSyntaxIndexV0,
  });
}

function collectSourceFrontendImportInputs(args: {
  readonly content: string;
  readonly filePath: string;
  readonly aliasResolver: AliasResolver;
  readonly fileExists: (path: string) => boolean;
}): SourceFrontendImportInputsV0 {
  const styleExtensions = getAllStyleExtensions();
  const importedStyleBindings: { binding: string; styleUri: string }[] = [];
  const styleImportFallbacks: SourceFrontendStyleImportFallbackV0[] = [];
  const classUtilFallbacks: SourceFrontendClassUtilFallbackV0[] = [];
  const missingStyleImports: { binding: string; resolved: StyleImport }[] = [];
  const classnamesBindBindings: string[] = [];

  for (const match of args.content.matchAll(IMPORT_FROM_PATTERN)) {
    const specifier = match.groups?.specifier;
    const localName = match.groups?.defaultName ?? match.groups?.namespaceName;
    if (!specifier || !localName) continue;

    if (specifier === "classnames/bind") {
      classnamesBindBindings.push(localName);
      continue;
    }
    if (isClassUtilityImportPath(specifier)) {
      classUtilFallbacks.push({
        localName,
        importPath: specifier,
        byteSpan: localNameByteSpan(args.content, match, localName),
      });
      continue;
    }

    const styleImport = resolveStyleImport(specifier, match, args);
    if (
      styleImport &&
      styleExtensions.some((extension) => styleImport.absolutePath.endsWith(extension))
    ) {
      if (styleImport.kind === "resolved") {
        importedStyleBindings.push({
          binding: localName,
          styleUri: pathToFileURL(styleImport.absolutePath).href,
        });
        styleImportFallbacks.push({
          binding: localName,
          styleUri: pathToFileURL(styleImport.absolutePath).href,
          importPath: specifier,
          byteSpan: localNameByteSpan(args.content, match, localName),
        });
      } else {
        missingStyleImports.push({ binding: localName, resolved: styleImport });
      }
    }
  }

  const classnamesBindUtilityFallbacks = collectClassnamesBindUtilityFallbacks(
    args.content,
    styleImportFallbacks,
    classnamesBindBindings,
  );

  return {
    importedStyleBindings: importedStyleBindings.toSorted((a, b) =>
      `${a.binding}:${a.styleUri}`.localeCompare(`${b.binding}:${b.styleUri}`),
    ),
    styleImportFallbacks: styleImportFallbacks.toSorted((a, b) =>
      `${a.binding}:${a.styleUri}`.localeCompare(`${b.binding}:${b.styleUri}`),
    ),
    classUtilFallbacks: classUtilFallbacks.toSorted((a, b) =>
      `${a.localName}:${a.importPath}`.localeCompare(`${b.localName}:${b.importPath}`),
    ),
    classnamesBindUtilityFallbacks: classnamesBindUtilityFallbacks.toSorted((a, b) =>
      `${a.localName}:${a.stylesLocalName}`.localeCompare(`${b.localName}:${b.stylesLocalName}`),
    ),
    missingStyleImports: missingStyleImports.toSorted((a, b) =>
      `${a.binding}:${a.resolved.absolutePath}`.localeCompare(
        `${b.binding}:${b.resolved.absolutePath}`,
      ),
    ),
    classnamesBindBindings: [...new Set(classnamesBindBindings)].toSorted(),
  };
}

function bindingIndexWithImportFallbacks(
  index: RustSourceBindingIndexV0,
  source: string,
  importInputs: SourceFrontendImportInputsV0,
): RustSourceBindingIndexV0 {
  const sourceFileScope = {
    kind: "sourceFile" as const,
    byteSpan: { start: 0, end: Buffer.byteLength(source, "utf8") },
  };
  const bindingScopes = index.bindingScopes.length > 0 ? index.bindingScopes : [sourceFileScope];
  const bindingDecls = [
    ...index.bindingDecls,
    ...importInputs.styleImportFallbacks
      .filter((fallback) => !hasDecl(index.bindingDecls, fallback.binding, "import"))
      .map((fallback) => ({
        kind: "import" as const,
        name: fallback.binding,
        importPath: fallback.importPath,
        byteSpan: fallback.byteSpan,
      })),
    ...importInputs.classUtilFallbacks
      .filter((fallback) => !hasDecl(index.bindingDecls, fallback.localName, "import"))
      .map((fallback) => ({
        kind: "import" as const,
        name: fallback.localName,
        importPath: fallback.importPath,
        byteSpan: fallback.byteSpan,
      })),
    ...importInputs.classnamesBindUtilityFallbacks
      .filter((fallback) => !hasDecl(index.bindingDecls, fallback.localName, "localVar"))
      .map((fallback) => ({
        kind: "localVar" as const,
        name: fallback.localName,
        byteSpan: fallback.byteSpan,
      })),
  ];
  return {
    ...index,
    bindingScopes,
    bindingDecls,
    styleImportBindings: [
      ...index.styleImportBindings,
      ...importInputs.styleImportFallbacks
        .filter(
          (fallback) =>
            !index.styleImportBindings.some(
              (entry) =>
                entry.localName === fallback.binding && entry.styleUri === fallback.styleUri,
            ),
        )
        .map((fallback) => ({ localName: fallback.binding, styleUri: fallback.styleUri })),
    ],
    declaresStyleImports: [
      ...index.declaresStyleImports,
      ...importInputs.styleImportFallbacks
        .filter(
          (fallback) =>
            !index.declaresStyleImports.some(
              (entry) =>
                entry.declName === fallback.binding &&
                entry.stylesLocalName === fallback.binding &&
                entry.styleUri === fallback.styleUri,
            ),
        )
        .map((fallback) => ({
          declName: fallback.binding,
          stylesLocalName: fallback.binding,
          styleUri: fallback.styleUri,
        })),
    ],
    styleImportResolvesModules: [
      ...index.styleImportResolvesModules,
      ...importInputs.styleImportFallbacks
        .filter(
          (fallback) =>
            !index.styleImportResolvesModules.some(
              (entry) =>
                entry.stylesLocalName === fallback.binding && entry.styleUri === fallback.styleUri,
            ),
        )
        .map((fallback) => ({
          stylesLocalName: fallback.binding,
          styleUri: fallback.styleUri,
        })),
    ],
    scopeContainsDecls: [
      ...index.scopeContainsDecls,
      ...bindingDecls
        .filter((decl) => !hasScopeContainsDecl(index.scopeContainsDecls, decl.name, decl.kind))
        .map((decl) => {
          const importPath = "importPath" in decl ? decl.importPath : undefined;
          if (importPath) {
            return {
              scopeKind: "sourceFile" as const,
              scopeByteSpan: sourceFileScope.byteSpan,
              declKind: decl.kind,
              declName: decl.name,
              declByteSpan: decl.byteSpan,
              importPath,
            };
          }
          return {
            scopeKind: "sourceFile" as const,
            scopeByteSpan: sourceFileScope.byteSpan,
            declKind: decl.kind,
            declName: decl.name,
            declByteSpan: decl.byteSpan,
          };
        }),
    ],
    classUtilBindings: [
      ...index.classUtilBindings,
      ...importInputs.classUtilFallbacks
        .filter(
          (fallback) =>
            !index.classUtilBindings.some((entry) => entry.localName === fallback.localName),
        )
        .map((fallback) => ({ localName: fallback.localName })),
    ],
    classnamesBindUtilityBindings: [
      ...index.classnamesBindUtilityBindings,
      ...importInputs.classnamesBindUtilityFallbacks.filter(
        (fallback) =>
          !index.classnamesBindUtilityBindings.some(
            (entry) => entry.localName === fallback.localName,
          ),
      ),
    ],
    declaresUtilityBindings: [
      ...index.declaresUtilityBindings,
      ...importInputs.classUtilFallbacks
        .filter(
          (fallback) =>
            !index.declaresUtilityBindings.some(
              (entry) => entry.utilityLocalName === fallback.localName,
            ),
        )
        .map((fallback) => ({
          declName: fallback.localName,
          utilityLocalName: fallback.localName,
          utilityKind: "classUtil" as const,
        })),
      ...importInputs.classnamesBindUtilityFallbacks
        .filter(
          (fallback) =>
            !index.declaresUtilityBindings.some(
              (entry) => entry.utilityLocalName === fallback.localName,
            ),
        )
        .map((fallback) => ({
          declName: fallback.localName,
          utilityLocalName: fallback.localName,
          utilityKind: "classnamesBind" as const,
        })),
    ],
    utilityUsesStyleImports: [
      ...index.utilityUsesStyleImports,
      ...importInputs.classnamesBindUtilityFallbacks
        .filter(
          (fallback) =>
            !index.utilityUsesStyleImports.some(
              (entry) =>
                entry.utilityLocalName === fallback.localName &&
                entry.stylesLocalName === fallback.stylesLocalName,
            ),
        )
        .map((fallback) => ({
          utilityLocalName: fallback.localName,
          stylesLocalName: fallback.stylesLocalName,
          styleUri: fallback.styleUri,
        })),
    ],
  };
}

function hasDecl(
  decls: RustSourceBindingIndexV0["bindingDecls"],
  name: string,
  kind: RustSourceBindingIndexV0["bindingDecls"][number]["kind"],
): boolean {
  return decls.some((decl) => decl.name === name && decl.kind === kind);
}

function hasScopeContainsDecl(
  edges: RustSourceBindingIndexV0["scopeContainsDecls"],
  name: string,
  kind: RustSourceBindingIndexV0["scopeContainsDecls"][number]["declKind"],
): boolean {
  return edges.some((edge) => edge.declName === name && edge.declKind === kind);
}

function collectClassnamesBindUtilityFallbacks(
  source: string,
  styleImportFallbacks: readonly SourceFrontendStyleImportFallbackV0[],
  classnamesBindBindings: readonly string[],
): readonly SourceFrontendClassnamesBindUtilityFallbackV0[] {
  const classnamesImports = new Set(classnamesBindBindings);
  const styleImportsByLocalName = new Map(
    styleImportFallbacks.map((fallback) => [fallback.binding, fallback] as const),
  );
  const fallbacks: SourceFrontendClassnamesBindUtilityFallbackV0[] = [];
  for (const match of source.matchAll(CLASSNAMES_BIND_INITIALIZER_PATTERN)) {
    const localName = match.groups?.localName;
    const classnamesImportName = match.groups?.classnamesImportName;
    const stylesLocalName = match.groups?.stylesLocalName;
    if (!localName || !classnamesImportName || !stylesLocalName) continue;
    if (!classnamesImports.has(classnamesImportName)) continue;
    const styleImport = styleImportsByLocalName.get(stylesLocalName);
    if (!styleImport) continue;
    fallbacks.push({
      localName,
      stylesLocalName,
      styleUri: styleImport.styleUri,
      classnamesImportName,
      byteSpan: localNameByteSpan(source, match, localName),
    });
  }
  return fallbacks;
}

function isClassUtilityImportPath(specifier: string): boolean {
  return specifier === "clsx" || specifier === "clsx/lite" || specifier === "classnames";
}

function localNameByteSpan(
  source: string,
  match: RegExpMatchArray,
  localName: string,
): RustSourceBindingIndexV0["bindingDecls"][number]["byteSpan"] {
  const matchStart = match.index ?? 0;
  const localStartInMatch = match[0].indexOf(localName);
  const start = matchStart + Math.max(localStartInMatch, 0);
  const end = start + localName.length;
  return {
    start: Buffer.byteLength(source.slice(0, start), "utf8"),
    end: Buffer.byteLength(source.slice(0, end), "utf8"),
  };
}

function resolveStyleImport(
  specifier: string,
  match: RegExpMatchArray,
  args: {
    readonly filePath: string;
    readonly content: string;
    readonly aliasResolver: AliasResolver;
    readonly fileExists: (path: string) => boolean;
  },
): StyleImport | null {
  const absolutePath = specifier.startsWith(".")
    ? path.resolve(path.dirname(args.filePath), specifier)
    : args.aliasResolver.resolve(specifier, args.fileExists, args.filePath);
  if (!absolutePath) return null;
  if (args.fileExists(absolutePath)) {
    return { kind: "resolved", absolutePath };
  }
  return {
    kind: "missing",
    absolutePath,
    specifier,
    range: rangeForSpecifierMatch(args.content, match, specifier),
  };
}

function sourceDocumentWithMissingStyleImports(
  sourceDocument: SourceFrontendAnalysisProviderResultV0["sourceDocument"],
  missingStyleImports: SourceFrontendImportInputsV0["missingStyleImports"],
): SourceFrontendAnalysisProviderResultV0["sourceDocument"] {
  if (missingStyleImports.length === 0) return sourceDocument;
  const existingLocals = new Set(sourceDocument.styleImports.map((entry) => entry.localName));
  const additions = missingStyleImports
    .filter((entry) => !existingLocals.has(entry.binding))
    .map((entry, index) =>
      makeStyleImportBinding(
        `rust-missing-style-import:${entry.binding}:${entry.resolved.absolutePath}:${index}`,
        entry.binding,
        `rust-missing-style-import-decl:${entry.binding}:${index}`,
        entry.resolved,
      ),
    );
  if (additions.length === 0) return sourceDocument;
  return {
    ...sourceDocument,
    styleImports: [...sourceDocument.styleImports, ...additions].toSorted(
      (a, b) => a.localName.localeCompare(b.localName) || a.id.localeCompare(b.id),
    ),
  };
}

function rangeForSpecifierMatch(source: string, match: RegExpMatchArray, specifier: string) {
  const groups = match.groups as { readonly specifier?: string } | undefined;
  const matchStart = match.index ?? 0;
  const specifierStartInMatch =
    groups?.specifier !== undefined
      ? match[0].indexOf(groups.specifier)
      : match[0].indexOf(specifier);
  const specifierStart = matchStart + Math.max(specifierStartInMatch, 0);
  const specifierEnd = specifierStart + specifier.length;
  return {
    start: positionAtOffset(source, specifierStart),
    end: positionAtOffset(source, specifierEnd),
  };
}

function positionAtOffset(source: string, offset: number) {
  let line = 0;
  let lineStart = 0;
  for (let index = 0; index < offset; index += 1) {
    if (source.charCodeAt(index) === 10) {
      line += 1;
      lineStart = index + 1;
    }
  }
  return { line, character: offset - lineStart };
}

function sourceLanguageForPath(sourcePath: string): SourceLanguage | null {
  const normalized = sourcePath.toLowerCase();
  if (normalized.endsWith(".tsx")) return "typescriptreact";
  if (normalized.endsWith(".ts") || normalized.endsWith(".mts") || normalized.endsWith(".cts")) {
    return "typescript";
  }
  if (normalized.endsWith(".jsx")) return "javascriptreact";
  if (normalized.endsWith(".js") || normalized.endsWith(".mjs") || normalized.endsWith(".cjs")) {
    return "javascript";
  }
  if (normalized.endsWith(".vue")) return "vue";
  return null;
}
