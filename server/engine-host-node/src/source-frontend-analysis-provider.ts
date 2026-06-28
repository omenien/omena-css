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
  type RustSourceBindingIndexV0,
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
  if (!value || value === "typescript-current") return "typescript-current";
  if (value === "rust-source-frontend") return "rust-source-frontend";
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
    const read = binding?.readSourceBindingIndexJson;
    if (typeof read !== "function") return null;

    const importInputs = collectSourceFrontendImportInputs({
      content: input.content,
      filePath: input.filePath,
      aliasResolver: options.aliasResolver(),
      fileExists: options.fileExists,
    });
    try {
      const raw = read(
        input.filePath,
        input.content,
        sourceLanguage,
        JSON.stringify(importInputs.importedStyleBindings),
        JSON.stringify(importInputs.classnamesBindBindings),
      );
      if (!raw) return null;
      const index = JSON.parse(raw) as RustSourceBindingIndexV0;
      return projectRustSourceBindingIndexV0({
        filePath: input.filePath,
        source: input.content,
        language: sourceLanguage,
        index,
      });
    } catch {
      return null;
    }
  };
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
