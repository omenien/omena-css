import nodePath from "node:path";
import type { ComposesRef, Range } from "@omena/shared";
import {
  getAllStyleExtensions,
  findLangForPath,
} from "../../engine-core-ts/src/core/scss/lang-registry";
import {
  findComposesTokenAtCursor,
  resolveComposesTarget,
} from "../../engine-core-ts/src/core/query/find-style-selector";
import { fileUrlToPath, pathToFileUrl } from "../../engine-core-ts/src/core/util/text-utils";
import { isRecord } from "../../engine-core-ts/src/core/util/value-guards";
import type { ProviderDeps } from "../../engine-core-ts/src/provider-deps";
import type {
  SelectorDeclHIR,
  StyleDocumentHIR,
} from "../../engine-core-ts/src/core/hir/style-types";
import {
  resolveSelectedQueryBackendKind,
  runRustSelectedQueryBackendJson,
  SELECTED_QUERY_RUNNER_COMMANDS,
} from "./selected-query-backend";

export interface CodeActionDiagnosticInput {
  readonly range: Range;
  readonly message: string;
  readonly data?: unknown;
}

export type CodeActionPlanKind = "quickfix" | "refactor.extract" | "refactor.inline";

type CodeActionDeps = Pick<ProviderDeps, "fileExists"> &
  Partial<Pick<ProviderDeps, "buildStyleDocument" | "readStyleFile" | "styleDocumentForPath">> & {
    readonly runRustSelectedQueryBackendJson?: typeof runRustSelectedQueryBackendJson;
  };

interface OmenaQueryCodeActionPlanJson {
  readonly product: "omena-query.code-actions";
  readonly fileUri: string;
  readonly fileKind: "style";
  readonly actionCount: number;
  readonly actions: readonly {
    readonly title: string;
    readonly kind: CodeActionPlanKind;
    readonly edits: readonly {
      readonly uri: string;
      readonly range: Range;
      readonly newText: string;
    }[];
    readonly source: string;
  }[];
}

export type CodeActionPlan =
  | {
      readonly kind: "textEdit";
      readonly actionKind?: CodeActionPlanKind;
      readonly title: string;
      readonly diagnosticIndex: number;
      readonly uri: string;
      readonly range: Range;
      readonly newText: string;
      readonly isPreferred?: boolean;
    }
  | {
      readonly kind: "createFile";
      readonly actionKind?: CodeActionPlanKind;
      readonly title: string;
      readonly uri: string;
      readonly diagnosticIndex?: number;
      readonly isPreferred?: boolean;
    }
  | {
      readonly kind: "workspaceEdit";
      readonly actionKind?: CodeActionPlanKind;
      readonly title: string;
      readonly edits: readonly {
        readonly uri: string;
        readonly range: Range;
        readonly newText: string;
      }[];
      readonly diagnosticIndex?: number;
      readonly isPreferred?: boolean;
    };

export function planCodeActions(
  args: {
    readonly documentUri: string;
    readonly documentContent?: string;
    readonly range?: Range;
    readonly diagnostics: readonly CodeActionDiagnosticInput[];
  },
  deps: CodeActionDeps,
): readonly CodeActionPlan[] {
  const plans: CodeActionPlan[] = [];
  const diagnosticCreateModuleUris = new Set<string>();

  let diagnosticIndex = 0;
  for (const diagnostic of args.diagnostics) {
    const suggestion = extractSuggestion(diagnostic);
    if (suggestion) {
      plans.push({
        kind: "textEdit",
        title: `Replace with '${suggestion}'`,
        diagnosticIndex,
        uri: args.documentUri,
        range: diagnostic.range,
        newText: suggestion,
        isPreferred: true,
      });
    }

    const createSelector = extractCreateSelector(diagnostic);
    if (createSelector) {
      const className =
        createSelector.selectorName ??
        extractCreateSelectorClassName(diagnostic.message, createSelector.newText);
      plans.push({
        kind: "textEdit",
        title: `Add '${selectorLabel(className)}' to ${fileLabel(createSelector.uri)}`,
        diagnosticIndex,
        uri: createSelector.uri,
        range: createSelector.range,
        newText: createSelector.newText,
      });
    }

    const createModuleFile = extractCreateModuleFile(diagnostic);
    if (createModuleFile) {
      diagnosticCreateModuleUris.add(createModuleFile.uri);
      plans.push({
        kind: "createFile",
        title: `Create ${fileLabel(createModuleFile.uri)}`,
        diagnosticIndex,
        uri: createModuleFile.uri,
        isPreferred: true,
      });
    }

    const createValue = extractCreateValue(diagnostic);
    if (createValue) {
      const valueName =
        createValue.valueName ?? extractCreateValueName(diagnostic.message, createValue.newText);
      plans.push({
        kind: "textEdit",
        title: `Add '@value ${valueName}' to ${fileLabel(createValue.uri)}`,
        diagnosticIndex,
        uri: createValue.uri,
        range: createValue.range,
        newText: createValue.newText,
      });
    }

    const createKeyframes = extractCreateKeyframes(diagnostic);
    if (createKeyframes) {
      const keyframesName =
        createKeyframes.keyframesName ??
        extractCreateKeyframesName(diagnostic.message, createKeyframes.newText);
      plans.push({
        kind: "textEdit",
        title: `Add '@keyframes ${keyframesName}' to ${fileLabel(createKeyframes.uri)}`,
        diagnosticIndex,
        uri: createKeyframes.uri,
        range: createKeyframes.range,
        newText: createKeyframes.newText,
      });
    }

    const createCustomProperty = extractCreateCustomProperty(diagnostic);
    if (createCustomProperty) {
      const propertyName =
        createCustomProperty.propertyName ??
        extractCreateCustomPropertyName(diagnostic.message, createCustomProperty.newText);
      plans.push({
        kind: "textEdit",
        title: `Add '${propertyName}' to ${fileLabel(createCustomProperty.uri)}`,
        diagnosticIndex,
        uri: createCustomProperty.uri,
        range: createCustomProperty.range,
        newText: createCustomProperty.newText,
      });
    }

    const createSassSymbol = extractCreateSassSymbol(diagnostic);
    if (createSassSymbol) {
      const label =
        createSassSymbol.symbolLabel ??
        extractCreateSassSymbolLabel(diagnostic.message, createSassSymbol.newText);
      plans.push({
        kind: "textEdit",
        title: `Add '${label}' to ${fileLabel(createSassSymbol.uri)}`,
        diagnosticIndex,
        uri: createSassSymbol.uri,
        range: createSassSymbol.range,
        newText: createSassSymbol.newText,
      });
    }

    diagnosticIndex += 1;
  }

  const queryOwnedStyleRefactors = planQueryOwnedStyleRefactors(args, deps);
  if (queryOwnedStyleRefactors) {
    plans.push(...queryOwnedStyleRefactors);
  } else {
    const inlineComposedClass = planInlineComposedClass(args, deps);
    if (inlineComposedClass) {
      plans.push(inlineComposedClass);
    }

    const extractCustomProperty = inlineComposedClass ? null : planExtractCustomProperty(args);
    if (extractCustomProperty) {
      plans.push(extractCustomProperty);
    }
    const extractValue = inlineComposedClass ? null : planExtractValue(args);
    if (extractValue) {
      plans.push(extractValue);
    }
  }

  if (diagnosticCreateModuleUris.size === 0) {
    for (const uri of listMissingSiblingStyleModuleUris(args.documentUri, deps)) {
      plans.push({
        kind: "createFile",
        title: `Create ${fileLabel(uri)}`,
        uri,
      });
    }
  }

  return plans;
}

function planInlineComposedClass(
  args: {
    readonly documentUri: string;
    readonly documentContent?: string;
    readonly range?: Range;
    readonly diagnostics: readonly CodeActionDiagnosticInput[];
  },
  deps: CodeActionDeps,
): CodeActionPlan | null {
  if (args.diagnostics.length > 0) return null;
  if (!args.documentContent || !args.range) return null;
  const filePath = fileUrlToPath(args.documentUri);
  if (findLangForPath(filePath) === null) return null;

  const styleDocument = buildStyleDocumentForCodeAction(filePath, args.documentContent, deps);
  if (!styleDocument) return null;
  const hit = findComposesTokenAtCursor(
    styleDocument,
    args.range.start.line,
    args.range.start.character,
  );
  if (!hit?.ref.range || hit.ref.fromGlobal) return null;

  const target = resolveComposesTarget(
    (candidatePath) => styleDocumentForInline(candidatePath, filePath, styleDocument, deps),
    filePath,
    hit,
  );
  if (!target) return null;

  const declarationLines = collectInlineDeclarations(target, {
    currentPath: filePath,
    currentDocument: styleDocument,
    deps,
    emitted: new Set(),
    visiting: new Set(),
  });
  if (!declarationLines) return null;
  if (declarationLines.length === 0) return null;

  const replacementRange = expandComposesDeclarationRange(args.documentContent, hit.ref.range);
  const indent = lineIndentAt(args.documentContent, replacementRange.start.line);
  return {
    kind: "workspaceEdit",
    actionKind: "refactor.inline",
    title: `Inline composed class '${hit.token.className}'${target.filePath === filePath ? "" : ` from ${fileLabel(target.filePath)}`}`,
    edits: [
      {
        uri: args.documentUri,
        range: replacementRange,
        newText: formatInlineDeclarations(declarationLines, indent),
      },
    ],
  };
}

interface InlineTarget {
  readonly filePath: string;
  readonly styleDocument: StyleDocumentHIR;
  readonly selector: SelectorDeclHIR;
}

interface InlineResolutionContext {
  readonly currentPath: string;
  readonly currentDocument: StyleDocumentHIR;
  readonly deps: CodeActionDeps;
  readonly emitted: Set<string>;
  readonly visiting: Set<string>;
}

function planQueryOwnedStyleRefactors(
  args: {
    readonly documentUri: string;
    readonly documentContent?: string;
    readonly range?: Range;
    readonly diagnostics: readonly CodeActionDiagnosticInput[];
  },
  deps: CodeActionDeps,
): readonly CodeActionPlan[] | null {
  if (resolveSelectedQueryBackendKind() !== "rust-selected-query") return null;
  if (args.diagnostics.length > 0) return null;
  if (!args.documentContent || !args.range || rangeIsEmpty(args.range)) return null;
  const filePath = fileUrlToPath(args.documentUri);
  if (findLangForPath(filePath) === null) return null;

  const runJson = deps.runRustSelectedQueryBackendJson ?? runRustSelectedQueryBackendJson;
  const plan = runJson<OmenaQueryCodeActionPlanJson>(
    SELECTED_QUERY_RUNNER_COMMANDS.styleCodeActions,
    {
      styleUri: args.documentUri,
      styleSource: args.documentContent,
      range: args.range,
      styles: collectStyleSourcesForQueryOwnedRefactors(
        filePath,
        args.documentUri,
        args.documentContent,
        deps,
      ),
      packageManifests: [],
    },
  );
  if (plan.product !== "omena-query.code-actions") return null;
  return plan.actions.map((action) => ({
    kind: "workspaceEdit",
    actionKind: action.kind,
    title: action.title,
    edits: action.edits,
  }));
}

function collectStyleSourcesForQueryOwnedRefactors(
  filePath: string,
  documentUri: string,
  documentContent: string,
  deps: CodeActionDeps,
): readonly { readonly stylePath: string; readonly styleSource: string }[] {
  const sources = new Map([[documentUri, documentContent]]);
  const styleDocument = buildStyleDocumentForCodeAction(filePath, documentContent, deps);
  if (!styleDocument)
    return [...sources].map(([stylePath, styleSource]) => ({ stylePath, styleSource }));

  for (const selector of styleDocument.selectors) {
    for (const ref of selector.composes) {
      if (ref.fromGlobal || !ref.from) continue;
      for (const candidatePath of resolveRelativeStyleModuleCandidates(filePath, ref.from)) {
        if (sources.has(pathToFileUrl(candidatePath))) continue;
        const source = deps.readStyleFile?.(candidatePath);
        if (source === undefined || source === null) continue;
        sources.set(pathToFileUrl(candidatePath), source);
      }
    }
  }

  return [...sources].map(([stylePath, styleSource]) => ({ stylePath, styleSource }));
}

function resolveRelativeStyleModuleCandidates(
  filePath: string,
  specifier: string,
): readonly string[] {
  if (!specifier.startsWith(".")) return [];
  const basePath = nodePath.resolve(nodePath.dirname(filePath), specifier);
  if (nodePath.extname(basePath)) return [basePath];
  return getAllStyleExtensions().map((extension) => `${basePath}${extension}`);
}

function collectInlineDeclarations(
  target: InlineTarget,
  context: InlineResolutionContext,
): readonly string[] | null {
  const key = `${target.filePath}\u0000${target.selector.canonicalName}`;
  if (context.emitted.has(key)) return [];
  if (context.visiting.has(key)) return null;

  context.visiting.add(key);
  const declarations: string[] = [];

  for (const ref of target.selector.composes) {
    if (ref.fromGlobal) return null;
    for (const className of ref.classNames) {
      const nestedTarget = resolveInlineComposesClass(target.filePath, ref, className, context);
      if (!nestedTarget) return null;
      const nestedDeclarations = collectInlineDeclarations(nestedTarget, context);
      if (!nestedDeclarations) return null;
      declarations.push(...nestedDeclarations);
    }
  }

  declarations.push(...splitInlineDeclarations(target.selector.declarations));
  context.visiting.delete(key);
  context.emitted.add(key);
  return declarations;
}

function resolveInlineComposesClass(
  ownerFilePath: string,
  ref: ComposesRef,
  className: string,
  context: InlineResolutionContext,
): InlineTarget | null {
  const targetPath = ref.from
    ? nodePath.resolve(nodePath.dirname(ownerFilePath), ref.from)
    : ownerFilePath;
  const targetDocument = styleDocumentForInline(
    targetPath,
    context.currentPath,
    context.currentDocument,
    context.deps,
  );
  if (!targetDocument) return null;
  const selector =
    targetDocument.selectors.find(
      (candidate) => candidate.canonicalName === className && candidate.viewKind === "canonical",
    ) ?? targetDocument.selectors.find((candidate) => candidate.canonicalName === className);
  if (!selector) return null;
  return {
    filePath: targetDocument.filePath,
    styleDocument: targetDocument,
    selector,
  };
}

function styleDocumentForInline(
  candidatePath: string,
  currentPath: string,
  currentDocument: StyleDocumentHIR,
  deps: CodeActionDeps,
): StyleDocumentHIR | null {
  if (candidatePath === currentPath) return currentDocument;

  const indexed = deps.styleDocumentForPath?.(candidatePath);
  if (indexed) return indexed;

  const content = deps.readStyleFile?.(candidatePath);
  if (content === undefined || content === null) return null;
  return buildStyleDocumentForCodeAction(candidatePath, content, deps);
}

function buildStyleDocumentForCodeAction(
  filePath: string,
  content: string,
  deps: CodeActionDeps,
): StyleDocumentHIR | null {
  return deps.buildStyleDocument?.(filePath, content) ?? null;
}

function planExtractCustomProperty(args: {
  readonly documentUri: string;
  readonly documentContent?: string;
  readonly range?: Range;
  readonly diagnostics: readonly CodeActionDiagnosticInput[];
}): CodeActionPlan | null {
  const filePath = fileUrlToPath(args.documentUri);
  if (findLangForPath(filePath) === null) return null;
  if (args.diagnostics.length > 0) return null;
  if (!args.documentContent || !args.range || rangeIsEmpty(args.range)) return null;

  const selected = sliceRange(args.documentContent, args.range);
  if (!selected) return null;
  const value = selected.trim();
  if (!isExtractableCssValue(value)) return null;

  const propertyName = nextCustomPropertyName(args.documentContent, customPropertyStem(value));
  return {
    kind: "workspaceEdit",
    actionKind: "refactor.extract",
    title: `Extract CSS custom property '${propertyName}'`,
    edits: [
      {
        uri: args.documentUri,
        range: {
          start: { line: 0, character: 0 },
          end: { line: 0, character: 0 },
        },
        newText: `:root {\n  ${propertyName}: ${value};\n}\n\n`,
      },
      {
        uri: args.documentUri,
        range: args.range,
        newText: `var(${propertyName})`,
      },
    ],
  };
}

function planExtractValue(args: {
  readonly documentUri: string;
  readonly documentContent?: string;
  readonly range?: Range;
  readonly diagnostics: readonly CodeActionDiagnosticInput[];
}): CodeActionPlan | null {
  const filePath = fileUrlToPath(args.documentUri);
  if (findLangForPath(filePath) === null) return null;
  if (args.diagnostics.length > 0) return null;
  if (!args.documentContent || !args.range || rangeIsEmpty(args.range)) return null;

  const selected = sliceRange(args.documentContent, args.range);
  if (!selected) return null;
  const value = selected.trim();
  if (!isExtractableCssValue(value)) return null;

  const valueName = nextValueName(args.documentContent, valueNameStem(value));
  return {
    kind: "workspaceEdit",
    actionKind: "refactor.extract",
    title: `Extract @value '${valueName}'`,
    edits: [
      {
        uri: args.documentUri,
        range: {
          start: { line: 0, character: 0 },
          end: { line: 0, character: 0 },
        },
        newText: `@value ${valueName}: ${value};\n\n`,
      },
      {
        uri: args.documentUri,
        range: args.range,
        newText: valueName,
      },
    ],
  };
}

function splitInlineDeclarations(declarations: string): readonly string[] {
  return declarations
    .split(";")
    .map((declaration) => declaration.trim())
    .filter((declaration) => declaration.length > 0);
}

function formatInlineDeclarations(
  declarations: readonly string[],
  continuationIndent: string,
): string {
  return declarations
    .map((declaration, index) => `${index === 0 ? "" : continuationIndent}${declaration};`)
    .join("\n");
}

function expandComposesDeclarationRange(content: string, range: Range): Range {
  const endOffset = offsetAt(content, range.end.line, range.end.character);
  if (endOffset === null) return range;
  let expandedEndOffset = endOffset;
  while (content[expandedEndOffset] === " " || content[expandedEndOffset] === "\t") {
    expandedEndOffset += 1;
  }
  if (content[expandedEndOffset] === ";") {
    expandedEndOffset += 1;
  }
  return {
    start: range.start,
    end: positionAt(content, expandedEndOffset),
  };
}

function lineIndentAt(content: string, line: number): string {
  const lineText = content.split("\n")[line] ?? "";
  return /^[ \t]*/u.exec(lineText)?.[0] ?? "";
}

function listMissingSiblingStyleModuleUris(
  uri: string,
  deps: Pick<ProviderDeps, "fileExists">,
): readonly string[] {
  const filePath = fileUrlToPath(uri);
  if (findLangForPath(filePath) !== null) return [];
  if (!isSetupEligibleSourcePath(filePath)) return [];

  const sourceBasePath = filePath.replace(/\.[^.]+$/u, "");
  const siblingPaths = getAllStyleExtensions().map((extension) => `${sourceBasePath}${extension}`);
  if (siblingPaths.some((candidate) => deps.fileExists(candidate))) {
    return [];
  }

  return siblingPaths.map((candidate) => pathToFileUrl(candidate));
}

function isSetupEligibleSourcePath(filePath: string): boolean {
  const extension = nodePath.extname(filePath).toLowerCase();
  return extension === ".tsx" || extension === ".jsx";
}

function rangeIsEmpty(range: Range): boolean {
  return range.start.line === range.end.line && range.start.character === range.end.character;
}

function sliceRange(content: string, range: Range): string | null {
  const start = offsetAt(content, range.start.line, range.start.character);
  const end = offsetAt(content, range.end.line, range.end.character);
  if (start === null || end === null || end <= start) return null;
  return content.slice(start, end);
}

function offsetAt(content: string, line: number, character: number): number | null {
  if (line < 0 || character < 0) return null;
  let offset = 0;
  for (let currentLine = 0; currentLine < line; currentLine += 1) {
    const nextLine = content.indexOf("\n", offset);
    if (nextLine === -1) return null;
    offset = nextLine + 1;
  }
  const lineEnd = content.indexOf("\n", offset);
  const maxCharacter = (lineEnd === -1 ? content.length : lineEnd) - offset;
  if (character > maxCharacter) return null;
  return offset + character;
}

function positionAt(content: string, offset: number): Range["start"] {
  const boundedOffset = Math.max(0, Math.min(offset, content.length));
  const prefix = content.slice(0, boundedOffset);
  const line = prefix.split("\n").length - 1;
  const lineStart = prefix.lastIndexOf("\n") + 1;
  return { line, character: boundedOffset - lineStart };
}

function isExtractableCssValue(value: string): boolean {
  if (value.length === 0 || value.includes("\n") || value.startsWith("var(")) return false;
  return (
    /^#[0-9a-fA-F]{3,8}$/u.test(value) ||
    /^-?\d+(?:\.\d+)?(?:px|rem|em|%|vh|vw|vmin|vmax|ch|ex|s|ms|deg)?$/u.test(value) ||
    /^(?:rgb|rgba|hsl|hsla)\([^)]*\)$/u.test(value) ||
    /^[a-zA-Z][a-zA-Z-]*$/u.test(value)
  );
}

function customPropertyStem(value: string): string {
  if (/^#[0-9a-fA-F]{3,8}$/u.test(value) || /^(?:rgb|rgba|hsl|hsla)\(/u.test(value)) {
    return "extracted-color";
  }
  if (/^-?\d+(?:\.\d+)?(?:px|rem|em|%|vh|vw|vmin|vmax|ch|ex)?$/u.test(value)) {
    return "extracted-size";
  }
  if (/^-?\d+(?:\.\d+)?(?:s|ms)$/u.test(value)) {
    return "extracted-duration";
  }
  if (/^-?\d+(?:\.\d+)?deg$/u.test(value)) {
    return "extracted-angle";
  }
  return "extracted-token";
}

function nextCustomPropertyName(content: string, stem: string): string {
  let candidate = `--${stem}`;
  let suffix = 2;
  while (content.includes(candidate)) {
    candidate = `--${stem}-${suffix}`;
    suffix += 1;
  }
  return candidate;
}

function valueNameStem(value: string): string {
  if (/^#[0-9a-fA-F]{3,8}$/u.test(value) || /^(?:rgb|rgba|hsl|hsla)\(/u.test(value)) {
    return "extractedColor";
  }
  if (/^-?\d+(?:\.\d+)?(?:s|ms)$/u.test(value)) {
    return "extractedDuration";
  }
  if (/^-?\d+(?:\.\d+)?deg$/u.test(value)) {
    return "extractedAngle";
  }
  if (/^-?\d+(?:\.\d+)?(?:px|rem|em|%|vh|vw|vmin|vmax|ch|ex)?$/u.test(value)) {
    return "extractedSize";
  }
  return "extractedToken";
}

function nextValueName(content: string, stem: string): string {
  let candidate = stem;
  let suffix = 2;
  while (new RegExp(`\\b${escapeRegExp(candidate)}\\b`, "u").test(content)) {
    candidate = `${stem}${suffix}`;
    suffix += 1;
  }
  return candidate;
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
}

function extractSuggestion(diagnostic: CodeActionDiagnosticInput): string | null {
  const data = diagnostic.data;
  if (!isRecord(data)) return null;
  const suggestion = data.suggestion;
  return typeof suggestion === "string" && suggestion.length > 0 ? suggestion : null;
}

function extractCreateSelector(diagnostic: CodeActionDiagnosticInput): {
  readonly uri: string;
  readonly range: Range;
  readonly newText: string;
  readonly selectorName?: string;
} | null {
  const data = diagnostic.data;
  if (!isRecord(data)) return null;
  const payload = data.createSelector;
  if (!isRecord(payload)) return null;
  if (typeof payload.uri !== "string" || typeof payload.newText !== "string") return null;
  const range = payload.range;
  if (!isRange(range)) return null;
  return {
    uri: payload.uri,
    range,
    newText: payload.newText,
    ...(nonEmptyString(payload.selectorName) ? { selectorName: payload.selectorName } : {}),
  };
}

function extractCreateModuleFile(
  diagnostic: CodeActionDiagnosticInput,
): { readonly uri: string } | null {
  const data = diagnostic.data;
  if (!isRecord(data)) return null;
  const payload = data.createModuleFile;
  if (!isRecord(payload)) return null;
  return typeof payload.uri === "string" && payload.uri.length > 0 ? { uri: payload.uri } : null;
}

function extractCreateValue(diagnostic: CodeActionDiagnosticInput): {
  readonly uri: string;
  readonly range: Range;
  readonly newText: string;
  readonly valueName?: string;
} | null {
  const data = diagnostic.data;
  if (!isRecord(data)) return null;
  const payload = data.createValue;
  if (!isRecord(payload)) return null;
  if (typeof payload.uri !== "string" || typeof payload.newText !== "string") return null;
  const range = payload.range;
  if (!isRange(range)) return null;
  return {
    uri: payload.uri,
    range,
    newText: payload.newText,
    ...(nonEmptyString(payload.valueName) ? { valueName: payload.valueName } : {}),
  };
}

function extractCreateKeyframes(diagnostic: CodeActionDiagnosticInput): {
  readonly uri: string;
  readonly range: Range;
  readonly newText: string;
  readonly keyframesName?: string;
} | null {
  const data = diagnostic.data;
  if (!isRecord(data)) return null;
  const payload = data.createKeyframes;
  if (!isRecord(payload)) return null;
  if (typeof payload.uri !== "string" || typeof payload.newText !== "string") return null;
  const range = payload.range;
  if (!isRange(range)) return null;
  return {
    uri: payload.uri,
    range,
    newText: payload.newText,
    ...(nonEmptyString(payload.keyframesName) ? { keyframesName: payload.keyframesName } : {}),
  };
}

function extractCreateCustomProperty(diagnostic: CodeActionDiagnosticInput): {
  readonly uri: string;
  readonly range: Range;
  readonly newText: string;
  readonly propertyName?: string;
} | null {
  const data = diagnostic.data;
  if (!isRecord(data)) return null;
  const payload = data.createCustomProperty;
  if (!isRecord(payload)) return null;
  if (typeof payload.uri !== "string" || typeof payload.newText !== "string") return null;
  const range = payload.range;
  if (!isRange(range)) return null;
  return {
    uri: payload.uri,
    range,
    newText: payload.newText,
    ...(nonEmptyString(payload.propertyName) ? { propertyName: payload.propertyName } : {}),
  };
}

function extractCreateSassSymbol(diagnostic: CodeActionDiagnosticInput): {
  readonly uri: string;
  readonly range: Range;
  readonly newText: string;
  readonly symbolLabel?: string;
} | null {
  const data = diagnostic.data;
  if (!isRecord(data)) return null;
  const payload = data.createSassSymbol;
  if (!isRecord(payload)) return null;
  if (typeof payload.uri !== "string" || typeof payload.newText !== "string") return null;
  const range = payload.range;
  if (!isRange(range)) return null;
  return {
    uri: payload.uri,
    range,
    newText: payload.newText,
    ...(nonEmptyString(payload.symbolLabel) ? { symbolLabel: payload.symbolLabel } : {}),
  };
}

function nonEmptyString(value: unknown): value is string {
  return typeof value === "string" && value.length > 0;
}

function isRange(value: unknown): value is Range {
  if (!isRecord(value)) return false;
  return isPosition(value.start) && isPosition(value.end);
}

function isPosition(value: unknown): value is Range["start"] {
  if (!isRecord(value)) return false;
  return typeof value.line === "number" && typeof value.character === "number";
}

function extractCreateSelectorClassName(message: string, newText: string): string {
  const fromMessage =
    /Class '\.([^']+)' not found/.exec(message)?.[1] ??
    /Selector '\.([^']+)' not found/.exec(message)?.[1];
  if (fromMessage) return fromMessage;
  return /^\s*\.([^{\s]+)\s*\{/u.exec(newText)?.[1] ?? "selector";
}

function selectorLabel(className: string): string {
  return className.startsWith(".") ? className : `.${className}`;
}

function extractCreateValueName(message: string, newText: string): string {
  const fromMessage =
    /@value '([^']+)' not found/.exec(message)?.[1] ?? /local binding '([^']+)'/.exec(message)?.[1];
  if (fromMessage) return fromMessage;
  return /^\s*@value\s+([^:\s]+)\s*:/u.exec(newText)?.[1] ?? "value";
}

function extractCreateKeyframesName(message: string, newText: string): string {
  const fromMessage = /@keyframes '([^']+)' not found/.exec(message)?.[1];
  if (fromMessage) return fromMessage;
  return /^\s*@keyframes\s+([^{\s]+)\s*\{/u.exec(newText)?.[1] ?? "keyframes";
}

function extractCreateCustomPropertyName(message: string, newText: string): string {
  const fromMessage = /CSS custom property '([^']+)' not found/.exec(message)?.[1];
  if (fromMessage) return fromMessage;
  return /^\s*(--[\w-]+)\s*:/mu.exec(newText)?.[1] ?? "custom-property";
}

function extractCreateSassSymbolLabel(message: string, newText: string): string {
  const lessVariableFromMessage = /Less variable '@([^']+)' not found/.exec(message)?.[1];
  if (lessVariableFromMessage) return `@${lessVariableFromMessage}`;
  const variableFromMessage = /Sass variable '\$([^']+)' not found/.exec(message)?.[1];
  if (variableFromMessage) return `$${variableFromMessage}`;
  const mixinFromMessage = /Sass mixin '@mixin ([^']+)' not found/.exec(message)?.[1];
  if (mixinFromMessage) return `@mixin ${mixinFromMessage}`;
  const functionFromMessage = /Sass function '([^']+)\(\)' not found/.exec(message)?.[1];
  if (functionFromMessage) return `@function ${functionFromMessage}`;

  const variableFromText = /^\s*\$([A-Za-z_-][A-Za-z0-9_-]*)\s*:/u.exec(newText)?.[1];
  if (variableFromText) return `$${variableFromText}`;
  const lessVariableFromText = /^\s*@([A-Za-z_-][A-Za-z0-9_-]*)\s*:/u.exec(newText)?.[1];
  if (lessVariableFromText) return `@${lessVariableFromText}`;
  const mixinFromText = /^\s*@mixin\s+([A-Za-z_-][A-Za-z0-9_-]*)/u.exec(newText)?.[1];
  if (mixinFromText) return `@mixin ${mixinFromText}`;
  const functionFromText = /^\s*@function\s+([A-Za-z_-][A-Za-z0-9_-]*)/u.exec(newText)?.[1];
  if (functionFromText) return `@function ${functionFromText}`;
  return "Sass symbol";
}

function fileLabel(uri: string): string {
  return uri.split("/").at(-1) ?? uri;
}
