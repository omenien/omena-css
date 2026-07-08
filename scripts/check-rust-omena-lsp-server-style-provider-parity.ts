import { spawnSync } from "node:child_process";
import { strict as assert } from "node:assert";
import { resolveOmenaLspServerInvocation } from "./omena-lsp-server-invocation";
import {
  findCustomPropertyDeclAtCursor,
  findCustomPropertyRefAtCursor,
  findSelectorAtCursor,
} from "../server/engine-core-ts/src/core/query";
import { parseStyleDocument } from "../server/engine-core-ts/src/core/scss/scss-parser";

const workspaceUri = "file:///tmp/cme-rust-lsp-style-provider";
const stylePath = "/tmp/cme-rust-lsp-style-provider/src/App.module.scss";
const otherStylePath = "/tmp/cme-rust-lsp-style-provider/src/Other.module.scss";
const styleUri = `${workspaceUri}/src/App.module.scss`;
const otherStyleUri = `${workspaceUri}/src/Other.module.scss`;
const sourceUri = `${workspaceUri}/src/App.tsx`;
const sourceText =
  'import styles from "./App.module.scss";\nconst view = <div className={styles.root} />;\nconst bracket = styles["theme"];\nconst bracketMissing = styles["ghost"];\nconst utility = <div className={clsx("alert")} />;\nconst conditional = <div className={active ? "conditional" : ""} />;\nconst missing = <div className="missing" />;';
const sourceSelectorRange = {
  start: { line: 1, character: 36 },
  end: { line: 1, character: 40 },
};
const sourceClassValueDomainRange = {
  start: { line: 1, character: 29 },
  end: { line: 1, character: 40 },
};
const sourceBracketSelectorRange = {
  start: { line: 2, character: 24 },
  end: { line: 2, character: 29 },
};
const sourceMissingSelectorRange = {
  start: { line: 6, character: 32 },
  end: { line: 6, character: 39 },
};
const sourceMissingImportedSelectorRange = {
  start: { line: 3, character: 31 },
  end: { line: 3, character: 36 },
};
const sourceUtilitySelectorRange = {
  start: { line: 4, character: 38 },
  end: { line: 4, character: 43 },
};
const sourceConditionalSelectorRange = {
  start: { line: 5, character: 46 },
  end: { line: 5, character: 57 },
};
const styleText =
  ".root { color: var(--brand); }\n.theme { --brand: red; }\n.alert { color: var(--missing); }\n.conditional { color: green; }";
const otherStyleText =
  ".root { color: blue; }\n.theme { color: green; }\n.ghost { color: gray; }\n.card { color: green; }";
const sourceSelectorQueryPosition = {
  line: 1,
  character: 37,
};
const sourceBracketSelectorQueryPosition = {
  line: 2,
  character: 25,
};
const sourceUtilitySelectorQueryPosition = {
  line: 4,
  character: 39,
};
const sourceConditionalSelectorQueryPosition = {
  line: 5,
  character: 47,
};
const selectorQueryPosition = {
  line: 0,
  character: 2,
};
const themeSelectorQueryPosition = {
  line: 1,
  character: 2,
};
const customPropertyReferenceQueryPosition = {
  line: 0,
  character: 21,
};
const customPropertyDeclarationQueryPosition = {
  line: 1,
  character: 11,
};
const missingCustomPropertyReferenceQueryPosition = {
  line: 2,
  character: 22,
};

const nodeStyleDocument = parseStyleDocument(styleText, stylePath);
const nodeOtherStyleDocument = parseStyleDocument(otherStyleText, otherStylePath);
const nodeSelector = findSelectorAtCursor(
  nodeStyleDocument,
  selectorQueryPosition.line,
  selectorQueryPosition.character,
);
assert.ok(nodeSelector, "node selector fixture did not produce a hover target");
const nodeThemeSelector = findSelectorAtCursor(
  nodeStyleDocument,
  themeSelectorQueryPosition.line,
  themeSelectorQueryPosition.character,
);
assert.ok(nodeThemeSelector, "node theme selector fixture did not produce a hover target");
const nodeAlertSelector = findSelectorAtCursor(nodeStyleDocument, 2, 2);
assert.ok(nodeAlertSelector, "node alert selector fixture did not produce a hover target");
const nodeConditionalSelector = findSelectorAtCursor(nodeStyleDocument, 3, 2);
assert.ok(
  nodeConditionalSelector,
  "node conditional selector fixture did not produce a hover target",
);
const nodeOtherRootSelector = findSelectorAtCursor(nodeOtherStyleDocument, 0, 2);
assert.ok(nodeOtherRootSelector, "node other root selector fixture did not produce a target");
const nodeOtherThemeSelector = findSelectorAtCursor(nodeOtherStyleDocument, 1, 2);
assert.ok(nodeOtherThemeSelector, "node other theme selector fixture did not produce a target");
const nodeOtherGhostSelector = findSelectorAtCursor(nodeOtherStyleDocument, 2, 2);
assert.ok(nodeOtherGhostSelector, "node other ghost selector fixture did not produce a target");
const nodeOtherCardSelector = findSelectorAtCursor(nodeOtherStyleDocument, 3, 2);
assert.ok(nodeOtherCardSelector, "node other card selector fixture did not produce a target");
const nodeCustomPropertyReference = findCustomPropertyRefAtCursor(
  nodeStyleDocument,
  customPropertyReferenceQueryPosition.line,
  customPropertyReferenceQueryPosition.character,
);
assert.ok(
  nodeCustomPropertyReference,
  "node custom property reference fixture did not produce a hover target",
);
const nodeCustomPropertyDeclaration = findCustomPropertyDeclAtCursor(
  nodeStyleDocument,
  customPropertyDeclarationQueryPosition.line,
  customPropertyDeclarationQueryPosition.character,
);
assert.ok(
  nodeCustomPropertyDeclaration,
  "node custom property declaration fixture did not produce a hover target",
);
const nodeMissingCustomPropertyReference = findCustomPropertyRefAtCursor(
  nodeStyleDocument,
  missingCustomPropertyReferenceQueryPosition.line,
  missingCustomPropertyReferenceQueryPosition.character,
);
assert.ok(
  nodeMissingCustomPropertyReference,
  "node missing custom property fixture did not produce a hover target",
);
const expectedMissingCustomPropertyDiagnostic = {
  range: nodeMissingCustomPropertyReference.range,
  severity: 2,
  source: "omena-css",
  code: "missingCustomProperty",
  message: "CSS custom property '--missing' not found in indexed style tokens.",
  data: {
    querySeverity: "warning",
    provenance: [
      "omena-parser.custom-property-facts",
      "omena-query.style-diagnostics",
      "omena-query-checker-orchestrator.product-diagnostic-gate",
      "omena-checker.rule-registry",
    ],
    polynomialProvenance: polynomialProvenanceFor([
      "omena-parser.custom-property-facts",
      "omena-query.style-diagnostics",
      "omena-query-checker-orchestrator.product-diagnostic-gate",
      "omena-checker.rule-registry",
    ]),
    createCustomProperty: {
      uri: styleUri,
      range: documentEndRange(styleText),
      newText: "\n\n:root {\n  --missing: ;\n}\n",
      propertyName: "--missing",
    },
  },
};
const expectedMissingSelectorDiagnostic = {
  range: sourceMissingSelectorRange,
  severity: 2,
  source: "omena-css",
  code: "missingSelector",
  message: "CSS Module selector '.missing' not found in indexed style tokens.",
  data: {
    querySeverity: "warning",
    provenance: [
      "omena-query.source-syntax-index",
      "omena-query.style-selector-definitions",
      "omena-query-checker-orchestrator.product-diagnostic-gate",
      "omena-checker.rule-registry",
    ],
    precision: sourceDiagnosticPrecision("classValueResolution", "sourceSyntaxIndex"),
    createSelector: {
      uri: styleUri,
      range: documentEndRange(styleText),
      newText: "\n\n.missing {\n}\n",
      selectorName: "missing",
    },
  },
};
const expectedMissingImportedStaticClassDiagnostic = {
  range: sourceMissingImportedSelectorRange,
  severity: 2,
  source: "omena-css",
  code: "missingStaticClass",
  message: "Class '.ghost' not found in target CSS Module. Did you mean 'root'?",
  data: {
    querySeverity: "warning",
    provenance: [
      "omena-query.source-syntax-index",
      "omena-query.style-selector-definitions",
      "omena-query-checker-orchestrator.product-diagnostic-gate",
      "omena-checker.rule-registry",
    ],
    precision: sourceDiagnosticPrecision("classValueResolution", "sourceSelectorReference"),
    createSelector: {
      uri: styleUri,
      range: documentEndRange(styleText),
      newText: "\n\n.ghost {\n}\n",
      selectorName: "ghost",
    },
  },
};
const expectedUnknownClassValueDomainDiagnostic = {
  range: sourceClassValueDomainRange,
  severity: 2,
  source: "omena-css",
  code: "unknownClassValueDomain",
  message:
    "CSS Module class value domain is unknown because tsgo could not find a project for this source. Dynamic class values in this file (1 site) are not checked until the provider is available.",
  data: {
    querySeverity: "warning",
    provenance: [
      "omena-query.source-syntax-index",
      "omena-tsgo-client.provider-capabilities",
      "tsgo-provider.unavailable->unknown-precision",
    ],
    precision: sourceDiagnosticPrecision(
      "unknown",
      "typeOracleProviderUnavailable",
      "perTypeFactTarget",
    ),
  },
};
const expectedUnusedAlertSelectorDiagnostic = unusedSelectorDiagnostic(nodeAlertSelector);
const expectedUnusedConditionalSelectorDiagnostic =
  unusedSelectorDiagnostic(nodeConditionalSelector);
const expectedUnusedOtherRootSelectorDiagnostic = unusedSelectorDiagnostic(nodeOtherRootSelector);
const expectedUnusedOtherThemeSelectorDiagnostic = unusedSelectorDiagnostic(nodeOtherThemeSelector);
const expectedUnusedOtherGhostSelectorDiagnostic = unusedSelectorDiagnostic(nodeOtherGhostSelector);
const expectedUnusedOtherCardSelectorDiagnostic = unusedSelectorDiagnostic(nodeOtherCardSelector);
const expectedMissingCustomPropertyDiagnosticWithSnapshot = diagnosticWithWorkspaceSnapshotId(
  expectedMissingCustomPropertyDiagnostic,
  1,
);
const expectedUnusedAlertSelectorDiagnosticWithSnapshot = diagnosticWithWorkspaceSnapshotId(
  expectedUnusedAlertSelectorDiagnostic,
  1,
);
const expectedUnusedConditionalSelectorDiagnosticWithSnapshot = diagnosticWithWorkspaceSnapshotId(
  expectedUnusedConditionalSelectorDiagnostic,
  1,
);
const expectedAppStyleDiagnostics = [
  expectedMissingCustomPropertyDiagnosticWithSnapshot,
  expectedUnusedAlertSelectorDiagnosticWithSnapshot,
  expectedUnusedConditionalSelectorDiagnosticWithSnapshot,
];
const expectedPublishedMissingCustomPropertyDiagnostic = diagnosticWithTier(
  expectedMissingCustomPropertyDiagnosticWithSnapshot,
  "baseline",
  "fastFactsV0",
);
const expectedPublishedMissingSelectorDiagnostic = diagnosticWithTier(
  expectedMissingSelectorDiagnostic,
  "baseline",
  "sourceSyntaxIndexV0",
);
const expectedPublishedMissingImportedStaticClassDiagnostic = diagnosticWithTier(
  expectedMissingImportedStaticClassDiagnostic,
  "baseline",
  "sourceSyntaxIndexV0",
);
const expectedPublishedUnusedAlertSelectorDiagnostic = diagnosticWithTier(
  expectedUnusedAlertSelectorDiagnosticWithSnapshot,
  "optimizing",
  "analyzedGraphV0",
);
const expectedPublishedUnusedConditionalSelectorDiagnostic = diagnosticWithTier(
  expectedUnusedConditionalSelectorDiagnosticWithSnapshot,
  "optimizing",
  "analyzedGraphV0",
);
const expectedPublishedUnusedOtherRootSelectorDiagnostic = diagnosticWithTier(
  diagnosticWithWorkspaceSnapshotId(expectedUnusedOtherRootSelectorDiagnostic, 2),
  "optimizing",
  "analyzedGraphV0",
);
const expectedPublishedUnusedOtherThemeSelectorDiagnostic = diagnosticWithTier(
  diagnosticWithWorkspaceSnapshotId(expectedUnusedOtherThemeSelectorDiagnostic, 2),
  "optimizing",
  "analyzedGraphV0",
);
const expectedPublishedUnusedOtherGhostSelectorDiagnostic = diagnosticWithTier(
  diagnosticWithWorkspaceSnapshotId(expectedUnusedOtherGhostSelectorDiagnostic, 2),
  "optimizing",
  "analyzedGraphV0",
);
const expectedPublishedUnusedOtherCardSelectorDiagnostic = diagnosticWithTier(
  diagnosticWithWorkspaceSnapshotId(expectedUnusedOtherCardSelectorDiagnostic, 2),
  "optimizing",
  "analyzedGraphV0",
);
const expectedPublishedUnknownClassValueDomainDiagnostic = diagnosticWithTier(
  expectedUnknownClassValueDomainDiagnostic,
  "optimizing",
  "workspaceSourceDiagnosticsV0",
);
const expectedPublishedAppStyleDiagnostics = [
  expectedPublishedMissingCustomPropertyDiagnostic,
  expectedPublishedUnusedAlertSelectorDiagnostic,
  expectedPublishedUnusedConditionalSelectorDiagnostic,
];
const expectedPublishedOtherStyleDiagnostics = [
  expectedPublishedUnusedOtherRootSelectorDiagnostic,
  expectedPublishedUnusedOtherThemeSelectorDiagnostic,
  expectedPublishedUnusedOtherGhostSelectorDiagnostic,
  expectedPublishedUnusedOtherCardSelectorDiagnostic,
];
const expectedPublishedSourceDiagnostics = [
  expectedPublishedMissingImportedStaticClassDiagnostic,
  expectedPublishedMissingSelectorDiagnostic,
];
const expectedPublishedSourceDiagnosticsWithUnknownTypeFacts = [
  expectedPublishedUnknownClassValueDomainDiagnostic,
  ...expectedPublishedSourceDiagnostics,
];

const initializeRequest = {
  jsonrpc: "2.0",
  id: 1,
  method: "initialize",
  params: {
    processId: null,
    rootUri: workspaceUri,
    workspaceFolders: [
      {
        uri: workspaceUri,
        name: "cme-rust-lsp-style-provider",
      },
    ],
    capabilities: {},
  },
};
const didOpenStyleNotification = {
  jsonrpc: "2.0",
  method: "textDocument/didOpen",
  params: {
    textDocument: {
      uri: styleUri,
      languageId: "scss",
      version: 1,
      text: styleText,
    },
  },
};
const didOpenOtherStyleNotification = {
  jsonrpc: "2.0",
  method: "textDocument/didOpen",
  params: {
    textDocument: {
      uri: otherStyleUri,
      languageId: "scss",
      version: 1,
      text: otherStyleText,
    },
  },
};
const didOpenSourceNotification = {
  jsonrpc: "2.0",
  method: "textDocument/didOpen",
  params: {
    textDocument: {
      uri: sourceUri,
      languageId: "typescriptreact",
      version: 1,
      text: sourceText,
    },
  },
};
const styleHoverCandidatesRequest = {
  jsonrpc: "2.0",
  id: 2,
  method: "omena/rustStyleHoverCandidates",
  params: {
    textDocument: {
      uri: styleUri,
    },
    position: selectorQueryPosition,
  },
};
const customPropertyReferenceCandidatesRequest = {
  jsonrpc: "2.0",
  id: 3,
  method: "omena/rustStyleHoverCandidates",
  params: {
    textDocument: {
      uri: styleUri,
    },
    position: customPropertyReferenceQueryPosition,
  },
};
const customPropertyDeclarationCandidatesRequest = {
  jsonrpc: "2.0",
  id: 4,
  method: "omena/rustStyleHoverCandidates",
  params: {
    textDocument: {
      uri: styleUri,
    },
    position: customPropertyDeclarationQueryPosition,
  },
};
const lspHoverRequest = {
  jsonrpc: "2.0",
  id: 5,
  method: "textDocument/hover",
  params: {
    textDocument: {
      uri: styleUri,
    },
    position: selectorQueryPosition,
  },
};
const lspDefinitionRequest = {
  jsonrpc: "2.0",
  id: 6,
  method: "textDocument/definition",
  params: {
    textDocument: {
      uri: styleUri,
    },
    position: customPropertyReferenceQueryPosition,
  },
};
const lspReferencesRequest = {
  jsonrpc: "2.0",
  id: 7,
  method: "textDocument/references",
  params: {
    textDocument: {
      uri: styleUri,
    },
    position: customPropertyReferenceQueryPosition,
    context: {
      includeDeclaration: true,
    },
  },
};
const lspCompletionRequest = {
  jsonrpc: "2.0",
  id: 8,
  method: "textDocument/completion",
  params: {
    textDocument: {
      uri: styleUri,
    },
    position: customPropertyReferenceQueryPosition,
  },
};
const styleDiagnosticsRequest = {
  jsonrpc: "2.0",
  id: 9,
  method: "omena/rustStyleDiagnostics",
  params: {
    textDocument: {
      uri: styleUri,
    },
  },
};
const lspCodeActionRequest = {
  jsonrpc: "2.0",
  id: 10,
  method: "textDocument/codeAction",
  params: {
    textDocument: {
      uri: styleUri,
    },
    range: nodeMissingCustomPropertyReference.range,
    context: {
      diagnostics: [expectedMissingCustomPropertyDiagnostic],
    },
  },
};
const lspPrepareRenameRequest = {
  jsonrpc: "2.0",
  id: 11,
  method: "textDocument/prepareRename",
  params: {
    textDocument: {
      uri: styleUri,
    },
    position: selectorQueryPosition,
  },
};
const lspRenameRequest = {
  jsonrpc: "2.0",
  id: 12,
  method: "textDocument/rename",
  params: {
    textDocument: {
      uri: styleUri,
    },
    position: customPropertyReferenceQueryPosition,
    newName: "--accent",
  },
};
const lspCodeLensRequest = {
  jsonrpc: "2.0",
  id: 13,
  method: "textDocument/codeLens",
  params: {
    textDocument: {
      uri: styleUri,
    },
  },
};
const lspSourceHoverRequest = {
  jsonrpc: "2.0",
  id: 14,
  method: "textDocument/hover",
  params: {
    textDocument: {
      uri: sourceUri,
    },
    position: sourceSelectorQueryPosition,
  },
};
const lspSourceDefinitionRequest = {
  jsonrpc: "2.0",
  id: 15,
  method: "textDocument/definition",
  params: {
    textDocument: {
      uri: sourceUri,
    },
    position: sourceSelectorQueryPosition,
  },
};
const lspSourceReferencesRequest = {
  jsonrpc: "2.0",
  id: 16,
  method: "textDocument/references",
  params: {
    textDocument: {
      uri: sourceUri,
    },
    position: sourceSelectorQueryPosition,
    context: {
      includeDeclaration: true,
    },
  },
};
const lspSourceCompletionRequest = {
  jsonrpc: "2.0",
  id: 17,
  method: "textDocument/completion",
  params: {
    textDocument: {
      uri: sourceUri,
    },
    position: sourceSelectorQueryPosition,
  },
};
const lspSourcePrepareRenameRequest = {
  jsonrpc: "2.0",
  id: 18,
  method: "textDocument/prepareRename",
  params: {
    textDocument: {
      uri: sourceUri,
    },
    position: sourceSelectorQueryPosition,
  },
};
const lspSourceRenameRequest = {
  jsonrpc: "2.0",
  id: 19,
  method: "textDocument/rename",
  params: {
    textDocument: {
      uri: sourceUri,
    },
    position: sourceSelectorQueryPosition,
    newName: "panel",
  },
};
const lspSourceCodeActionRequest = {
  jsonrpc: "2.0",
  id: 20,
  method: "textDocument/codeAction",
  params: {
    textDocument: {
      uri: sourceUri,
    },
    range: sourceMissingSelectorRange,
    context: {
      diagnostics: [expectedMissingSelectorDiagnostic],
    },
  },
};
const lspStyleSelectorRenameRequest = {
  jsonrpc: "2.0",
  id: 21,
  method: "textDocument/rename",
  params: {
    textDocument: {
      uri: styleUri,
    },
    position: selectorQueryPosition,
    newName: "panel",
  },
};
const lspSourceNoopCompletionRequest = {
  jsonrpc: "2.0",
  id: 22,
  method: "textDocument/completion",
  params: {
    textDocument: {
      uri: sourceUri,
    },
    position: {
      line: 0,
      character: 5,
    },
  },
};
const lspSourceBracketHoverRequest = {
  jsonrpc: "2.0",
  id: 23,
  method: "textDocument/hover",
  params: {
    textDocument: {
      uri: sourceUri,
    },
    position: sourceBracketSelectorQueryPosition,
  },
};
const lspSourceBracketDefinitionRequest = {
  jsonrpc: "2.0",
  id: 24,
  method: "textDocument/definition",
  params: {
    textDocument: {
      uri: sourceUri,
    },
    position: sourceBracketSelectorQueryPosition,
  },
};
const lspSourceBracketReferencesRequest = {
  jsonrpc: "2.0",
  id: 25,
  method: "textDocument/references",
  params: {
    textDocument: {
      uri: sourceUri,
    },
    position: sourceBracketSelectorQueryPosition,
    context: {
      includeDeclaration: true,
    },
  },
};
const lspSourceBracketCompletionRequest = {
  jsonrpc: "2.0",
  id: 26,
  method: "textDocument/completion",
  params: {
    textDocument: {
      uri: sourceUri,
    },
    position: sourceBracketSelectorQueryPosition,
  },
};
const lspSourceBracketRenameRequest = {
  jsonrpc: "2.0",
  id: 27,
  method: "textDocument/rename",
  params: {
    textDocument: {
      uri: sourceUri,
    },
    position: sourceBracketSelectorQueryPosition,
    newName: "surface",
  },
};
const lspSourceUtilityHoverRequest = {
  jsonrpc: "2.0",
  id: 28,
  method: "textDocument/hover",
  params: {
    textDocument: {
      uri: sourceUri,
    },
    position: sourceUtilitySelectorQueryPosition,
  },
};
const lspSourceUtilityDefinitionRequest = {
  jsonrpc: "2.0",
  id: 29,
  method: "textDocument/definition",
  params: {
    textDocument: {
      uri: sourceUri,
    },
    position: sourceUtilitySelectorQueryPosition,
  },
};
const lspSourceUtilityReferencesRequest = {
  jsonrpc: "2.0",
  id: 30,
  method: "textDocument/references",
  params: {
    textDocument: {
      uri: sourceUri,
    },
    position: sourceUtilitySelectorQueryPosition,
    context: {
      includeDeclaration: true,
    },
  },
};
const lspSourceUtilityCompletionRequest = {
  jsonrpc: "2.0",
  id: 31,
  method: "textDocument/completion",
  params: {
    textDocument: {
      uri: sourceUri,
    },
    position: sourceUtilitySelectorQueryPosition,
  },
};
const lspSourceUtilityRenameRequest = {
  jsonrpc: "2.0",
  id: 32,
  method: "textDocument/rename",
  params: {
    textDocument: {
      uri: sourceUri,
    },
    position: sourceUtilitySelectorQueryPosition,
    newName: "notice",
  },
};
const lspSourceConditionalHoverRequest = {
  jsonrpc: "2.0",
  id: 33,
  method: "textDocument/hover",
  params: {
    textDocument: {
      uri: sourceUri,
    },
    position: sourceConditionalSelectorQueryPosition,
  },
};
const lspSourceConditionalDefinitionRequest = {
  jsonrpc: "2.0",
  id: 34,
  method: "textDocument/definition",
  params: {
    textDocument: {
      uri: sourceUri,
    },
    position: sourceConditionalSelectorQueryPosition,
  },
};
const lspSourceConditionalReferencesRequest = {
  jsonrpc: "2.0",
  id: 35,
  method: "textDocument/references",
  params: {
    textDocument: {
      uri: sourceUri,
    },
    position: sourceConditionalSelectorQueryPosition,
    context: {
      includeDeclaration: true,
    },
  },
};
const lspSourceConditionalCompletionRequest = {
  jsonrpc: "2.0",
  id: 36,
  method: "textDocument/completion",
  params: {
    textDocument: {
      uri: sourceUri,
    },
    position: sourceConditionalSelectorQueryPosition,
  },
};
const lspSourceConditionalRenameRequest = {
  jsonrpc: "2.0",
  id: 37,
  method: "textDocument/rename",
  params: {
    textDocument: {
      uri: sourceUri,
    },
    position: sourceConditionalSelectorQueryPosition,
    newName: "stateful",
  },
};
const shutdownRequest = {
  jsonrpc: "2.0",
  id: 38,
  method: "shutdown",
};
const exitNotification = {
  jsonrpc: "2.0",
  method: "exit",
};

const invocation = resolveOmenaLspServerInvocation();
const result = spawnSync(invocation.command, [...invocation.args], {
  cwd: process.cwd(),
  input: [
    initializeRequest,
    didOpenSourceNotification,
    didOpenStyleNotification,
    didOpenOtherStyleNotification,
    styleHoverCandidatesRequest,
    customPropertyReferenceCandidatesRequest,
    customPropertyDeclarationCandidatesRequest,
    lspHoverRequest,
    lspDefinitionRequest,
    lspReferencesRequest,
    lspCompletionRequest,
    styleDiagnosticsRequest,
    lspCodeActionRequest,
    lspPrepareRenameRequest,
    lspRenameRequest,
    lspCodeLensRequest,
    lspSourceHoverRequest,
    lspSourceDefinitionRequest,
    lspSourceReferencesRequest,
    lspSourceCompletionRequest,
    lspSourcePrepareRenameRequest,
    lspSourceRenameRequest,
    lspSourceCodeActionRequest,
    lspStyleSelectorRenameRequest,
    lspSourceNoopCompletionRequest,
    lspSourceBracketHoverRequest,
    lspSourceBracketDefinitionRequest,
    lspSourceBracketReferencesRequest,
    lspSourceBracketCompletionRequest,
    lspSourceBracketRenameRequest,
    lspSourceUtilityHoverRequest,
    lspSourceUtilityDefinitionRequest,
    lspSourceUtilityReferencesRequest,
    lspSourceUtilityCompletionRequest,
    lspSourceUtilityRenameRequest,
    lspSourceConditionalHoverRequest,
    lspSourceConditionalDefinitionRequest,
    lspSourceConditionalReferencesRequest,
    lspSourceConditionalCompletionRequest,
    lspSourceConditionalRenameRequest,
    shutdownRequest,
    exitNotification,
  ]
    .map(frame)
    .join(""),
  encoding: "utf8",
  stdio: ["pipe", "pipe", "pipe"],
});

assert.equal(
  result.status,
  0,
  [
    "omena-lsp-server style provider parity failed",
    result.error ? `error=${result.error.message}` : null,
    result.stderr.trim() ? `stderr=${result.stderr.trim()}` : null,
  ]
    .filter(Boolean)
    .join("\n"),
);

const messages = readFrames(result.stdout);
// RFC 0009 Pillar A (rfcs#67): hover/definition responses are produced by the
// query worker and may arrive out of arrival-order relative to synchronous
// responses — the LSP permits that. Sort by id so the positional assertions
// below keep checking response CONTENT, not transport interleaving.
const responses = messages
  .filter((message) => "id" in message)
  .sort((a, b) => Number(a.id) - Number(b.id));
const diagnosticNotifications = messages.filter(
  (message) => message.method === "textDocument/publishDiagnostics",
);
assert.equal(responses.length, 38);
const expectedDiagnosticNotifications = [
  {
    jsonrpc: "2.0",
    method: "textDocument/publishDiagnostics",
    params: {
      uri: styleUri,
      diagnostics: [expectedPublishedMissingCustomPropertyDiagnostic],
    },
  },
  {
    jsonrpc: "2.0",
    method: "textDocument/publishDiagnostics",
    params: {
      uri: styleUri,
      diagnostics: expectedPublishedAppStyleDiagnostics,
    },
  },
  {
    jsonrpc: "2.0",
    method: "textDocument/publishDiagnostics",
    params: {
      uri: sourceUri,
      diagnostics: expectedPublishedSourceDiagnostics,
    },
  },
  {
    jsonrpc: "2.0",
    method: "textDocument/publishDiagnostics",
    params: {
      uri: otherStyleUri,
      diagnostics: [],
    },
  },
  {
    jsonrpc: "2.0",
    method: "textDocument/publishDiagnostics",
    params: {
      uri: otherStyleUri,
      diagnostics: expectedPublishedOtherStyleDiagnostics,
    },
  },
  {
    jsonrpc: "2.0",
    method: "textDocument/publishDiagnostics",
    params: {
      uri: sourceUri,
      diagnostics: expectedPublishedSourceDiagnosticsWithUnknownTypeFacts,
    },
  },
];
assert.equal(diagnosticNotifications.length, expectedDiagnosticNotifications.length);
assert.deepEqual(
  sortDiagnosticNotifications(diagnosticNotifications),
  sortDiagnosticNotifications(expectedDiagnosticNotifications),
);

const styleHoverResponse = responses[1]!;
assert.equal(styleHoverResponse.id, 2);
assertSingleCandidate(styleHoverResponse, selectorQueryPosition, {
  kind: "selector",
  name: nodeSelector.name,
  range: nodeSelector.range,
  source: "omenaParserSelectorFacts",
});

const customPropertyReferenceResponse = responses[2]!;
assert.equal(customPropertyReferenceResponse.id, 3);
assertSingleCandidate(customPropertyReferenceResponse, customPropertyReferenceQueryPosition, {
  kind: "customPropertyReference",
  name: nodeCustomPropertyReference.name,
  range: nodeCustomPropertyReference.range,
  source: "omenaParserVariableFacts",
});

const customPropertyDeclarationResponse = responses[3]!;
assert.equal(customPropertyDeclarationResponse.id, 4);
assertSingleCandidate(customPropertyDeclarationResponse, customPropertyDeclarationQueryPosition, {
  kind: "customPropertyDeclaration",
  name: nodeCustomPropertyDeclaration.name,
  range: nodeCustomPropertyDeclaration.range,
  source: "omenaParserVariableFacts",
});

const lspHoverResponse = responses[4]!;
assert.equal(lspHoverResponse.id, 5);
assert.deepEqual(lspHoverResponse.result.range, nodeSelector.range);
assert.equal(lspHoverResponse.result.contents.kind, "markdown");
assert.match(lspHoverResponse.result.contents.value, /\.root/);

const lspDefinitionResponse = responses[5]!;
assert.equal(lspDefinitionResponse.id, 6);
assert.deepEqual(lspDefinitionResponse.result, [
  {
    uri: styleUri,
    range: nodeCustomPropertyDeclaration.range,
  },
]);

const lspReferencesResponse = responses[6]!;
assert.equal(lspReferencesResponse.id, 7);
assert.deepEqual(lspReferencesResponse.result, [
  {
    uri: styleUri,
    range: nodeCustomPropertyReference.range,
  },
  {
    uri: styleUri,
    range: nodeCustomPropertyDeclaration.range,
  },
]);

const lspCompletionResponse = responses[7]!;
assert.equal(lspCompletionResponse.id, 8);
assert.equal(lspCompletionResponse.result.isIncomplete, false);
assert.deepEqual(
  lspCompletionResponse.result.items.map((item: { readonly label: string }) => item.label),
  ["--brand"],
);

const styleDiagnosticsResponse = responses[8]!;
assert.equal(styleDiagnosticsResponse.id, 9);
assert.deepEqual(styleDiagnosticsResponse.result, expectedAppStyleDiagnostics);

const lspCodeActionResponse = responses[9]!;
assert.equal(lspCodeActionResponse.id, 10);
assert.deepEqual(lspCodeActionResponse.result, [
  {
    title: "Add '--missing' to App.module.scss",
    kind: "quickfix",
    diagnostics: [expectedMissingCustomPropertyDiagnostic],
    edit: {
      changes: {
        [styleUri]: [
          {
            range: expectedMissingCustomPropertyDiagnostic.data.createCustomProperty.range,
            newText: expectedMissingCustomPropertyDiagnostic.data.createCustomProperty.newText,
          },
        ],
      },
    },
    data: {
      source: "omenaQueryStyleDiagnosticsForFile",
      diagnosticIndex: 0,
    },
  },
  {
    title: "Suppress this diagnostic on the next line",
    kind: "quickfix",
    diagnostics: [expectedMissingCustomPropertyDiagnostic],
    edit: {
      changes: {
        [styleUri]: [
          {
            range: {
              start: {
                line: expectedMissingCustomPropertyDiagnostic.range.start.line,
                character: 0,
              },
              end: {
                line: expectedMissingCustomPropertyDiagnostic.range.start.line,
                character: 0,
              },
            },
            newText: "/* omena-ignore-next-line missingCustomProperty [reason: 'TODO'] */\n",
          },
        ],
      },
    },
    data: {
      source: "omenaLspDiagnosticSuppressionCodeAction",
      diagnosticIndex: 0,
      code: "missingCustomProperty",
    },
  },
  {
    title: "Suppress diagnostics in this block",
    kind: "quickfix",
    diagnostics: [expectedMissingCustomPropertyDiagnostic],
    edit: {
      changes: {
        [styleUri]: [
          {
            range: {
              start: {
                line: expectedMissingCustomPropertyDiagnostic.range.start.line,
                character: 0,
              },
              end: {
                line: expectedMissingCustomPropertyDiagnostic.range.start.line,
                character: 0,
              },
            },
            newText: "/* omena-ignore missingCustomProperty [reason: 'TODO'] */\n",
          },
        ],
      },
    },
    data: {
      source: "omenaLspDiagnosticSuppressionCodeAction",
      diagnosticIndex: 0,
      code: "missingCustomProperty",
      scope: "block",
    },
  },
]);

const lspPrepareRenameResponse = responses[10]!;
assert.equal(lspPrepareRenameResponse.id, 11);
assert.deepEqual(lspPrepareRenameResponse.result, {
  range: nodeSelector.range,
  placeholder: nodeSelector.name,
});

const lspRenameResponse = responses[11]!;
assert.equal(lspRenameResponse.id, 12);
assert.deepEqual(lspRenameResponse.result, {
  changes: {
    [styleUri]: [
      {
        range: nodeCustomPropertyReference.range,
        newText: "--accent",
      },
      {
        range: nodeCustomPropertyDeclaration.range,
        newText: "--accent",
      },
    ],
  },
});

const lspCodeLensResponse = responses[12]!;
assert.equal(lspCodeLensResponse.id, 13);
assert.deepEqual(lspCodeLensResponse.result, [
  {
    range: {
      start: nodeSelector.range.start,
      end: nodeSelector.range.start,
    },
    command: {
      title: "1 reference",
      command: "editor.action.showReferences",
      arguments: [
        styleUri,
        nodeSelector.range.start,
        [
          {
            uri: sourceUri,
            range: sourceSelectorRange,
          },
        ],
      ],
    },
  },
  {
    range: {
      start: nodeThemeSelector.range.start,
      end: nodeThemeSelector.range.start,
    },
    command: {
      title: "1 reference",
      command: "editor.action.showReferences",
      arguments: [
        styleUri,
        nodeThemeSelector.range.start,
        [
          {
            uri: sourceUri,
            range: sourceBracketSelectorRange,
          },
        ],
      ],
    },
  },
  {
    range: {
      start: nodeAlertSelector.range.start,
      end: nodeAlertSelector.range.start,
    },
    command: {
      title: "1 reference",
      command: "editor.action.showReferences",
      arguments: [
        styleUri,
        nodeAlertSelector.range.start,
        [
          {
            uri: sourceUri,
            range: sourceUtilitySelectorRange,
          },
        ],
      ],
    },
  },
  {
    range: {
      start: nodeConditionalSelector.range.start,
      end: nodeConditionalSelector.range.start,
    },
    command: {
      title: "1 reference",
      command: "editor.action.showReferences",
      arguments: [
        styleUri,
        nodeConditionalSelector.range.start,
        [
          {
            uri: sourceUri,
            range: sourceConditionalSelectorRange,
          },
        ],
      ],
    },
  },
]);

const lspSourceHoverResponse = responses[13]!;
assert.equal(lspSourceHoverResponse.id, 14);
assert.deepEqual(lspSourceHoverResponse.result.range, sourceSelectorRange);
assert.equal(lspSourceHoverResponse.result.contents.kind, "markdown");
assert.match(lspSourceHoverResponse.result.contents.value, /\.root/);
assert.match(lspSourceHoverResponse.result.contents.value, /App\.module\.scss/);

const lspSourceDefinitionResponse = responses[14]!;
assert.equal(lspSourceDefinitionResponse.id, 15);
assert.deepEqual(lspSourceDefinitionResponse.result, [
  {
    uri: styleUri,
    range: nodeSelector.range,
  },
]);

const lspSourceReferencesResponse = responses[15]!;
assert.equal(lspSourceReferencesResponse.id, 16);
assert.deepEqual(lspSourceReferencesResponse.result, [
  {
    uri: styleUri,
    range: nodeSelector.range,
  },
  {
    uri: sourceUri,
    range: sourceSelectorRange,
  },
]);

const lspSourceCompletionResponse = responses[16]!;
assert.equal(lspSourceCompletionResponse.id, 17);
assert.equal(lspSourceCompletionResponse.result.isIncomplete, false);
assert.deepEqual(
  lspSourceCompletionResponse.result.items.map((item: { readonly label: string }) => item.label),
  ["alert", "conditional", "root", "theme"],
);

const lspSourcePrepareRenameResponse = responses[17]!;
assert.equal(lspSourcePrepareRenameResponse.id, 18);
assert.deepEqual(lspSourcePrepareRenameResponse.result, {
  range: sourceSelectorRange,
  placeholder: "root",
});

const lspSourceRenameResponse = responses[18]!;
assert.equal(lspSourceRenameResponse.id, 19);
assert.deepEqual(lspSourceRenameResponse.result, {
  changes: {
    [styleUri]: [
      {
        range: nodeSelector.range,
        newText: "panel",
      },
    ],
    [sourceUri]: [
      {
        range: sourceSelectorRange,
        newText: "panel",
      },
    ],
  },
});

const lspSourceCodeActionResponse = responses[19]!;
assert.equal(lspSourceCodeActionResponse.id, 20);
assert.deepEqual(lspSourceCodeActionResponse.result, [
  {
    title: "Add '.missing' to App.module.scss",
    kind: "quickfix",
    diagnostics: [expectedMissingSelectorDiagnostic],
    edit: {
      changes: {
        [styleUri]: [
          {
            range: expectedMissingSelectorDiagnostic.data.createSelector.range,
            newText: expectedMissingSelectorDiagnostic.data.createSelector.newText,
          },
        ],
      },
    },
    data: {
      source: "omenaQuerySourceSyntaxIndex",
      diagnosticIndex: 0,
    },
  },
]);

const lspStyleSelectorRenameResponse = responses[20]!;
assert.equal(lspStyleSelectorRenameResponse.id, 21);
assert.deepEqual(lspStyleSelectorRenameResponse.result, {
  changes: {
    [styleUri]: [
      {
        range: nodeSelector.range,
        newText: "panel",
      },
    ],
    [sourceUri]: [
      {
        range: sourceSelectorRange,
        newText: "panel",
      },
    ],
  },
});

const lspSourceNoopCompletionResponse = responses[21]!;
assert.equal(lspSourceNoopCompletionResponse.id, 22);
assert.equal(lspSourceNoopCompletionResponse.result, null);

const lspSourceBracketHoverResponse = responses[22]!;
assert.equal(lspSourceBracketHoverResponse.id, 23);
assert.deepEqual(lspSourceBracketHoverResponse.result.range, sourceBracketSelectorRange);
assert.equal(lspSourceBracketHoverResponse.result.contents.kind, "markdown");
assert.match(lspSourceBracketHoverResponse.result.contents.value, /\.theme/);
assert.match(lspSourceBracketHoverResponse.result.contents.value, /App\.module\.scss/);

const lspSourceBracketDefinitionResponse = responses[23]!;
assert.equal(lspSourceBracketDefinitionResponse.id, 24);
assert.deepEqual(lspSourceBracketDefinitionResponse.result, [
  {
    uri: styleUri,
    range: nodeThemeSelector.range,
  },
]);

const lspSourceBracketReferencesResponse = responses[24]!;
assert.equal(lspSourceBracketReferencesResponse.id, 25);
assert.deepEqual(lspSourceBracketReferencesResponse.result, [
  {
    uri: styleUri,
    range: nodeThemeSelector.range,
  },
  {
    uri: sourceUri,
    range: sourceBracketSelectorRange,
  },
]);

const lspSourceBracketCompletionResponse = responses[25]!;
assert.equal(lspSourceBracketCompletionResponse.id, 26);
assert.equal(lspSourceBracketCompletionResponse.result.isIncomplete, false);
assert.deepEqual(
  lspSourceBracketCompletionResponse.result.items.map(
    (item: { readonly label: string }) => item.label,
  ),
  ["alert", "conditional", "root", "theme"],
);

const lspSourceBracketRenameResponse = responses[26]!;
assert.equal(lspSourceBracketRenameResponse.id, 27);
assert.deepEqual(lspSourceBracketRenameResponse.result, {
  changes: {
    [styleUri]: [
      {
        range: nodeThemeSelector.range,
        newText: "surface",
      },
    ],
    [sourceUri]: [
      {
        range: sourceBracketSelectorRange,
        newText: "surface",
      },
    ],
  },
});

const lspSourceUtilityHoverResponse = responses[27]!;
assert.equal(lspSourceUtilityHoverResponse.id, 28);
assert.deepEqual(lspSourceUtilityHoverResponse.result.range, sourceUtilitySelectorRange);
assert.equal(lspSourceUtilityHoverResponse.result.contents.kind, "markdown");
assert.match(lspSourceUtilityHoverResponse.result.contents.value, /\.alert/);
assert.match(lspSourceUtilityHoverResponse.result.contents.value, /App\.module\.scss/);

const lspSourceUtilityDefinitionResponse = responses[28]!;
assert.equal(lspSourceUtilityDefinitionResponse.id, 29);
assert.deepEqual(lspSourceUtilityDefinitionResponse.result, [
  {
    uri: styleUri,
    range: nodeAlertSelector.range,
  },
]);

const lspSourceUtilityReferencesResponse = responses[29]!;
assert.equal(lspSourceUtilityReferencesResponse.id, 30);
assert.deepEqual(lspSourceUtilityReferencesResponse.result, [
  {
    uri: styleUri,
    range: nodeAlertSelector.range,
  },
  {
    uri: sourceUri,
    range: sourceUtilitySelectorRange,
  },
]);

const lspSourceUtilityCompletionResponse = responses[30]!;
assert.equal(lspSourceUtilityCompletionResponse.id, 31);
assert.equal(lspSourceUtilityCompletionResponse.result.isIncomplete, false);
assert.deepEqual(
  lspSourceUtilityCompletionResponse.result.items.map(
    (item: { readonly label: string }) => item.label,
  ),
  ["alert", "card", "conditional", "ghost", "root", "theme"],
);

const lspSourceUtilityRenameResponse = responses[31]!;
assert.equal(lspSourceUtilityRenameResponse.id, 32);
assert.deepEqual(lspSourceUtilityRenameResponse.result, {
  changes: {
    [styleUri]: [
      {
        range: nodeAlertSelector.range,
        newText: "notice",
      },
    ],
    [sourceUri]: [
      {
        range: sourceUtilitySelectorRange,
        newText: "notice",
      },
    ],
  },
});

const lspSourceConditionalHoverResponse = responses[32]!;
assert.equal(lspSourceConditionalHoverResponse.id, 33);
assert.deepEqual(lspSourceConditionalHoverResponse.result.range, sourceConditionalSelectorRange);
assert.equal(lspSourceConditionalHoverResponse.result.contents.kind, "markdown");
assert.match(lspSourceConditionalHoverResponse.result.contents.value, /\.conditional/);
assert.match(lspSourceConditionalHoverResponse.result.contents.value, /App\.module\.scss/);

const lspSourceConditionalDefinitionResponse = responses[33]!;
assert.equal(lspSourceConditionalDefinitionResponse.id, 34);
assert.deepEqual(lspSourceConditionalDefinitionResponse.result, [
  {
    uri: styleUri,
    range: nodeConditionalSelector.range,
  },
]);

const lspSourceConditionalReferencesResponse = responses[34]!;
assert.equal(lspSourceConditionalReferencesResponse.id, 35);
assert.deepEqual(lspSourceConditionalReferencesResponse.result, [
  {
    uri: styleUri,
    range: nodeConditionalSelector.range,
  },
  {
    uri: sourceUri,
    range: sourceConditionalSelectorRange,
  },
]);

const lspSourceConditionalCompletionResponse = responses[35]!;
assert.equal(lspSourceConditionalCompletionResponse.id, 36);
assert.equal(lspSourceConditionalCompletionResponse.result.isIncomplete, false);
assert.deepEqual(
  lspSourceConditionalCompletionResponse.result.items.map(
    (item: { readonly label: string }) => item.label,
  ),
  ["alert", "card", "conditional", "ghost", "root", "theme"],
);

const lspSourceConditionalRenameResponse = responses[36]!;
assert.equal(lspSourceConditionalRenameResponse.id, 37);
assert.deepEqual(lspSourceConditionalRenameResponse.result, {
  changes: {
    [styleUri]: [
      {
        range: nodeConditionalSelector.range,
        newText: "stateful",
      },
    ],
    [sourceUri]: [
      {
        range: sourceConditionalSelectorRange,
        newText: "stateful",
      },
    ],
  },
});

const crossFileSassCompletionLabels = assertCrossFileSassCompletionParity(invocation);

process.stdout.write(
  [
    "validated omena-lsp-server style provider parity:",
    `command=${invocation.command}`,
    `candidate=${styleHoverResponse.result.candidates[0].name}`,
    `customPropertyReference=${customPropertyReferenceResponse.result.candidates[0].name}`,
    `customPropertyDeclaration=${customPropertyDeclarationResponse.result.candidates[0].name}`,
    `lspHover=${lspHoverResponse.result.contents.kind}`,
    `lspDefinitionTargets=${lspDefinitionResponse.result.length}`,
    `lspReferences=${lspReferencesResponse.result.length}`,
    `lspCompletionItems=${lspCompletionResponse.result.items.length}`,
    `diagnostics=${styleDiagnosticsResponse.result.length}`,
    `diagnosticNotifications=${diagnosticNotifications.length}`,
    `codeActions=${lspCodeActionResponse.result.length}`,
    `prepareRename=${lspPrepareRenameResponse.result.placeholder}`,
    `renameEdits=${lspRenameResponse.result.changes[styleUri].length}`,
    `codeLens=${lspCodeLensResponse.result.length}`,
    `sourceHover=${lspSourceHoverResponse.result.contents.kind}`,
    `sourceDefinitionTargets=${lspSourceDefinitionResponse.result.length}`,
    `sourceReferences=${lspSourceReferencesResponse.result.length}`,
    `sourceCompletionItems=${lspSourceCompletionResponse.result.items.length}`,
    `sourceRenameEdits=${
      lspSourceRenameResponse.result.changes[styleUri].length +
      lspSourceRenameResponse.result.changes[sourceUri].length
    }`,
    `sourceCodeActions=${lspSourceCodeActionResponse.result.length}`,
    `styleSelectorRenameEdits=${
      lspStyleSelectorRenameResponse.result.changes[styleUri].length +
      lspStyleSelectorRenameResponse.result.changes[sourceUri].length
    }`,
    `sourceNoopCompletion=${lspSourceNoopCompletionResponse.result}`,
    `sourceBracketHover=${lspSourceBracketHoverResponse.result.contents.kind}`,
    `sourceBracketDefinitionTargets=${lspSourceBracketDefinitionResponse.result.length}`,
    `sourceBracketReferences=${lspSourceBracketReferencesResponse.result.length}`,
    `sourceBracketCompletionItems=${lspSourceBracketCompletionResponse.result.items.length}`,
    `sourceBracketRenameEdits=${
      lspSourceBracketRenameResponse.result.changes[styleUri].length +
      lspSourceBracketRenameResponse.result.changes[sourceUri].length
    }`,
    `sourceUtilityHover=${lspSourceUtilityHoverResponse.result.contents.kind}`,
    `sourceUtilityDefinitionTargets=${lspSourceUtilityDefinitionResponse.result.length}`,
    `sourceUtilityReferences=${lspSourceUtilityReferencesResponse.result.length}`,
    `sourceUtilityCompletionItems=${lspSourceUtilityCompletionResponse.result.items.length}`,
    `sourceUtilityRenameEdits=${
      lspSourceUtilityRenameResponse.result.changes[styleUri].length +
      lspSourceUtilityRenameResponse.result.changes[sourceUri].length
    }`,
    `sourceConditionalHover=${lspSourceConditionalHoverResponse.result.contents.kind}`,
    `sourceConditionalDefinitionTargets=${lspSourceConditionalDefinitionResponse.result.length}`,
    `sourceConditionalReferences=${lspSourceConditionalReferencesResponse.result.length}`,
    `sourceConditionalCompletionItems=${lspSourceConditionalCompletionResponse.result.items.length}`,
    `sourceConditionalRenameEdits=${
      lspSourceConditionalRenameResponse.result.changes[styleUri].length +
      lspSourceConditionalRenameResponse.result.changes[sourceUri].length
    }`,
    `line=${styleHoverResponse.result.candidates[0].range.start.line}`,
    `character=${styleHoverResponse.result.candidates[0].range.start.character}`,
    `nodeRangeParity=${JSON.stringify(styleHoverResponse.result.candidates[0].range)}`,
    `crossFileSassCompletion=${crossFileSassCompletionLabels.join(",")}`,
  ].join(" "),
);
process.stdout.write("\n");

function frame(value: unknown): string {
  const body = JSON.stringify(value);
  return `Content-Length: ${Buffer.byteLength(body, "utf8")}\r\n\r\n${body}`;
}

function assertCrossFileSassCompletionParity(invocation: {
  readonly command: string;
  readonly args: readonly string[];
}): string[] {
  const sassWorkspaceUri = "file:///tmp/omena-lsp-sass-completion-parity";
  const sassAppUri = `${sassWorkspaceUri}/src/App.module.scss`;
  const sassTokensUri = `${sassWorkspaceUri}/src/_tokens.scss`;
  const sassAppText = '@use "./tokens" as t;\n.button { border-radius: t.$ }';
  const result = spawnSync(invocation.command, [...invocation.args], {
    cwd: process.cwd(),
    input: [
      {
        jsonrpc: "2.0",
        id: 1,
        method: "initialize",
        params: {
          workspaceFolders: [{ uri: sassWorkspaceUri, name: "sass-completion-parity" }],
        },
      },
      {
        jsonrpc: "2.0",
        method: "textDocument/didOpen",
        params: {
          textDocument: {
            uri: sassAppUri,
            languageId: "scss",
            version: 1,
            text: sassAppText,
          },
        },
      },
      {
        jsonrpc: "2.0",
        method: "textDocument/didOpen",
        params: {
          textDocument: {
            uri: sassTokensUri,
            languageId: "scss",
            version: 1,
            text: "$radius-small: 2px;",
          },
        },
      },
      {
        jsonrpc: "2.0",
        id: 2,
        method: "textDocument/completion",
        params: {
          textDocument: { uri: sassAppUri },
          position: { line: 1, character: 28 },
        },
      },
      {
        jsonrpc: "2.0",
        id: 3,
        method: "shutdown",
      },
      {
        jsonrpc: "2.0",
        method: "exit",
      },
    ]
      .map(frame)
      .join(""),
    encoding: "utf8",
    stdio: ["pipe", "pipe", "pipe"],
  });
  assert.equal(
    result.status,
    0,
    [
      "omena-lsp-server cross-file Sass completion parity failed",
      result.error ? `error=${result.error.message}` : null,
      result.stderr.trim() ? `stderr=${result.stderr.trim()}` : null,
    ]
      .filter(Boolean)
      .join("\n"),
  );
  const responses = readFrames(result.stdout)
    .filter((message) => "id" in message)
    .sort((a, b) => Number(a.id) - Number(b.id));
  const completionResponse = responses.find((message) => message.id === 2);
  assert.ok(completionResponse, "cross-file Sass completion response should be present");
  const labels = completionResponse.result.items.map(
    (item: { readonly label: string }) => item.label,
  );
  assert.deepEqual(labels, ["t.$radius-small"]);
  return labels;
}

function documentEndRange(text: string): {
  readonly start: { readonly line: number; readonly character: number };
  readonly end: { readonly line: number; readonly character: number };
} {
  const lines = text.split("\n");
  const position = {
    line: lines.length - 1,
    character: lines[lines.length - 1]!.length,
  };
  return {
    start: position,
    end: position,
  };
}

function unusedSelectorDiagnostic(selector: { readonly name: string; readonly range: unknown }): {
  readonly range: unknown;
  readonly severity: 4;
  readonly source: "omena-css";
  readonly code: "unusedSelector";
  readonly message: string;
  readonly data: {
    readonly querySeverity: "hint";
    readonly provenance: readonly [
      "omena-parser.selector-facts",
      "omena-query.source-selector-usage",
      "omena-query-checker-orchestrator.product-diagnostic-gate",
      "omena-checker.rule-registry",
    ];
    readonly polynomialProvenance: ReturnType<typeof polynomialProvenanceFor>;
  };
  readonly tags: readonly [1];
} {
  return {
    range: selector.range,
    severity: 4,
    source: "omena-css",
    code: "unusedSelector",
    message: `Selector '.${selector.name}' is declared but never used.`,
    data: {
      querySeverity: "hint",
      provenance: [
        "omena-parser.selector-facts",
        "omena-query.source-selector-usage",
        "omena-query-checker-orchestrator.product-diagnostic-gate",
        "omena-checker.rule-registry",
      ],
      polynomialProvenance: polynomialProvenanceFor([
        "omena-parser.selector-facts",
        "omena-query.source-selector-usage",
        "omena-query-checker-orchestrator.product-diagnostic-gate",
        "omena-checker.rule-registry",
      ]),
    },
    tags: [1],
  };
}

function diagnosticWithTier<T extends { readonly data: Record<string, unknown> }>(
  diagnostic: T,
  pipelineTier: "baseline" | "optimizing",
  pipelineTierEvidence:
    | "fastFactsV0"
    | "sourceSyntaxIndexV0"
    | "analyzedGraphV0"
    | "workspaceSourceDiagnosticsV0",
): T & {
  readonly data: T["data"] & {
    readonly pipelineTier: typeof pipelineTier;
    readonly pipelineTierEvidence: typeof pipelineTierEvidence;
    readonly polynomialProvenance?: ReturnType<typeof polynomialProvenanceFor>;
  };
} {
  const data = {
    ...diagnostic.data,
    pipelineTier,
    pipelineTierEvidence,
  } as T["data"] & {
    readonly pipelineTier: typeof pipelineTier;
    readonly pipelineTierEvidence: typeof pipelineTierEvidence;
    polynomialProvenance?: ReturnType<typeof polynomialProvenanceFor>;
  };
  if (
    pipelineTierEvidence !== "sourceSyntaxIndexV0" &&
    pipelineTierEvidence !== "workspaceSourceDiagnosticsV0"
  ) {
    data.polynomialProvenance = polynomialProvenanceFor(diagnostic.data.provenance);
  }
  return {
    ...diagnostic,
    data,
  };
}

function diagnosticWithWorkspaceSnapshotId<T extends { readonly data: Record<string, unknown> }>(
  diagnostic: T,
  value: number,
): T & {
  readonly data: T["data"] & {
    readonly snapshotId: { readonly value: number };
  };
} {
  return {
    ...diagnostic,
    data: {
      ...diagnostic.data,
      snapshotId: { value },
    },
  };
}

function sourceDiagnosticPrecision(
  valueDomain: "classValueResolution" | "unknown",
  flowSensitivity:
    | "sourceSyntaxIndex"
    | "sourceSelectorReference"
    | "typeOracleProviderUnavailable",
  contextSensitivity: "perSourceReference" | "perTypeFactTarget" = "perSourceReference",
): {
  readonly product: "omena-query.analysis-precision";
  readonly valueDomain: "classValueResolution" | "unknown";
  readonly flowSensitivity:
    | "sourceSyntaxIndex"
    | "sourceSelectorReference"
    | "typeOracleProviderUnavailable";
  readonly contextSensitivity: "perSourceReference" | "perTypeFactTarget";
  readonly revisionAxis: "OmenaQuerySourceDiagnosticsForFileV0.input";
} {
  return {
    product: "omena-query.analysis-precision",
    valueDomain,
    flowSensitivity,
    contextSensitivity,
    revisionAxis: "OmenaQuerySourceDiagnosticsForFileV0.input",
  };
}

function polynomialProvenanceFor(provenance: unknown): {
  readonly schemaVersion: "0";
  readonly product: "omena-abstract-value.polynomial-provenance";
  readonly layerMarker: "qtt-graded";
  readonly featureGate: "qtt-provenance-polynomial-v0";
  readonly theoremClaimed: false;
  readonly claimLevel: "fixtureWitnessPolynomialProjection";
  readonly polynomialKind: "naturalCountPolynomialOverLabels";
  readonly rootOperator: "sum";
  readonly selectedLadder: "diagnosticDefaultThreeTier";
  readonly availableLadderTiers: readonly [
    "linearLabels",
    "naturalCountPolynomial",
    "homomorphicProjections",
  ];
  readonly variables: readonly { readonly variable: string; readonly label: string }[];
  readonly terms: readonly { readonly coefficient: 1; readonly variables: readonly string[] }[];
  readonly projections: readonly {
    readonly projectionKind: "why" | "whyNot" | "confidence" | "tropical";
    readonly semiringIdentifier: "lin01" | "naturalCount" | "tropical";
    readonly value: string;
  }[];
} {
  assert.ok(Array.isArray(provenance), "diagnostic provenance must be an array");
  assert.ok(
    provenance.every((entry) => typeof entry === "string"),
    "diagnostic provenance must contain only strings",
  );
  const labels = provenance as string[];
  return {
    schemaVersion: "0",
    product: "omena-abstract-value.polynomial-provenance",
    layerMarker: "qtt-graded",
    featureGate: "qtt-provenance-polynomial-v0",
    theoremClaimed: false,
    claimLevel: "fixtureWitnessPolynomialProjection",
    polynomialKind: "naturalCountPolynomialOverLabels",
    rootOperator: "sum",
    selectedLadder: "diagnosticDefaultThreeTier",
    availableLadderTiers: ["linearLabels", "naturalCountPolynomial", "homomorphicProjections"],
    variables: labels.map((label, index) => ({
      variable: `x${index}`,
      label,
    })),
    terms: labels.map((_, index) => ({
      coefficient: 1,
      variables: [`x${index}`],
    })),
    projections: [
      {
        projectionKind: "why",
        semiringIdentifier: "lin01",
        value: labels.join(" -> "),
      },
      {
        projectionKind: "whyNot",
        semiringIdentifier: "lin01",
        value: "noUnsupportedTermsInFixture",
      },
      {
        projectionKind: "confidence",
        semiringIdentifier: "naturalCount",
        value: `${labels.length}/${labels.length}`,
      },
      {
        projectionKind: "tropical",
        semiringIdentifier: "tropical",
        value: "1",
      },
    ],
  };
}

function sortDiagnosticNotifications<T>(notifications: readonly T[]): T[] {
  return notifications.toSorted((left, right) =>
    diagnosticNotificationSignature(left).localeCompare(diagnosticNotificationSignature(right)),
  );
}

function diagnosticNotificationSignature(notification: unknown): string {
  const params = isRecord(notification) ? notification.params : undefined;
  const uri = isRecord(params) && typeof params.uri === "string" ? params.uri : "";
  const diagnostics =
    isRecord(params) && Array.isArray(params.diagnostics) ? params.diagnostics : [];
  const codes = diagnostics
    .map((diagnostic) => {
      if (!isRecord(diagnostic)) return "";
      const code = typeof diagnostic.code === "string" ? diagnostic.code : "";
      const message = typeof diagnostic.message === "string" ? diagnostic.message : "";
      return `${code}:${message}`;
    })
    .join("|");
  return `${uri}::${diagnostics.length}::${codes}`;
}

function assertSingleCandidate(
  response: any,
  queryPosition: { readonly line: number; readonly character: number },
  expectedCandidate: {
    readonly kind: string;
    readonly name: string;
    readonly range: unknown;
    readonly source: string;
  },
): void {
  assert.equal(response.result.product, "omena-lsp-server.style-hover-candidates");
  assert.equal(response.result.documentUri, styleUri);
  assert.equal(response.result.workspaceFolderUri, workspaceUri);
  assert.equal(response.result.language, "scss");
  assert.equal(response.result.candidateCount, 1);
  assert.deepEqual(response.result.queryPosition, queryPosition);
  assert.deepEqual(response.result.candidates, [expectedCandidate]);
}

function readFrames(stdout: string): any[] {
  const frames: any[] = [];
  let offset = 0;

  while (offset < stdout.length) {
    const headerEnd = stdout.indexOf("\r\n\r\n", offset);
    if (headerEnd < 0) break;
    const header = stdout.slice(offset, headerEnd);
    const match = /^Content-Length:\s*(\d+)$/imu.exec(header);
    assert.ok(match, `missing Content-Length in response header: ${header}`);
    const length = Number(match[1]);
    const bodyStart = headerEnd + 4;
    const bodyEnd = bodyStart + length;
    assert.ok(bodyEnd <= stdout.length, "incomplete response body");
    frames.push(JSON.parse(stdout.slice(bodyStart, bodyEnd)));
    offset = bodyEnd;
  }

  return frames;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
