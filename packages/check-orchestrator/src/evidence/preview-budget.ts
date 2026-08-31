import type { CostLedger } from "../manifest/cost-ledger";
import type { CheckGate, CheckManifest } from "../manifest/types";

export const EVIDENCE_PREVIEW_BUDGET_MS = 60_000;
export const EVIDENCE_PREVIEW_EMPTY_BUDGET_MS = 5_000;

export interface EvidencePreviewEntryV0 {
  readonly gateId: string;
  readonly p95Ms: number | null;
}

export interface EvidencePreviewBudgetPlanV0 {
  readonly budgetMs: 60_000;
  readonly estimatedRunMs: number;
  readonly pricedSkippedMs: number;
  readonly unboundedSkippedCount: number;
  readonly ranPrefix: readonly EvidencePreviewEntryV0[];
  readonly skipped: readonly EvidencePreviewEntryV0[];
  readonly omittedWriteModeGateIds: readonly string[];
}

export function buildEvidencePreviewBudgetPlan(
  gateIds: readonly string[],
  manifest: CheckManifest,
  ledger: CostLedger,
): EvidencePreviewBudgetPlanV0 {
  const gateById = new Map(manifest.gates.map((gate) => [gate.id, gate]));
  const p95ByGate = new Map(ledger.gates.map((row) => [row.gateId, row.p95Ms]));
  const omittedWriteModeGateIds: string[] = [];
  const candidates: EvidencePreviewEntryV0[] = [];
  for (const gateId of [...new Set(gateIds)].toSorted()) {
    const gate = gateById.get(gateId);
    if (!gate) throw new Error(`evidence preview references unknown gate: ${gateId}`);
    if (!isEvidencePreviewCheckGate(gate)) {
      omittedWriteModeGateIds.push(gateId);
      continue;
    }
    candidates.push({ gateId, p95Ms: p95ByGate.get(gateId) ?? null });
  }
  candidates.sort((left, right) => {
    const priceOrder =
      (left.p95Ms ?? Number.POSITIVE_INFINITY) - (right.p95Ms ?? Number.POSITIVE_INFINITY);
    return priceOrder || compareText(left.gateId, right.gateId);
  });
  const ranPrefix: EvidencePreviewEntryV0[] = [];
  const skipped: EvidencePreviewEntryV0[] = [];
  let estimatedRunMs = 0;
  for (const entry of candidates) {
    if (entry.p95Ms !== null && estimatedRunMs + entry.p95Ms <= EVIDENCE_PREVIEW_BUDGET_MS) {
      ranPrefix.push(entry);
      estimatedRunMs += entry.p95Ms;
    } else {
      skipped.push(entry);
    }
  }
  if (estimatedRunMs > EVIDENCE_PREVIEW_BUDGET_MS) {
    throw new Error(
      `evidence preview estimated prefix ${estimatedRunMs}ms exceeds ${EVIDENCE_PREVIEW_BUDGET_MS}ms`,
    );
  }
  const pricedSkippedMs = skipped.reduce((sum, entry) => sum + (entry.p95Ms ?? 0), 0);
  const unboundedSkippedCount = skipped.filter((entry) => entry.p95Ms === null).length;
  return {
    budgetMs: EVIDENCE_PREVIEW_BUDGET_MS,
    estimatedRunMs,
    pricedSkippedMs,
    unboundedSkippedCount,
    ranPrefix,
    skipped,
    omittedWriteModeGateIds: omittedWriteModeGateIds.toSorted(),
  };
}

export function isEvidencePreviewCheckGate(gate: CheckGate): boolean {
  if (gate.scriptName.startsWith("update:")) return false;
  return !/(?:^|\s)--(?:write(?:-[a-z0-9-]+)?|update(?:-[a-z0-9-]+)?)(?=$|[\s=:])/u.test(
    gate.command,
  );
}

export async function executeEvidencePreviewBudget(
  plan: EvidencePreviewBudgetPlanV0,
  executeGate: (gateId: string) => Promise<number>,
): Promise<readonly { readonly gateId: string; readonly elapsedMs: number }[]> {
  const receipts: { gateId: string; elapsedMs: number }[] = [];
  for (const entry of plan.ranPrefix) {
    const startedAt = performance.now();
    const exitCode = await executeGate(entry.gateId);
    const elapsedMs = Math.round(performance.now() - startedAt);
    receipts.push({ gateId: entry.gateId, elapsedMs });
    if (exitCode !== 0) {
      throw new Error(`evidence preview gate failed: ${entry.gateId} (exit ${exitCode})`);
    }
  }
  return receipts;
}

function compareText(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
}
