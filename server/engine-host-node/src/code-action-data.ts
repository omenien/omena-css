import type { Range } from "@css-module-explainer/shared";
import type {
  SassSymbolKind,
  StylePreprocessorSymbolSyntax,
  StyleDocumentHIR,
} from "../../engine-core-ts/src/core/hir/style-types";
import { pathToFileUrl } from "../../engine-core-ts/src/core/util/text-utils";

export interface RecoveryEditActionData {
  readonly uri: string;
  readonly range: Range;
  readonly newText: string;
}

export interface CreateSelectorActionData extends RecoveryEditActionData {
  readonly selectorName: string;
}

export interface CreateValueActionData extends RecoveryEditActionData {
  readonly valueName: string;
}

export interface CreateKeyframesActionData extends RecoveryEditActionData {
  readonly keyframesName: string;
}

export interface CreateCustomPropertyActionData extends RecoveryEditActionData {
  readonly propertyName: string;
}

export interface CreateSassSymbolActionData extends RecoveryEditActionData {
  readonly symbolKind: SassSymbolKind;
  readonly symbolName: string;
  readonly symbolLabel: string;
}

export function buildCreateSelectorActionData(
  className: string,
  scssModulePath: string,
  styleDocument: StyleDocumentHIR,
  styleContent?: string,
): CreateSelectorActionData {
  const insertionRange = findSelectorInsertionRange(styleDocument, styleContent);
  return {
    uri: pathToFileUrl(scssModulePath),
    range: insertionRange,
    newText:
      styleDocument.selectors.length > 0 ? `\n\n.${className} {\n}\n` : `.${className} {\n}\n`,
    selectorName: className,
  };
}

export function buildCreateValueActionData(
  valueName: string,
  scssModulePath: string,
  styleDocument: StyleDocumentHIR,
  styleContent?: string,
): CreateValueActionData {
  const insertionRange = findValueInsertionRange(styleDocument, styleContent);
  return {
    uri: pathToFileUrl(scssModulePath),
    range: insertionRange,
    newText:
      styleDocument.valueDecls.length > 0 || styleDocument.valueImports.length > 0
        ? `\n@value ${valueName}: ;`
        : `@value ${valueName}: ;\n`,
    valueName,
  };
}

export function buildCreateKeyframesActionData(
  keyframesName: string,
  scssModulePath: string,
  styleDocument: StyleDocumentHIR,
): CreateKeyframesActionData {
  const insertionRange = findKeyframesInsertionRange(styleDocument);
  const hasExistingContent =
    styleDocument.keyframes.length > 0 ||
    styleDocument.valueDecls.length > 0 ||
    styleDocument.valueImports.length > 0 ||
    styleDocument.selectors.length > 0;
  const insertsAtTop =
    insertionRange.start.line === 0 &&
    insertionRange.start.character === 0 &&
    insertionRange.end.line === 0 &&
    insertionRange.end.character === 0 &&
    styleDocument.keyframes.length === 0 &&
    styleDocument.valueDecls.length === 0 &&
    styleDocument.valueImports.length === 0;

  return {
    uri: pathToFileUrl(scssModulePath),
    range: insertionRange,
    newText: insertsAtTop
      ? hasExistingContent
        ? `@keyframes ${keyframesName} {\n}\n\n`
        : `@keyframes ${keyframesName} {\n}\n`
      : `\n\n@keyframes ${keyframesName} {\n}\n`,
    keyframesName,
  };
}

export function buildCreateCustomPropertyActionData(
  propertyName: string,
  scssModulePath: string,
  styleDocument: StyleDocumentHIR,
): CreateCustomPropertyActionData {
  const insertionRange = findStyleDocumentAppendRange(styleDocument);
  const insertsAtTop =
    insertionRange.start.line === 0 &&
    insertionRange.start.character === 0 &&
    insertionRange.end.line === 0 &&
    insertionRange.end.character === 0;
  const hasExistingContent = hasStyleDocumentContent(styleDocument);
  const block = `:root {\n  ${propertyName}: ;\n}`;

  return {
    uri: pathToFileUrl(scssModulePath),
    range: insertionRange,
    newText: insertsAtTop ? (hasExistingContent ? `${block}\n\n` : `${block}\n`) : `\n\n${block}\n`,
    propertyName,
  };
}

export function buildCreateSassSymbolActionData(
  symbolKind: SassSymbolKind,
  symbolName: string,
  scssModulePath: string,
  styleDocument: StyleDocumentHIR,
  syntax: StylePreprocessorSymbolSyntax = "sass",
): CreateSassSymbolActionData {
  const insertionRange = findSassSymbolInsertionRange(styleDocument);
  const stub = sassSymbolStub(symbolKind, symbolName, syntax);
  const insertsAtTop =
    insertionRange.start.line === 0 &&
    insertionRange.start.character === 0 &&
    insertionRange.end.line === 0 &&
    insertionRange.end.character === 0;
  const hasExistingContent =
    styleDocument.valueDecls.length > 0 ||
    styleDocument.valueImports.length > 0 ||
    styleDocument.keyframes.length > 0 ||
    styleDocument.selectors.length > 0;

  return {
    uri: pathToFileUrl(scssModulePath),
    range: insertionRange,
    newText: insertsAtTop ? (hasExistingContent ? `${stub}\n\n` : `${stub}\n`) : `\n\n${stub}`,
    symbolKind,
    symbolName,
    symbolLabel: sassSymbolLabel(symbolKind, symbolName, syntax),
  };
}

function findSelectorInsertionRange(styleDocument: StyleDocumentHIR, styleContent?: string): Range {
  if (styleDocument.selectors.length === 0) {
    return {
      start: { line: 0, character: 0 },
      end: { line: 0, character: 0 },
    };
  }

  let latest = styleDocument.selectors[0]!.ruleRange.end;
  for (const selector of styleDocument.selectors) {
    const end = selector.ruleRange.end;
    if (end.line > latest.line || (end.line === latest.line && end.character > latest.character)) {
      latest = end;
    }
  }

  const position = normalizeAppendPosition(styleContent, latest);
  return {
    start: position,
    end: position,
  };
}

function findValueInsertionRange(styleDocument: StyleDocumentHIR, styleContent?: string): Range {
  const valueFacts = [...styleDocument.valueDecls, ...styleDocument.valueImports];
  if (valueFacts.length === 0) {
    return {
      start: { line: 0, character: 0 },
      end: { line: 0, character: 0 },
    };
  }

  let latest = valueFacts[0]!.ruleRange.end;
  for (const valueFact of valueFacts) {
    const end = valueFact.ruleRange.end;
    if (end.line > latest.line || (end.line === latest.line && end.character > latest.character)) {
      latest = end;
    }
  }

  const position = normalizeAppendPosition(styleContent, latest);
  return {
    start: position,
    end: position,
  };
}

function normalizeAppendPosition(
  styleContent: string | undefined,
  position: Range["start"],
): Range["start"] {
  if (!styleContent) return position;
  const line = styleContent.split(/\r\n|\n|\r/u)[position.line];
  if (!line) return position;
  const next = line[position.character];
  if (next !== "}" && next !== ";") return position;
  return {
    line: position.line,
    character: position.character + 1,
  };
}

function findSassSymbolInsertionRange(styleDocument: StyleDocumentHIR): Range {
  const valueFacts = [...styleDocument.valueDecls, ...styleDocument.valueImports];
  if (valueFacts.length === 0) {
    return {
      start: { line: 0, character: 0 },
      end: { line: 0, character: 0 },
    };
  }

  let latest = valueFacts[0]!.ruleRange.end;
  for (const valueFact of valueFacts) {
    const end = valueFact.ruleRange.end;
    if (end.line > latest.line || (end.line === latest.line && end.character > latest.character)) {
      latest = end;
    }
  }

  return {
    start: { line: latest.line, character: latest.character },
    end: { line: latest.line, character: latest.character },
  };
}

function sassSymbolStub(
  symbolKind: SassSymbolKind,
  symbolName: string,
  syntax: StylePreprocessorSymbolSyntax,
): string {
  if (syntax === "less") return `@${symbolName}: ;`;
  switch (symbolKind) {
    case "variable":
      return `$${symbolName}: ;`;
    case "mixin":
      return `@mixin ${symbolName}() {\n}`;
    case "function":
      return `@function ${symbolName}() {\n  @return null;\n}`;
  }
}

function sassSymbolLabel(
  symbolKind: SassSymbolKind,
  symbolName: string,
  syntax: StylePreprocessorSymbolSyntax,
): string {
  if (syntax === "less") return `@${symbolName}`;
  switch (symbolKind) {
    case "variable":
      return `$${symbolName}`;
    case "mixin":
      return `@mixin ${symbolName}`;
    case "function":
      return `@function ${symbolName}`;
  }
}

function findStyleDocumentAppendRange(styleDocument: StyleDocumentHIR): Range {
  const facts = [
    ...styleDocument.customPropertyDecls,
    ...styleDocument.keyframes,
    ...styleDocument.valueDecls,
    ...styleDocument.valueImports,
    ...styleDocument.selectors,
    ...styleDocument.sassSymbolDecls,
  ];
  if (facts.length === 0) {
    return {
      start: { line: 0, character: 0 },
      end: { line: 0, character: 0 },
    };
  }

  let latest = facts[0]!.ruleRange.end;
  for (const fact of facts) {
    const end = fact.ruleRange.end;
    if (end.line > latest.line || (end.line === latest.line && end.character > latest.character)) {
      latest = end;
    }
  }

  return {
    start: { line: latest.line, character: latest.character },
    end: { line: latest.line, character: latest.character },
  };
}

function hasStyleDocumentContent(styleDocument: StyleDocumentHIR): boolean {
  return (
    styleDocument.customPropertyDecls.length > 0 ||
    styleDocument.keyframes.length > 0 ||
    styleDocument.valueDecls.length > 0 ||
    styleDocument.valueImports.length > 0 ||
    styleDocument.selectors.length > 0 ||
    styleDocument.sassSymbolDecls.length > 0
  );
}

function findKeyframesInsertionRange(styleDocument: StyleDocumentHIR): Range {
  if (styleDocument.keyframes.length > 0) {
    let latest = styleDocument.keyframes[0]!.ruleRange.end;
    for (const keyframes of styleDocument.keyframes) {
      const end = keyframes.ruleRange.end;
      if (
        end.line > latest.line ||
        (end.line === latest.line && end.character > latest.character)
      ) {
        latest = end;
      }
    }
    return {
      start: { line: latest.line, character: latest.character },
      end: { line: latest.line, character: latest.character },
    };
  }

  const valueFacts = [...styleDocument.valueDecls, ...styleDocument.valueImports];
  if (valueFacts.length > 0) {
    let latest = valueFacts[0]!.ruleRange.end;
    for (const valueFact of valueFacts) {
      const end = valueFact.ruleRange.end;
      if (
        end.line > latest.line ||
        (end.line === latest.line && end.character > latest.character)
      ) {
        latest = end;
      }
    }
    return {
      start: { line: latest.line, character: latest.character },
      end: { line: latest.line, character: latest.character },
    };
  }

  return {
    start: { line: 0, character: 0 },
    end: { line: 0, character: 0 },
  };
}
