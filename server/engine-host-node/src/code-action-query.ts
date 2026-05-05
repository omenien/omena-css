import nodePath from "node:path";
import type { Range } from "@css-module-explainer/shared";
import {
  getAllStyleExtensions,
  findLangForPath,
} from "../../engine-core-ts/src/core/scss/lang-registry";
import { fileUrlToPath, pathToFileUrl } from "../../engine-core-ts/src/core/util/text-utils";
import { isRecord } from "../../engine-core-ts/src/core/util/value-guards";
import type { ProviderDeps } from "../../engine-core-ts/src/provider-deps";

export interface CodeActionDiagnosticInput {
  readonly range: Range;
  readonly message: string;
  readonly data?: unknown;
}

export type CodeActionPlanKind = "quickfix" | "refactor.extract";

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
  deps: Pick<ProviderDeps, "fileExists">,
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

  const extractCustomProperty = planExtractCustomProperty(args);
  if (extractCustomProperty) {
    plans.push(extractCustomProperty);
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
