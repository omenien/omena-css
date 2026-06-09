import {
  CodeActionKind,
  type CodeAction,
  type CodeActionParams,
  type CreateFile,
  type WorkspaceEdit,
} from "vscode-languageserver/node";
import {
  planCodeActions,
  type CodeActionPlan,
} from "../../../engine-host-node/src/code-action-query";
import { wrapHandler } from "./_wrap-handler";
import type { ProviderDeps } from "./provider-deps";

/**
 * Handle `textDocument/codeAction` by mapping host-side recovery plans
 * into LSP quick fixes.
 */
export const handleCodeAction = wrapHandler<
  CodeActionParams,
  [documentContent?: string],
  CodeAction[] | null
>(
  "codeAction",
  (params, deps: ProviderDeps, documentContent?: string) => {
    const plans = planCodeActions(
      {
        documentUri: params.textDocument.uri,
        ...(documentContent !== undefined ? { documentContent } : {}),
        range: params.range,
        // LSP 3.18 widened Diagnostic.message to string | MarkupContent; the
        // host-side plan input is protocol-agnostic and takes plain text.
        diagnostics: params.context.diagnostics.map((diagnostic) => ({
          ...diagnostic,
          message:
            typeof diagnostic.message === "string" ? diagnostic.message : diagnostic.message.value,
        })),
      },
      deps,
    );
    if (plans.length === 0) return null;
    return plans.map((plan) => toCodeAction(plan, params.context.diagnostics));
  },
  null,
);

function toCodeAction(
  plan: CodeActionPlan,
  diagnostics: CodeActionParams["context"]["diagnostics"],
): CodeAction {
  return {
    title: plan.title,
    kind: toCodeActionKind(plan),
    ...(plan.diagnosticIndex !== undefined
      ? { diagnostics: [diagnostics[plan.diagnosticIndex]!] }
      : {}),
    ...(plan.isPreferred ? { isPreferred: true } : {}),
    edit: toWorkspaceEdit(plan),
  };
}

function toCodeActionKind(plan: CodeActionPlan): CodeActionKind {
  if (plan.actionKind === "refactor.extract") return CodeActionKind.RefactorExtract;
  if (plan.actionKind === "refactor.inline") return CodeActionKind.RefactorInline;
  return CodeActionKind.QuickFix;
}

function toWorkspaceEdit(plan: CodeActionPlan): WorkspaceEdit {
  if (plan.kind === "createFile") {
    const createFile: CreateFile = {
      kind: "create",
      uri: plan.uri,
      options: {
        overwrite: false,
        ignoreIfExists: true,
      },
    };
    return { documentChanges: [createFile] };
  }

  if (plan.kind === "workspaceEdit") {
    const changes: WorkspaceEdit["changes"] = {};
    for (const edit of plan.edits) {
      const edits = changes[edit.uri] ?? [];
      changes[edit.uri] = [...edits, { range: edit.range, newText: edit.newText }];
    }
    return { changes };
  }

  return {
    changes: {
      [plan.uri]: [
        {
          range: plan.range,
          newText: plan.newText,
        },
      ],
    },
  };
}
