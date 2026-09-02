import { strict as assert } from "node:assert";
import { execFileSync } from "node:child_process";

import {
  RELEASE_REHEARSAL_ESCALATION_TITLE,
  RELEASE_REHEARSAL_JOB_NAME,
  RELEASE_REHEARSAL_PATH_STEPS,
  RELEASE_REHEARSAL_WORKFLOW,
} from "./lib/release-rehearsal-contract.ts";

interface StepReceipt {
  readonly name: string;
  readonly conclusion: string | null;
}

interface RunReceipt {
  readonly id: number;
  readonly createdAt: string;
  readonly completed: boolean;
  readonly defaultBranch: boolean;
  readonly conclusion: string | null;
  readonly steps: readonly StepReceipt[];
}

interface IssueReceipt {
  readonly number: number;
  readonly title: string;
  readonly createdAt: string;
}

interface RetirementInput {
  readonly runs: readonly RunReceipt[];
  readonly issues: readonly IssueReceipt[];
  readonly historyExists: boolean;
}

const fixture = process.argv.find((arg) => arg.startsWith("--fixture-"));
const now = Date.now();
const input = fixture ? fixtureInput(fixture, now) : liveInput();
const defaultBranchRuns = input.runs.filter((run) => run.defaultBranch);
const greenRuns = defaultBranchRuns.filter(laneGreen);
const newestGreen = greenRuns.toSorted((left, right) =>
  right.createdAt.localeCompare(left.createdAt),
)[0];
const greenAgeDays = newestGreen ? ageDays(newestGreen.createdAt, now) : null;
const completedHistory = defaultBranchRuns.filter((run) => run.completed);
const oldestHistoryAgeDays = completedHistory.length
  ? Math.max(...completedHistory.map((run) => ageDays(run.createdAt, now)))
  : null;

if (
  input.historyExists &&
  (greenAgeDays !== null ? greenAgeDays > 21 : (oldestHistoryAgeDays ?? 0) > 21)
) {
  throw new Error("NOTICE: no default-branch lane-green rehearsal receipt exists within 21 days");
}

const staleIssue = input.issues.find(
  (issue) =>
    issue.title === RELEASE_REHEARSAL_ESCALATION_TITLE && ageDays(issue.createdAt, now) > 21,
);
const issueNotice =
  newestGreen && staleIssue
    ? `NOTICE: lane is green but stale escalation issue #${staleIssue.number} must be closed: ${staleIssue.title}`
    : null;

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "release.rehearsal.retirement",
      rehearsalPathSteps: RELEASE_REHEARSAL_PATH_STEPS,
      defaultBranchRunCount: defaultBranchRuns.length,
      laneGreenRunCount: greenRuns.length,
      newestLaneGreenRunId: newestGreen?.id ?? null,
      newestLaneGreenAgeDays: greenAgeDays,
      oldestCompletedHistoryAgeDays: oldestHistoryAgeDays,
      coldStart: !input.historyExists,
      issueNotice,
    },
    null,
    2,
  )}\n`,
);

function laneGreen(run: RunReceipt): boolean {
  const conclusions = new Map(run.steps.map((step) => [step.name, step.conclusion]));
  return RELEASE_REHEARSAL_PATH_STEPS.every((name) => conclusions.get(name) === "success");
}

function liveInput(): RetirementInput {
  assert.equal(process.argv.includes("--live"), true, "normal retirement checks require --live");
  const repository = process.env.GITHUB_REPOSITORY ?? "omenien/omena-css";
  const currentRunId = Number.parseInt(process.env.GITHUB_RUN_ID ?? "0", 10);
  assert.ok(currentRunId > 0, "GITHUB_RUN_ID is required for live retirement history");
  const repo = ghJson<{ readonly default_branch: string }>(`repos/${repository}`);
  const history = ghJson<{
    readonly workflow_runs: readonly {
      readonly id: number;
      readonly head_branch: string | null;
      readonly status: string;
      readonly conclusion: string | null;
      readonly created_at: string;
    }[];
  }>(`repos/${repository}/actions/workflows/${RELEASE_REHEARSAL_WORKFLOW}/runs?per_page=50`);
  const selected = history.workflow_runs.filter(
    (run) => run.head_branch === repo.default_branch || run.id === currentRunId,
  );
  const runs = selected.map((run): RunReceipt => {
    const jobs = ghJson<{
      readonly jobs: readonly {
        readonly name: string;
        readonly steps?: readonly StepReceipt[];
      }[];
    }>(`repos/${repository}/actions/runs/${run.id}/jobs?filter=all&per_page=100`);
    const job = jobs.jobs.find((candidate) => candidate.name === RELEASE_REHEARSAL_JOB_NAME);
    assert(job, `release rehearsal job is absent from run ${run.id}`);
    return {
      id: run.id,
      createdAt: run.created_at,
      completed: run.status === "completed",
      defaultBranch: run.head_branch === repo.default_branch,
      conclusion: run.conclusion,
      steps: job.steps ?? [],
    };
  });
  const issues = ghJson<
    readonly { readonly number: number; readonly title: string; readonly created_at: string }[]
  >(`repos/${repository}/issues?state=open&per_page=100`).map((issue) => ({
    number: issue.number,
    title: issue.title,
    createdAt: issue.created_at,
  }));
  return {
    runs,
    issues,
    historyExists: runs.some((run) => run.id !== currentRunId && run.completed),
  };
}

function fixtureInput(name: string, timestamp: number): RetirementInput {
  const recent = new Date(timestamp - 2 * 86_400_000).toISOString();
  const stale = new Date(timestamp - 22 * 86_400_000).toISOString();
  const greenSteps = RELEASE_REHEARSAL_PATH_STEPS.map((step) => ({
    name: step,
    conclusion: "success",
  }));
  if (name === "--fixture-cold-start") return { runs: [], issues: [], historyExists: false };
  if (name === "--fixture-healing") {
    return {
      runs: [
        {
          id: 4,
          createdAt: recent,
          completed: true,
          defaultBranch: true,
          conclusion: "failure",
          steps: greenSteps,
        },
      ],
      issues: [],
      historyExists: true,
    };
  }
  if (name === "--fixture-signal-b") {
    return {
      runs: [
        {
          id: 5,
          createdAt: recent,
          completed: true,
          defaultBranch: true,
          conclusion: "success",
          steps: greenSteps,
        },
      ],
      issues: [{ number: 98, title: RELEASE_REHEARSAL_ESCALATION_TITLE, createdAt: stale }],
      historyExists: true,
    };
  }
  if (name === "--fixture-signal-a") {
    return {
      runs: [
        {
          id: 1,
          createdAt: stale,
          completed: true,
          defaultBranch: true,
          conclusion: "success",
          steps: greenSteps,
        },
      ],
      issues: [],
      historyExists: true,
    };
  }
  if (name === "--fixture-partial") {
    const partial = greenSteps.map((step, index) =>
      index === 0 ? { name: step.name, conclusion: "failure" } : step,
    );
    return {
      runs: [
        {
          id: 2,
          createdAt: stale,
          completed: true,
          defaultBranch: true,
          conclusion: "failure",
          steps: partial,
        },
      ],
      issues: [],
      historyExists: true,
    };
  }
  throw new Error(`unknown retirement fixture ${name}`);
}

function ageDays(value: string, timestamp: number): number {
  return Math.floor((timestamp - Date.parse(value)) / 86_400_000);
}

function ghJson<T>(endpoint: string): T {
  return JSON.parse(execFileSync("gh", ["api", endpoint], { encoding: "utf8" })) as T;
}
