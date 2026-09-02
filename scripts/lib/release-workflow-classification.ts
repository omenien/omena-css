import { createHash } from "node:crypto";

import {
  collectParsedWorkflowJobs,
  type ParsedWorkflowJobView,
} from "../../packages/check-orchestrator/src/manifest/workflows.ts";

export interface ReleaseWorkflowJobFact extends ParsedWorkflowJobView {
  readonly key: string;
  readonly semanticDigest: string;
}

export function releaseWorkflowJobFacts(rootDir: string): readonly ReleaseWorkflowJobFact[] {
  return collectParsedWorkflowJobs(rootDir).map((view) => ({
    workflowFile: view.workflowFile,
    jobName: view.jobName,
    events: view.events,
    workflowPermissions: view.workflowPermissions,
    workflowEnv: view.workflowEnv,
    workflowCallInputs: view.workflowCallInputs,
    job: view.job,
    key: `${view.workflowFile}:${view.jobName}`,
    semanticDigest: releaseWorkflowJobDigest(view),
  }));
}

export function releaseWorkflowJobDigest(view: ParsedWorkflowJobView): string {
  const semanticInput = normalizeSemanticValue({
    workflow: {
      permissions: view.workflowPermissions,
      env: view.workflowEnv,
      workflowCallInputs: view.workflowCallInputs,
    },
    job: view.job,
  });
  return createHash("sha256").update(JSON.stringify(semanticInput)).digest("hex");
}

function normalizeSemanticValue(value: unknown, parentKey = ""): unknown {
  if (Array.isArray(value)) {
    return value.map((item) => normalizeSemanticValue(item));
  }
  if (value !== null && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value as Record<string, unknown>)
        .toSorted(([left], [right]) => left.localeCompare(right))
        .map(([key, child]) => [key, normalizeSemanticValue(child, key)]),
    );
  }
  if (parentKey === "uses" && typeof value === "string" && value.includes("@")) {
    return value.replace(/@[^@]+$/u, "@<ref>");
  }
  return value;
}
