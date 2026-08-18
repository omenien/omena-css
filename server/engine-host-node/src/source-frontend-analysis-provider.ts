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
import { utf16OffsetAtUtf8ByteOffset } from "../../engine-core-ts/src/core/source-frontend/source-text-offsets";
import {
  loadDefaultOmenaNapiSourceFrontendBinding,
  type OmenaNapiSourceFrontendBinding,
  type OmenaNapiSourceImportDeclarationSummaryV0,
  type OmenaNapiSourceImportDeclarationV0,
} from "./omena-napi-source-frontend-binding";

export type SourceFrontendBackendKind = "rust-source-frontend";

export type SourceFrontendAnalysisOutcomeKindV0 =
  | "success"
  | "unsupportedLanguage"
  | "bindingUnavailable"
  | "napiFailure"
  | "jsonFailure"
  | "projectionFailure";

export type SourceFrontendAnalysisOutcomeV0 =
  | {
      readonly kind: "success";
      readonly result: SourceFrontendAnalysisProviderResultV0;
    }
  | {
      readonly kind: Exclude<SourceFrontendAnalysisOutcomeKindV0, "success">;
      readonly detail: string;
    };

export interface SourceFrontendAnalysisOutcomeCounterSnapshotV0 {
  readonly total: number;
  readonly success: number;
  readonly unsupportedLanguage: number;
  readonly bindingUnavailable: number;
  readonly napiFailure: number;
  readonly jsonFailure: number;
  readonly projectionFailure: number;
}

export class SourceFrontendAnalysisOutcomeCountersV0 {
  private readonly counts = new Map<SourceFrontendAnalysisOutcomeKindV0, number>();

  record(kind: SourceFrontendAnalysisOutcomeKindV0): void {
    this.counts.set(kind, (this.counts.get(kind) ?? 0) + 1);
  }

  snapshot(): SourceFrontendAnalysisOutcomeCounterSnapshotV0 {
    const success = this.counts.get("success") ?? 0;
    const unsupportedLanguage = this.counts.get("unsupportedLanguage") ?? 0;
    const bindingUnavailable = this.counts.get("bindingUnavailable") ?? 0;
    const napiFailure = this.counts.get("napiFailure") ?? 0;
    const jsonFailure = this.counts.get("jsonFailure") ?? 0;
    const projectionFailure = this.counts.get("projectionFailure") ?? 0;
    return {
      total:
        success +
        unsupportedLanguage +
        bindingUnavailable +
        napiFailure +
        jsonFailure +
        projectionFailure,
      success,
      unsupportedLanguage,
      bindingUnavailable,
      napiFailure,
      jsonFailure,
      projectionFailure,
    };
  }
}

export class SourceFrontendAnalysisError extends Error {
  constructor(
    readonly outcomeKind: Exclude<SourceFrontendAnalysisOutcomeKindV0, "success">,
    readonly sourcePath: string,
    detail: string,
  ) {
    super(`Rust source frontend ${outcomeKind} for ${sourcePath}: ${detail}`);
    this.name = "SourceFrontendAnalysisError";
  }
}

export interface RustSourceFrontendAnalysisProviderOptions {
  readonly aliasResolver: () => AliasResolver;
  readonly fileExists: (path: string) => boolean;
  readonly loadBinding?: () => OmenaNapiSourceFrontendBinding | null | undefined;
  readonly outcomeCounters?: SourceFrontendAnalysisOutcomeCountersV0;
}

interface SourceFrontendImportInputsV0 {
  readonly styleImportResolutions: readonly {
    readonly declarationId: string;
    readonly styleUri: string;
  }[];
  readonly missingStyleImports: readonly {
    readonly declarationId: string;
    readonly binding: string;
    readonly resolved: StyleImport;
  }[];
}

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
  const provider = createDefaultRustSourceFrontendAnalysisProvider(options);
  return (input) => {
    const outcome = provider(input);
    if (outcome.kind === "success") return outcome.result;
    if (outcome.kind === "unsupportedLanguage") return null;
    throw new SourceFrontendAnalysisError(outcome.kind, input.filePath, outcome.detail);
  };
}

export function createDefaultRustSourceFrontendAnalysisProvider(
  options: RustSourceFrontendAnalysisProviderOptions,
): (input: SourceFrontendAnalysisProviderInputV0) => SourceFrontendAnalysisOutcomeV0 {
  const loadBinding = options.loadBinding ?? loadDefaultOmenaNapiSourceFrontendBinding;
  const emit = (outcome: SourceFrontendAnalysisOutcomeV0): SourceFrontendAnalysisOutcomeV0 => {
    options.outcomeCounters?.record(outcome.kind);
    return outcome;
  };

  return (input) => {
    const sourceLanguage = sourceLanguageForPath(input.filePath);
    if (!sourceLanguage) {
      return emit({ kind: "unsupportedLanguage", detail: "unsupported source extension" });
    }

    let binding: OmenaNapiSourceFrontendBinding | null | undefined;
    try {
      binding = loadBinding();
    } catch (error) {
      return emit({ kind: "bindingUnavailable", detail: errorDetail(error) });
    }
    const readBinding = binding?.readSourceBindingIndexJson;
    const readImports = binding?.readSourceImportDeclarations;
    if (!binding || typeof readBinding !== "function" || typeof readImports !== "function") {
      return emit({
        kind: "bindingUnavailable",
        detail: "required @omena/napi source-frontend exports are unavailable",
      });
    }

    let importSummaryValue: OmenaNapiSourceImportDeclarationSummaryV0 | null | undefined;
    try {
      importSummaryValue = readImports(input.filePath, input.content, sourceLanguage);
    } catch (error) {
      return emit({ kind: "napiFailure", detail: errorDetail(error) });
    }
    if (!importSummaryValue) {
      return emit({ kind: "napiFailure", detail: "import declaration export returned no value" });
    }
    if (!isRustSourceImportDeclarationSummaryV0(importSummaryValue)) {
      return emit({
        kind: "projectionFailure",
        detail: "import declaration result does not match the Rust summary shape",
      });
    }

    let importInputs: SourceFrontendImportInputsV0;
    try {
      importInputs = collectSourceFrontendImportInputs({
        imports: importSummaryValue.imports,
        content: input.content,
        filePath: input.filePath,
        aliasResolver: options.aliasResolver(),
        fileExists: options.fileExists,
      });
    } catch (error) {
      return emit({ kind: "projectionFailure", detail: errorDetail(error) });
    }
    const styleImportResolutionsJson = JSON.stringify(importInputs.styleImportResolutions);

    let bindingIndexRaw: string | null | undefined;
    try {
      bindingIndexRaw = readBinding(
        input.filePath,
        input.content,
        sourceLanguage,
        styleImportResolutionsJson,
      );
    } catch (error) {
      return emit({ kind: "napiFailure", detail: errorDetail(error) });
    }
    if (!bindingIndexRaw) {
      return emit({ kind: "napiFailure", detail: "binding index export returned no JSON" });
    }

    let bindingIndex: RustSourceBindingIndexV0;
    try {
      bindingIndex = JSON.parse(bindingIndexRaw) as RustSourceBindingIndexV0;
    } catch (error) {
      return emit({ kind: "jsonFailure", detail: errorDetail(error) });
    }

    let projected: ReturnType<typeof projectRustSourceBindingIndexV0>;
    try {
      projected = projectRustSourceBindingIndexV0({
        filePath: input.filePath,
        source: input.content,
        language: sourceLanguage,
        index: bindingIndex,
      });
    } catch (error) {
      return emit({ kind: "projectionFailure", detail: errorDetail(error) });
    }

    let extras: ReturnType<typeof projectRustSourceSyntaxExtrasV0> | null = null;
    const readSyntax = binding.readSourceSyntaxIndexJson;
    if (typeof readSyntax === "function") {
      let syntaxIndexRaw: string | null | undefined;
      try {
        syntaxIndexRaw = readSyntax(
          input.filePath,
          input.content,
          sourceLanguage,
          styleImportResolutionsJson,
        );
      } catch (error) {
        return emit({ kind: "napiFailure", detail: errorDetail(error) });
      }
      if (!syntaxIndexRaw) {
        return emit({ kind: "napiFailure", detail: "syntax index export returned no JSON" });
      }
      let syntaxIndex: RustSourceSyntaxIndexV0;
      try {
        syntaxIndex = JSON.parse(syntaxIndexRaw) as RustSourceSyntaxIndexV0;
      } catch (error) {
        return emit({ kind: "jsonFailure", detail: errorDetail(error) });
      }
      try {
        extras = projectRustSourceSyntaxExtrasV0({
          filePath: input.filePath,
          source: input.content,
          index: syntaxIndex,
        });
      } catch (error) {
        return emit({ kind: "projectionFailure", detail: errorDetail(error) });
      }
    }

    try {
      const sourceDocument = sourceDocumentWithMissingStyleImports(
        {
          ...projected.sourceDocument,
          ...(extras ? { domainClassReferences: extras.domainClassReferences } : {}),
        },
        importInputs.missingStyleImports,
      );
      return emit({
        kind: "success",
        result: {
          ...projected,
          sourceDocument,
          ...(extras ? { classValueUniverses: extras.classValueUniverses } : {}),
        },
      });
    } catch (error) {
      return emit({ kind: "projectionFailure", detail: errorDetail(error) });
    }
  };
}

function errorDetail(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function isRustSourceImportDeclarationSummaryV0(
  value: unknown,
): value is OmenaNapiSourceImportDeclarationSummaryV0 {
  if (!value || typeof value !== "object") return false;
  const imports = (value as { readonly imports?: unknown }).imports;
  return (
    Array.isArray(imports) &&
    imports.every(
      (entry) =>
        !!entry &&
        typeof entry === "object" &&
        typeof (entry as OmenaNapiSourceImportDeclarationV0).declarationId === "string" &&
        typeof (entry as OmenaNapiSourceImportDeclarationV0).binding === "string" &&
        typeof (entry as OmenaNapiSourceImportDeclarationV0).specifier === "string" &&
        isSourceFrontendByteSpan((entry as OmenaNapiSourceImportDeclarationV0).specifierByteSpan),
    )
  );
}

function isSourceFrontendByteSpan(value: unknown): boolean {
  if (!value || typeof value !== "object") return false;
  const span = value as { readonly start?: unknown; readonly end?: unknown };
  return (
    typeof span.start === "number" &&
    Number.isSafeInteger(span.start) &&
    span.start >= 0 &&
    typeof span.end === "number" &&
    Number.isSafeInteger(span.end) &&
    span.end >= span.start
  );
}

function collectSourceFrontendImportInputs(args: {
  readonly imports: readonly OmenaNapiSourceImportDeclarationV0[];
  readonly content: string;
  readonly filePath: string;
  readonly aliasResolver: AliasResolver;
  readonly fileExists: (path: string) => boolean;
}): SourceFrontendImportInputsV0 {
  const styleExtensions = getAllStyleExtensions();
  const styleImportResolutions: { declarationId: string; styleUri: string }[] = [];
  const missingStyleImports: {
    declarationId: string;
    binding: string;
    resolved: StyleImport;
  }[] = [];

  for (const declaration of args.imports) {
    const styleImport = resolveStyleImport(declaration, args);
    if (
      !styleImport ||
      !styleExtensions.some((extension) => styleImport.absolutePath.endsWith(extension))
    ) {
      continue;
    }
    if (styleImport.kind === "resolved") {
      styleImportResolutions.push({
        declarationId: declaration.declarationId,
        styleUri: pathToFileURL(styleImport.absolutePath).href,
      });
    } else {
      missingStyleImports.push({
        declarationId: declaration.declarationId,
        binding: declaration.binding,
        resolved: styleImport,
      });
    }
  }

  return {
    styleImportResolutions: styleImportResolutions.toSorted((a, b) =>
      compareUtf8ByteOrder(`${a.declarationId}:${a.styleUri}`, `${b.declarationId}:${b.styleUri}`),
    ),
    missingStyleImports: missingStyleImports.toSorted((a, b) =>
      compareUtf8ByteOrder(
        `${a.declarationId}:${a.resolved.absolutePath}`,
        `${b.declarationId}:${b.resolved.absolutePath}`,
      ),
    ),
  };
}

function resolveStyleImport(
  declaration: OmenaNapiSourceImportDeclarationV0,
  args: {
    readonly filePath: string;
    readonly content: string;
    readonly aliasResolver: AliasResolver;
    readonly fileExists: (path: string) => boolean;
  },
): StyleImport | null {
  const specifier = declaration.specifier;
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
    range: rangeForSpecifierByteSpan(args.content, declaration.specifierByteSpan),
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
    .map((entry) =>
      makeStyleImportBinding(
        `rust-missing-style-import:${entry.declarationId}:${entry.resolved.absolutePath}`,
        entry.binding,
        entry.declarationId,
        entry.resolved,
      ),
    );
  if (additions.length === 0) return sourceDocument;
  return {
    ...sourceDocument,
    styleImports: [...sourceDocument.styleImports, ...additions].toSorted(
      (a, b) => compareUtf8ByteOrder(a.localName, b.localName) || compareUtf8ByteOrder(a.id, b.id),
    ),
  };
}

function compareUtf8ByteOrder(left: string, right: string): number {
  return Buffer.compare(Buffer.from(left, "utf8"), Buffer.from(right, "utf8"));
}

function rangeForSpecifierByteSpan(
  source: string,
  span: OmenaNapiSourceImportDeclarationV0["specifierByteSpan"],
) {
  const specifierStart = utf16OffsetAtUtf8ByteOffset(source, span.start);
  const specifierEnd = utf16OffsetAtUtf8ByteOffset(source, span.end);
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
