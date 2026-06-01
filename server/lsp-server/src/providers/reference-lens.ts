import type { CodeLens, CodeLensParams } from "vscode-languageserver/node";
import type { ShowReferencesArgs } from "@omena/shared";
import { findLangForPath } from "../../../engine-core-ts/src/core/scss/lang-registry";
import { fileUrlToPath } from "../../../engine-core-ts/src/core/util/text-utils";
import {
  resolveStyleReferenceLenses,
  resolveStyleReferenceLensesAsync,
} from "../../../engine-host-node/src/style-reference-lens-query";
import { wrapHandler } from "./_wrap-handler";
import { getRustSelectedQueryBackendJsonRunnerAsync } from "./selected-query-runner";

/**
 * Handle `textDocument/codeLens` on `.module.{scss,css}` files.
 *
 * For every canonical selector in the file, emit a CodeLens anchored
 * at the class token's start position. The command resolves the
 * reference list via the shared reference query at codeLens request
 * time (not at resolve time — `resolveProvider: false`), and
 * invokes VS Code's built-in `editor.action.showReferences`.
 *
 * This handler does not dispatch on a cursor position — it
 * iterates the style document only. The `wrapHandler`
 * boundary captures sync exceptions.
 */
export const handleCodeLens = wrapHandler<CodeLensParams, [], CodeLens[] | null>(
  "codeLens",
  (params, deps) => {
    const filePath = fileUrlToPath(params.textDocument.uri);
    if (!findLangForPath(filePath)) return null;

    const styleDocument = deps.styleDocumentForPath(filePath);
    if (!styleDocument) return null;

    const rustRunner = getRustSelectedQueryBackendJsonRunnerAsync(deps);
    if (rustRunner) {
      return resolveStyleReferenceLensesAsync(filePath, styleDocument, deps, {
        runRustSelectedQueryBackendJsonAsync: rustRunner,
      }).then((lenses) => {
        const codeLenses = lenses.map<CodeLens>((lens) => buildLens(params.textDocument.uri, lens));
        return codeLenses.length > 0 ? codeLenses : null;
      });
    }

    const lenses = resolveStyleReferenceLenses(filePath, styleDocument, deps).map<CodeLens>(
      (lens) => buildLens(params.textDocument.uri, lens),
    );
    return lenses.length > 0 ? lenses : null;
  },
  null,
);

function buildLens(
  uri: string,
  lens: ReturnType<typeof resolveStyleReferenceLenses>[number],
): CodeLens {
  // VS Code's built-in `editor.action.showReferences` command takes
  // (uri, position, locations) positionally, so the wire arguments
  // must be a 3-tuple. The `ShowReferencesArgs` contract (shared)
  // documents this shape for both the server and the client
  // middleware (see client/src/extension.ts).
  const args: ShowReferencesArgs = [uri, lens.position, lens.locations];
  return {
    range: {
      start: { line: lens.position.line, character: lens.position.character },
      end: { line: lens.position.line, character: lens.position.character },
    },
    command: {
      title: lens.title,
      command: "editor.action.showReferences",
      arguments: [...args],
    },
  };
}
