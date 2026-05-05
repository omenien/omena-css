import path from "node:path";
import { explainExpressionAtLocation } from "../server/engine-host-node/src/explain-expression";

interface ParsedArgs {
  readonly workspaceRoot: string;
  readonly filePath: string;
  readonly line: number;
  readonly character: number;
  readonly json: boolean;
}

void (async () => {
  const parsed = parseArgs(process.argv.slice(2), process.cwd());
  if ("error" in parsed) {
    process.stderr.write(`${parsed.error}\n`);
    process.stderr.write(buildHelpText());
    process.exitCode = 2;
    return;
  }
  if ("helpText" in parsed) {
    process.stdout.write(parsed.helpText);
    return;
  }

  const result = explainExpressionAtLocation(parsed);
  if (!result) {
    process.stderr.write("No explainable source class expression found at the given location.\n");
    process.exitCode = 1;
    return;
  }

  if (parsed.json) {
    process.stdout.write(`${JSON.stringify(result, null, 2)}\n`);
    return;
  }

  process.stdout.write(formatExplainResult(result, parsed.workspaceRoot));
})();

function parseArgs(
  argv: readonly string[],
  cwd: string,
): ParsedArgs | { readonly error: string } | { readonly helpText: string } {
  let workspaceRoot = cwd;
  let locationArg: string | null = null;
  let json = false;

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index]!;
    if (arg === "--") continue;
    if (arg === "--help" || arg === "-h") return { helpText: buildHelpText() };
    if (arg === "--json") {
      json = true;
      continue;
    }
    if (arg === "--root") {
      const value = argv[index + 1];
      if (!value) return { error: "Missing value for --root" };
      workspaceRoot = path.resolve(cwd, value);
      index += 1;
      continue;
    }
    if (!locationArg) {
      locationArg = arg;
      continue;
    }
    return { error: `Unexpected argument: ${arg}` };
  }

  if (!locationArg) return { helpText: buildHelpText() };
  const location = parseLocationArg(locationArg, workspaceRoot);
  if ("error" in location) return location;

  return {
    workspaceRoot,
    filePath: location.filePath,
    line: location.line,
    character: location.character,
    json,
  };
}

function parseLocationArg(
  value: string,
  workspaceRoot: string,
):
  | { readonly filePath: string; readonly line: number; readonly character: number }
  | { readonly error: string } {
  const lastColon = value.lastIndexOf(":");
  if (lastColon < 0) return { error: "Expected <file>:<line>:<column>" };
  const secondLastColon = value.lastIndexOf(":", lastColon - 1);
  if (secondLastColon < 0) return { error: "Expected <file>:<line>:<column>" };

  const filePart = value.slice(0, secondLastColon);
  const linePart = value.slice(secondLastColon + 1, lastColon);
  const characterPart = value.slice(lastColon + 1);
  const line = Number.parseInt(linePart, 10);
  const character = Number.parseInt(characterPart, 10);

  if (!filePart) return { error: "Expected <file>:<line>:<column>" };
  if (!Number.isInteger(line) || line < 1) return { error: "Line must be a positive integer." };
  if (!Number.isInteger(character) || character < 1) {
    return { error: "Column must be a positive integer." };
  }

  return {
    filePath: path.resolve(workspaceRoot, filePart),
    line: line - 1,
    character: character - 1,
  };
}

function formatExplainResult(
  result: {
    readonly filePath: string;
    readonly line: number;
    readonly character: number;
    readonly expressionKind: string;
    readonly styleFilePath: string;
    readonly selectorNames: readonly string[];
    readonly analysisV2: {
      readonly valueDomainKind: string;
      readonly valueConstraintKind?: string;
      readonly valuePrefix?: string;
      readonly valueSuffix?: string;
      readonly valueMinLen?: number;
      readonly valueMaxLen?: number;
      readonly valueCharMust?: string;
      readonly valueCharMay?: string;
      readonly valueMayIncludeOtherChars?: boolean;
      readonly valueDomainDerivation?: {
        readonly inputFactKind: string;
        readonly inputConstraintKind?: string;
        readonly inputValueCount: number;
        readonly reducedKind: string;
        readonly steps: readonly {
          readonly operation: string;
          readonly inputKind?: string;
          readonly refinementKind?: string;
          readonly resultKind: string;
          readonly reason: string;
        }[];
      };
      readonly valueDomainProvenanceTree?: {
        readonly root: {
          readonly operation: string;
          readonly resultKind: string;
          readonly resultProvenance?: string;
          readonly detail?: string;
          readonly reason: string;
          readonly children: readonly {
            readonly operation: string;
            readonly resultKind: string;
            readonly resultProvenance?: string;
            readonly detail?: string;
            readonly reason: string;
          }[];
        };
      };
      readonly valueCertaintyShapeKind?: string;
      readonly valueCertaintyConstraintKind?: string;
      readonly selectorCertaintyShapeKind?: string;
      readonly selectorConstraintKind?: string;
    };
    readonly dynamicExplanation: {
      readonly kind: string;
      readonly subject: string;
      readonly candidates: readonly string[];
      readonly valueDomainLabel?: string;
      readonly valueDomainReasonLabel?: string;
      readonly valueDomainDerivationLabel?: string;
      readonly valueDomainDerivationStepLabels?: readonly string[];
      readonly valueDomainProvenanceLabel?: string;
      readonly valueDomainProvenanceStepLabels?: readonly string[];
      readonly valueCertainty?: string;
      readonly valueCertaintyShapeLabel?: string;
      readonly valueCertaintyReasonLabel?: string;
      readonly selectorCertainty?: string;
      readonly selectorCertaintyShapeLabel?: string;
      readonly selectorCertaintyReasonLabel?: string;
      readonly reasonLabel?: string;
    } | null;
  },
  workspaceRoot: string,
): string {
  const lines = [
    `File: ${relativeOrAbsolute(result.filePath, workspaceRoot)}:${result.line + 1}:${result.character + 1}`,
    `Expression kind: ${result.expressionKind}`,
    `Style module: ${relativeOrAbsolute(result.styleFilePath, workspaceRoot)}`,
    `Matched selectors: ${result.selectorNames.length > 0 ? result.selectorNames.join(", ") : "(none)"}`,
  ];

  if (result.dynamicExplanation) {
    lines.push(`Reference subject: ${result.dynamicExplanation.subject}`);
    if (result.dynamicExplanation.reasonLabel) {
      lines.push(`Resolution reason: ${result.dynamicExplanation.reasonLabel}`);
    }
    if (result.dynamicExplanation.candidates.length > 0) {
      lines.push(`Candidates: ${result.dynamicExplanation.candidates.join(", ")}`);
    }
    if (result.dynamicExplanation.valueDomainLabel) {
      lines.push(`Value domain: ${result.dynamicExplanation.valueDomainLabel}`);
    }
    if (result.dynamicExplanation.valueDomainReasonLabel) {
      lines.push(`Value domain reason: ${result.dynamicExplanation.valueDomainReasonLabel}`);
    }
    if (result.dynamicExplanation.valueDomainDerivationLabel) {
      lines.push(
        `Value domain derivation: ${result.dynamicExplanation.valueDomainDerivationLabel}`,
      );
    }
    for (const step of result.dynamicExplanation.valueDomainDerivationStepLabels ?? []) {
      lines.push(`Value domain derivation step: ${step}`);
    }
    if (result.dynamicExplanation.valueDomainProvenanceLabel) {
      lines.push(
        `Value domain provenance: ${result.dynamicExplanation.valueDomainProvenanceLabel}`,
      );
    }
    for (const step of result.dynamicExplanation.valueDomainProvenanceStepLabels ?? []) {
      lines.push(`Value domain provenance step: ${step}`);
    }
    if (result.dynamicExplanation.valueCertainty) {
      lines.push(`Value certainty: ${result.dynamicExplanation.valueCertainty}`);
    }
    if (result.dynamicExplanation.valueCertaintyShapeLabel) {
      lines.push(`Value certainty shape: ${result.dynamicExplanation.valueCertaintyShapeLabel}`);
    }
    if (result.dynamicExplanation.valueCertaintyReasonLabel) {
      lines.push(`Value certainty reason: ${result.dynamicExplanation.valueCertaintyReasonLabel}`);
    }
    if (result.dynamicExplanation.selectorCertainty) {
      lines.push(`Selector certainty: ${result.dynamicExplanation.selectorCertainty}`);
    }
    if (result.dynamicExplanation.selectorCertaintyShapeLabel) {
      lines.push(
        `Selector certainty shape: ${result.dynamicExplanation.selectorCertaintyShapeLabel}`,
      );
    }
    if (result.dynamicExplanation.selectorCertaintyReasonLabel) {
      lines.push(
        `Selector certainty reason: ${result.dynamicExplanation.selectorCertaintyReasonLabel}`,
      );
    }
  }

  lines.push(`V2 value domain kind: ${result.analysisV2.valueDomainKind}`);
  if (result.analysisV2.valueConstraintKind) {
    lines.push(`V2 value constraint kind: ${result.analysisV2.valueConstraintKind}`);
  }
  if (result.analysisV2.valuePrefix) {
    lines.push(`V2 value prefix: ${result.analysisV2.valuePrefix}`);
  }
  if (result.analysisV2.valueSuffix) {
    lines.push(`V2 value suffix: ${result.analysisV2.valueSuffix}`);
  }
  if (result.analysisV2.valueMinLen !== undefined) {
    lines.push(`V2 value min length: ${result.analysisV2.valueMinLen}`);
  }
  if (result.analysisV2.valueMaxLen !== undefined) {
    lines.push(`V2 value max length: ${result.analysisV2.valueMaxLen}`);
  }
  if (result.analysisV2.valueCharMust) {
    lines.push(`V2 value char must: ${result.analysisV2.valueCharMust}`);
  }
  if (result.analysisV2.valueCharMay) {
    lines.push(`V2 value char may: ${result.analysisV2.valueCharMay}`);
  }
  if (result.analysisV2.valueMayIncludeOtherChars) {
    lines.push("V2 value may include other chars: true");
  }
  if (result.analysisV2.valueDomainDerivation) {
    const derivation = result.analysisV2.valueDomainDerivation;
    const inputKind = derivation.inputConstraintKind
      ? `${derivation.inputFactKind}/${derivation.inputConstraintKind}`
      : derivation.inputFactKind;
    lines.push(`V2 value derivation: ${inputKind} -> ${derivation.reducedKind}`);
  }
  if (result.analysisV2.valueDomainProvenanceTree) {
    const root = result.analysisV2.valueDomainProvenanceTree.root;
    const resultKind = root.resultProvenance
      ? `${root.resultKind}/${root.resultProvenance}`
      : root.resultKind;
    lines.push(`V2 value provenance: ${root.operation}: ${resultKind} (${root.reason})`);
    if (root.detail) {
      lines.push(`V2 value provenance detail: ${root.detail}`);
    }
    for (const child of root.children) {
      const childResult = child.resultProvenance
        ? `${child.resultKind}/${child.resultProvenance}`
        : child.resultKind;
      const childDetail = child.detail ? ` ${child.detail}` : "";
      lines.push(
        `V2 value provenance constraint: ${child.operation}:${childDetail} -> ${childResult} (${child.reason})`,
      );
    }
  }
  if (result.analysisV2.valueCertaintyShapeKind) {
    lines.push(`V2 value certainty shape kind: ${result.analysisV2.valueCertaintyShapeKind}`);
  }
  if (result.analysisV2.valueCertaintyConstraintKind) {
    lines.push(
      `V2 value certainty constraint kind: ${result.analysisV2.valueCertaintyConstraintKind}`,
    );
  }
  if (result.analysisV2.selectorCertaintyShapeKind) {
    lines.push(`V2 selector certainty shape kind: ${result.analysisV2.selectorCertaintyShapeKind}`);
  }
  if (result.analysisV2.selectorConstraintKind) {
    lines.push(`V2 selector constraint kind: ${result.analysisV2.selectorConstraintKind}`);
  }

  return `${lines.join("\n")}\n`;
}

function relativeOrAbsolute(filePath: string, workspaceRoot: string): string {
  const relativePath = path.relative(workspaceRoot, filePath);
  if (!relativePath || relativePath.startsWith("..")) return filePath;
  return relativePath;
}

function buildHelpText(): string {
  return [
    "Usage: pnpm explain:expression -- <file>:<line>:<column> [options]",
    "",
    "Options:",
    "  --root <path>   Workspace root (defaults to cwd)",
    "  --json          Emit JSON instead of text",
    "  --help, -h      Show this help text",
    "",
  ].join("\n");
}
