import type { LocationLink } from "vscode-languageserver/node";
import type { Range } from "@omena/shared";
import { findLangForPath } from "../../../engine-core-ts/src/core/scss/lang-registry";
import { pathToFileUrl } from "../../../engine-core-ts/src/core/util/text-utils";
import {
  resolveSourceExpressionDefinitionTargets,
  resolveSourceExpressionDefinitionTargetsAsync,
} from "../../../engine-host-node/src/source-definition-query";
import type { RustSelectedQueryBackendJsonRunnerAsync } from "../../../engine-host-node/src/selected-query-backend";
import {
  resolveStyleDefinitionTargets,
  resolveStyleDefinitionTargetsAsync,
} from "../../../engine-host-node/src/style-definition-query";
import { toLspRange } from "./lsp-adapters";
import { wrapHandler } from "./_wrap-handler";
import { withSourceExpressionAtCursor, type SourceExpressionContext } from "./cursor-dispatch";
import { getRustSelectedQueryBackendJsonRunnerAsync } from "./selected-query-runner";
import type { CursorParams, ProviderDeps } from "./provider-deps";

/**
 * Handle `textDocument/definition` for any class expression under the
 * cursor.
 *
 * Dispatches through the unified expression cursor stage and
 * routes source-side query evaluation through the Node host boundary.
 * Each target becomes a `LocationLink`, which lets VS Code
 * offer multi-match selection when a ref resolves to more than one
 * selector.
 *
 * Each selector becomes a `LocationLink`:
 *   - `originSelectionRange` — the class token in source (drives
 *     the underline on the click target)
 *   - `targetUri`            — `file://` URL of the SCSS module
 *   - `targetRange`          — full `{ ... }` rule block (peek preview)
 *   - `targetSelectionRange` — class token range (caret placement)
 *
 * An empty match returns `null` rather than `[]`, so other
 * providers can still attempt. The `wrapHandler` boundary ensures
 * a single handler bug never crashes the server.
 */
export const handleDefinition = wrapHandler<CursorParams, [], LocationLink[] | null>(
  "definition",
  (params, deps) => {
    const rustRunner = getRustSelectedQueryBackendJsonRunnerAsync(deps);
    if (findLangForPath(params.filePath)) {
      if (rustRunner) return buildStyleDefinitionAsync(params, deps, rustRunner);
      return buildStyleDefinition(params, deps);
    }
    if (rustRunner) {
      return withSourceExpressionAtCursor(params, deps, (ctx) =>
        buildLinksAsync(ctx, params, deps, rustRunner),
      );
    }
    return withSourceExpressionAtCursor(params, deps, (ctx) => buildLinks(ctx, params, deps));
  },
  null,
);

function buildLinks(
  ctx: SourceExpressionContext,
  params: CursorParams,
  deps: ProviderDeps,
): LocationLink[] | null {
  const targets = resolveSourceExpressionDefinitionTargets(ctx, params, deps);
  if (targets.length === 0) return null;
  return targets.map<LocationLink>((target) =>
    toLocationLinkFromTarget(
      target.originRange,
      pathToFileUrl(target.targetFilePath),
      target.targetRange,
      target.targetSelectionRange,
    ),
  );
}

async function buildLinksAsync(
  ctx: SourceExpressionContext,
  params: CursorParams,
  deps: ProviderDeps,
  rustRunner: RustSelectedQueryBackendJsonRunnerAsync,
): Promise<LocationLink[] | null> {
  const targets = await resolveSourceExpressionDefinitionTargetsAsync(ctx, params, deps, {
    runRustSelectedQueryBackendJsonAsync: rustRunner,
  });
  if (targets.length === 0) return null;
  return targets.map<LocationLink>((target) =>
    toLocationLinkFromTarget(
      target.originRange,
      pathToFileUrl(target.targetFilePath),
      target.targetRange,
      target.targetSelectionRange,
    ),
  );
}

function toLocationLinkFromTarget(
  originRange: Range,
  targetUri: string,
  targetRange: Range,
  targetSelectionRange: Range,
): LocationLink {
  return {
    originSelectionRange: toLspRange(originRange),
    targetUri,
    targetRange: toLspRange(targetRange),
    targetSelectionRange: toLspRange(targetSelectionRange),
  };
}

function buildStyleDefinition(params: CursorParams, deps: ProviderDeps): LocationLink[] | null {
  const targets = resolveStyleDefinitionTargets(params, deps);
  if (targets.length === 0) return null;
  return targets.map((target) =>
    toLocationLinkFromTarget(
      target.originRange,
      pathToFileUrl(target.targetFilePath),
      target.targetRange,
      target.targetSelectionRange,
    ),
  );
}

async function buildStyleDefinitionAsync(
  params: CursorParams,
  deps: ProviderDeps,
  rustRunner: RustSelectedQueryBackendJsonRunnerAsync,
): Promise<LocationLink[] | null> {
  const targets = await resolveStyleDefinitionTargetsAsync(params, deps, {
    runRustSelectedQueryBackendJsonAsync: rustRunner,
  });
  if (targets.length === 0) return null;
  return targets.map((target) =>
    toLocationLinkFromTarget(
      target.originRange,
      pathToFileUrl(target.targetFilePath),
      target.targetRange,
      target.targetSelectionRange,
    ),
  );
}
