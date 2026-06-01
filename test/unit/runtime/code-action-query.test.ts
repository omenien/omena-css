import { describe, expect, it } from "vitest";
import {
  planCodeActions,
  type CodeActionDiagnosticInput,
} from "../../../server/engine-host-node/src/code-action-query";
import { SELECTED_QUERY_RUNNER_COMMANDS } from "../../../server/engine-host-node/src/selected-query-backend";

function diagnostic(suggestion: string | undefined, message = "foo"): CodeActionDiagnosticInput {
  const className = /Class '\.([^']+)'/.exec(message)?.[1] ?? "generated";
  return {
    range: {
      start: { line: 4, character: 15 },
      end: { line: 4, character: 24 },
    },
    message,
    data:
      suggestion === undefined
        ? undefined
        : {
            suggestion,
            createSelector: {
              uri: "file:///fake/src/Button.module.scss",
              range: {
                start: { line: 1, character: 0 },
                end: { line: 1, character: 0 },
              },
              newText: `\n\n.${className} {\n}\n`,
            },
          },
  };
}

describe("planCodeActions", () => {
  it("plans replacement and create-selector quick fixes from diagnostic data", () => {
    const result = planCodeActions(
      {
        documentUri: "file:///fake/src/Button.tsx",
        diagnostics: [
          diagnostic("indicator", "Class '.indicaror' not found. Did you mean 'indicator'?"),
        ],
      },
      { fileExists: () => true },
    );

    expect(result).toEqual([
      {
        kind: "textEdit",
        title: "Replace with 'indicator'",
        diagnosticIndex: 0,
        uri: "file:///fake/src/Button.tsx",
        range: {
          start: { line: 4, character: 15 },
          end: { line: 4, character: 24 },
        },
        newText: "indicator",
        isPreferred: true,
      },
      {
        kind: "textEdit",
        title: "Add '.indicaror' to Button.module.scss",
        diagnosticIndex: 0,
        uri: "file:///fake/src/Button.module.scss",
        range: {
          start: { line: 1, character: 0 },
          end: { line: 1, character: 0 },
        },
        newText: "\n\n.indicaror {\n}\n",
      },
    ]);
  });

  it("plans create-file actions from missing module diagnostics", () => {
    const result = planCodeActions(
      {
        documentUri: "file:///fake/src/Button.tsx",
        diagnostics: [
          {
            range: {
              start: { line: 0, character: 19 },
              end: { line: 0, character: 38 },
            },
            message: "Cannot resolve CSS Module './Button.module.scss'. The file does not exist.",
            data: {
              createModuleFile: {
                uri: "file:///fake/src/Button.module.scss",
              },
            },
          },
        ],
      },
      { fileExists: () => false },
    );

    expect(result).toEqual([
      {
        kind: "createFile",
        title: "Create Button.module.scss",
        diagnosticIndex: 0,
        uri: "file:///fake/src/Button.module.scss",
        isPreferred: true,
      },
    ]);
  });

  it("uses explicit creation labels from diagnostic data before parsing messages", () => {
    const range = {
      start: { line: 1, character: 0 },
      end: { line: 1, character: 0 },
    };
    const result = planCodeActions(
      {
        documentUri: "file:///fake/src/Button.module.scss",
        diagnostics: [
          {
            range,
            message: "unparseable selector diagnostic",
            data: {
              createSelector: {
                uri: "file:///fake/src/Button.module.scss",
                range,
                newText: "/* generated selector */",
                selectorName: "from-data",
              },
            },
          },
          {
            range,
            message: "unparseable value diagnostic",
            data: {
              createValue: {
                uri: "file:///fake/src/tokens.module.scss",
                range,
                newText: "/* generated value */",
                valueName: "accent",
              },
            },
          },
          {
            range,
            message: "unparseable keyframes diagnostic",
            data: {
              createKeyframes: {
                uri: "file:///fake/src/Button.module.scss",
                range,
                newText: "/* generated keyframes */",
                keyframesName: "fade-in",
              },
            },
          },
          {
            range,
            message: "unparseable custom property diagnostic",
            data: {
              createCustomProperty: {
                uri: "file:///fake/src/Button.module.scss",
                range,
                newText: "/* generated custom property */",
                propertyName: "--brand",
              },
            },
          },
          {
            range,
            message: "unparseable Sass symbol diagnostic",
            data: {
              createSassSymbol: {
                uri: "file:///fake/src/Button.module.scss",
                range,
                newText: "/* generated symbol */",
                symbolLabel: "@mixin raised",
              },
            },
          },
        ],
      },
      { fileExists: () => true },
    );

    expect(result.map((plan) => plan.title)).toEqual([
      "Add '.from-data' to Button.module.scss",
      "Add '@value accent' to tokens.module.scss",
      "Add '@keyframes fade-in' to Button.module.scss",
      "Add '--brand' to Button.module.scss",
      "Add '@mixin raised' to Button.module.scss",
    ]);
  });

  it("plans proactive sibling module actions for unstyled source files", () => {
    const result = planCodeActions(
      {
        documentUri: "file:///fake/src/Button.tsx",
        diagnostics: [],
      },
      { fileExists: () => false },
    );

    expect(result.map((plan) => plan.title)).toEqual([
      "Create Button.module.scss",
      "Create Button.module.css",
      "Create Button.module.less",
    ]);
  });

  it("routes style refactor actions through omena-query when rust selected-query is active", () => {
    const previousBackend = process.env.OMENA_SELECTED_QUERY_BACKEND;
    process.env.OMENA_SELECTED_QUERY_BACKEND = "rust-selected-query";
    const calls: unknown[] = [];
    try {
      const result = planCodeActions(
        {
          documentUri: "file:///fake/src/Button.module.scss",
          documentContent: ".button { color: #fff; }\n",
          range: {
            start: { line: 0, character: 17 },
            end: { line: 0, character: 21 },
          },
          diagnostics: [],
        },
        {
          fileExists: () => true,
          runRustSelectedQueryBackendJson: (command: string, input: unknown) => {
            calls.push({ command, input });
            return {
              product: "omena-query.code-actions",
              fileUri: "file:///fake/src/Button.module.scss",
              fileKind: "style",
              actionCount: 1,
              actions: [
                {
                  title: "Extract CSS custom property '--extracted-color'",
                  kind: "refactor.extract",
                  source: "omenaQueryStyleExtractCodeActions",
                  edits: [
                    {
                      uri: "file:///fake/src/Button.module.scss",
                      range: {
                        start: { line: 0, character: 0 },
                        end: { line: 0, character: 0 },
                      },
                      newText: ":root {\n  --extracted-color: #fff;\n}\n\n",
                    },
                  ],
                },
              ],
            };
          },
        },
      );

      expect(calls).toHaveLength(1);
      expect(calls[0]).toMatchObject({
        command: SELECTED_QUERY_RUNNER_COMMANDS.styleCodeActions,
        input: {
          styleUri: "file:///fake/src/Button.module.scss",
          styleSource: ".button { color: #fff; }\n",
        },
      });
      expect(result).toEqual([
        {
          kind: "workspaceEdit",
          actionKind: "refactor.extract",
          title: "Extract CSS custom property '--extracted-color'",
          edits: [
            {
              uri: "file:///fake/src/Button.module.scss",
              range: {
                start: { line: 0, character: 0 },
                end: { line: 0, character: 0 },
              },
              newText: ":root {\n  --extracted-color: #fff;\n}\n\n",
            },
          ],
        },
      ]);
    } finally {
      if (previousBackend === undefined) {
        delete process.env.OMENA_SELECTED_QUERY_BACKEND;
      } else {
        process.env.OMENA_SELECTED_QUERY_BACKEND = previousBackend;
      }
    }
  });
});
