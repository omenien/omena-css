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

export type SourceFrontendBackendKind = "typescript-current" | "rust-source-frontend";

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
  readonly missingStyleImports: readonly {
    readonly binding: string;
    readonly resolved: StyleImport;
  }[];
  readonly classnamesBindBindings: readonly string[];
}

const IMPORT_FROM_PATTERN =
  /\bimport\s+(?:type\s+)?(?:(?<defaultName>[A-Za-z_$][\w$]*)\s*,?\s*)?(?:\*\s+as\s+(?<namespaceName>[A-Za-z_$][\w$]*)\s*)?(?:\{[^}]*\}\s*)?from\s*["'](?<specifier>[^"']+)["']/g;

export function resolveSourceFrontendBackendKind(
  env: NodeJS.ProcessEnv = process.env,
): SourceFrontendBackendKind {
  const value = env.OMENA_SOURCE_FRONTEND_BACKEND?.trim();
  if (!value || value === "rust-source-frontend") return "rust-source-frontend";
  if (value === "typescript-current") return "typescript-current";
  throw new Error(`Unknown source frontend backend: ${value}`);
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
      const bindingIndex = JSON.parse(raw) as RustSourceBindingIndexV0;
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
      } else {
        missingStyleImports.push({ binding: localName, resolved: styleImport });
      }
    }
  }

  return {
    importedStyleBindings: importedStyleBindings.toSorted((a, b) =>
      `${a.binding}:${a.styleUri}`.localeCompare(`${b.binding}:${b.styleUri}`),
    ),
    missingStyleImports: missingStyleImports.toSorted((a, b) =>
      `${a.binding}:${a.resolved.absolutePath}`.localeCompare(
        `${b.binding}:${b.resolved.absolutePath}`,
      ),
    ),
    classnamesBindBindings: [...new Set(classnamesBindBindings)].toSorted(),
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
