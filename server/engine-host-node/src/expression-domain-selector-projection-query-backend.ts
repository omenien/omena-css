import type { ProviderDeps } from "../../engine-core-ts/src/provider-deps";
import type { EdgeCertainty } from "../../engine-core-ts/src/core/semantic/certainty";
import {
  buildSelectedQueryBackendInput,
  isEngineShadowRunnerCancelledError,
  SELECTED_QUERY_RUNNER_COMMANDS,
  runRustSelectedQueryBackendJson,
  runRustSelectedQueryBackendJsonAsync,
  type RustSelectedQueryBackendJsonRunnerAsync,
  type SelectedQueryBackendDocument,
} from "./selected-query-backend";

export interface ExpressionDomainSelectorProjectionEntryV0 {
  readonly graphId: string;
  readonly filePath: string;
  readonly nodeId: string;
  readonly targetStylePaths: readonly string[];
  readonly valueKind: string;
  readonly selectorNames: readonly string[];
  readonly certainty: EdgeCertainty;
}

interface ExpressionDomainSelectorProjectionSummaryV0 {
  readonly product: "omena-query.expression-domain-selector-projection";
  readonly projections: readonly ExpressionDomainSelectorProjectionEntryV0[];
}

export function resolveRustExpressionDomainSelectorProjections(
  document: SelectedQueryBackendDocument,
  scssModulePath: string,
  deps: Pick<
    ProviderDeps,
    "analysisCache" | "styleDocumentForPath" | "typeResolver" | "workspaceRoot" | "settings"
  >,
): readonly ExpressionDomainSelectorProjectionEntryV0[] {
  const input = buildSelectedQueryBackendInput(document, scssModulePath, deps);
  try {
    const summary = runRustSelectedQueryBackendJson<ExpressionDomainSelectorProjectionSummaryV0>(
      SELECTED_QUERY_RUNNER_COMMANDS.expressionDomainSelectorProjection,
      input,
    );
    return summary.projections;
  } catch (err) {
    if (isEngineShadowRunnerCancelledError(err)) return [];
    throw err;
  }
}

export async function resolveRustExpressionDomainSelectorProjectionsAsync(
  document: SelectedQueryBackendDocument,
  scssModulePath: string,
  deps: Pick<
    ProviderDeps,
    "analysisCache" | "styleDocumentForPath" | "typeResolver" | "workspaceRoot" | "settings"
  >,
  runJson: RustSelectedQueryBackendJsonRunnerAsync = runRustSelectedQueryBackendJsonAsync,
): Promise<readonly ExpressionDomainSelectorProjectionEntryV0[]> {
  const input = buildSelectedQueryBackendInput(document, scssModulePath, deps);
  try {
    const summary = await runJson<ExpressionDomainSelectorProjectionSummaryV0>(
      SELECTED_QUERY_RUNNER_COMMANDS.expressionDomainSelectorProjection,
      input,
    );
    return summary.projections;
  } catch (err) {
    if (isEngineShadowRunnerCancelledError(err)) return [];
    throw err;
  }
}
