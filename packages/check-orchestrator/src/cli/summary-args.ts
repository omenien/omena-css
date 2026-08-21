interface SummaryArgumentParent {
  readonly id: string;
  readonly kind: string;
}

interface SummaryArgumentTarget {
  readonly target: string;
  readonly args?: readonly string[];
}

export function resolveSummaryMemberArgs(
  parent: SummaryArgumentParent,
  target: SummaryArgumentTarget,
  extraArgs: readonly string[],
): readonly string[] {
  const targetArgs = target.args ?? [];
  return target.target === parent.id || parent.kind === "alias"
    ? [...targetArgs, ...extraArgs]
    : targetArgs;
}
