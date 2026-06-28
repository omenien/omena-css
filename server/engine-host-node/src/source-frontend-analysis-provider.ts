import path from "node:path";
import { pathToFileURL } from "node:url";
import type { StyleImport } from "@omena/shared";
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
      const extras = readRustSourceSyntaxExtras({
        binding,
        filePath: input.filePath,
        content: input.content,
        sourceLanguage,
        importedStyleBindingsJson,
        classnamesBindBindingsJson,
      });
      return extras
        ? {
            ...projected,
            sourceDocument: {
              ...projected.sourceDocument,
              domainClassReferences: extras.domainClassReferences,
            },
            classValueUniverses: extras.classValueUniverses,
          }
        : projected;
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
  const classnamesBindBindings: string[] = [];

  for (const match of args.content.matchAll(IMPORT_FROM_PATTERN)) {
    const specifier = match.groups?.specifier;
    const localName = match.groups?.defaultName ?? match.groups?.namespaceName;
    if (!specifier || !localName) continue;

    if (specifier === "classnames/bind") {
      classnamesBindBindings.push(localName);
      continue;
    }

    const styleImport = resolveStyleImport(specifier, args);
    if (
      styleImport &&
      styleExtensions.some((extension) => styleImport.absolutePath.endsWith(extension))
    ) {
      importedStyleBindings.push({
        binding: localName,
        styleUri: pathToFileURL(styleImport.absolutePath).href,
      });
    }
  }

  return {
    importedStyleBindings: importedStyleBindings.toSorted((a, b) =>
      `${a.binding}:${a.styleUri}`.localeCompare(`${b.binding}:${b.styleUri}`),
    ),
    classnamesBindBindings: [...new Set(classnamesBindBindings)].toSorted(),
  };
}

function resolveStyleImport(
  specifier: string,
  args: {
    readonly filePath: string;
    readonly aliasResolver: AliasResolver;
    readonly fileExists: (path: string) => boolean;
  },
): StyleImport | null {
  const absolutePath = specifier.startsWith(".")
    ? path.resolve(path.dirname(args.filePath), specifier)
    : args.aliasResolver.resolve(specifier, args.fileExists, args.filePath);
  if (!absolutePath) return null;
  return { kind: "resolved", absolutePath };
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
