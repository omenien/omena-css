import { strict as assert } from "node:assert";
import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { parse as parseYaml } from "yaml";

// ci.yml becomes a generated artifact. The registry carries every job
// BLOCK VERBATIM (REV4: run bodies, matrices, permissions, artifact names are
// carried, not synthesized) plus structured metadata that generation VALIDATES
// against the block — so byte-identity is achievable while the derived facts
// (annotations, needs, the ci-required set) stay machine-checked. Deleting a
// registry entry removes exactly that job (the template-hardcoding killer).

export interface CiWorkflowJobEntry {
  readonly name: string;
  /** Verbatim job block lines, INCLUDING the `  <name>:` header line. */
  readonly block: readonly string[];
  readonly requiredAnnotation: boolean | null;
  readonly tierAnnotation: string | null;
  readonly needs: readonly string[];
}

export interface CiWorkflowRegistry {
  readonly schemaVersion: "0";
  readonly product: "omena.check-orchestrator.ci-workflow";
  /** Verbatim lines before the first job (name, on, concurrency, permissions, `jobs:`). */
  readonly header: readonly string[];
  readonly jobs: readonly CiWorkflowJobEntry[];
}

export function ciWorkflowRegistryPath(rootDir: string): string {
  return path.join(rootDir, "packages/check-orchestrator/ci-workflow.json");
}

export function ciWorkflowPath(rootDir: string): string {
  return path.join(rootDir, ".github/workflows/ci.yml");
}

export function renderCiWorkflow(registry: CiWorkflowRegistry): string {
  const lines = [...registry.header];
  for (const job of registry.jobs) lines.push(...job.block);
  return `${lines.join("\n")}\n`;
}

const JOB_HEADER = /^ {2}([A-Za-z0-9_-]+):\s*$/;
const TIER_ANNOTATION = /^\s*#\s*omena-ci-tier:\s*([A-Za-z0-9_-]+)\s*$/;
const REQUIRED_ANNOTATION = /^\s*#\s*omena-ci-required:\s*(true|false)\s*$/;

export function adoptCiWorkflow(source: string): CiWorkflowRegistry {
  const lines = source.replace(/\n$/, "").split("\n");
  const jobsIndex = lines.findIndex((line) => line === "jobs:");
  assert.ok(jobsIndex >= 0, "ci.yml must contain a top-level jobs: key");

  const boundaries: Array<{ name: string; start: number }> = [];
  for (let index = jobsIndex + 1; index < lines.length; index += 1) {
    const name = lines[index]?.match(JOB_HEADER)?.[1];
    if (name) boundaries.push({ name, start: index });
  }
  assert.ok(boundaries.length > 0, "ci.yml must declare at least one job");

  const firstJobStart = boundaries[0]?.start ?? lines.length;
  const header = lines.slice(0, firstJobStart);
  const jobs: CiWorkflowJobEntry[] = boundaries.map((boundary, position) => {
    const end = boundaries[position + 1]?.start ?? lines.length;
    const block = lines.slice(boundary.start, end);
    return {
      name: boundary.name,
      block,
      requiredAnnotation: parseAnnotation(block, REQUIRED_ANNOTATION),
      tierAnnotation: parseTier(block),
      needs: parseNeeds(block),
    };
  });
  return {
    schemaVersion: "0",
    product: "omena.check-orchestrator.ci-workflow",
    header,
    jobs,
  };
}

function parseAnnotation(block: readonly string[], pattern: RegExp): boolean | null {
  for (const line of block) {
    const value = line.match(pattern)?.[1];
    if (value === "true") return true;
    if (value === "false") return false;
  }
  return null;
}

function parseTier(block: readonly string[]): string | null {
  for (const line of block) {
    const value = line.match(TIER_ANNOTATION)?.[1];
    if (value) return value;
  }
  return null;
}

function parseNeeds(block: readonly string[]): readonly string[] {
  const needsIndex = block.findIndex((line) => /^ {4}needs:/.test(line));
  if (needsIndex < 0) return [];
  const needsLine = block[needsIndex] ?? "";
  const scalar = needsLine.match(/^ {4}needs:\s*([A-Za-z0-9_-]+)\s*$/)?.[1];
  if (scalar) return [scalar];
  const inline = needsLine.match(/^ {4}needs:\s*\[([^\]]*)\]\s*$/)?.[1];
  if (inline !== undefined) {
    return inline
      .split(",")
      .map((value) => value.trim())
      .filter(Boolean);
  }
  const needs: string[] = [];
  for (const line of block.slice(needsIndex + 1)) {
    const item = line.match(/^ {6}-\s*([A-Za-z0-9_-]+)\s*$/)?.[1];
    if (item) {
      needs.push(item);
      continue;
    }
    if (/^ {0,4}\S/.test(line)) break;
  }
  return needs;
}

export interface CiWorkflowValidationResult {
  readonly errors: readonly string[];
}

/**
 * Structured-vs-verbatim coherence: the registry's machine-readable fields must
 * agree with what the verbatim block actually says, and the ci-required job's
 * needs must equal exactly the required-annotated jobs. Generation refuses to
 * emit an incoherent registry, so hand edits to either side are caught.
 */
export function validateCiWorkflowRegistry(
  registry: CiWorkflowRegistry,
): CiWorkflowValidationResult {
  const errors: string[] = [];
  const names = new Set<string>();
  for (const job of registry.jobs) {
    if (names.has(job.name)) errors.push(`duplicate job "${job.name}"`);
    names.add(job.name);
    const headerName = job.block[0]?.match(JOB_HEADER)?.[1];
    if (headerName !== job.name) {
      errors.push(`job "${job.name}" block header says "${headerName ?? "<missing>"}"`);
    }
    if (parseAnnotation(job.block, REQUIRED_ANNOTATION) !== job.requiredAnnotation) {
      errors.push(`job "${job.name}" requiredAnnotation disagrees with its block`);
    }
    if (parseTier(job.block) !== job.tierAnnotation) {
      errors.push(`job "${job.name}" tierAnnotation disagrees with its block`);
    }
    const blockNeeds = parseNeeds(job.block);
    if (JSON.stringify(blockNeeds) !== JSON.stringify(job.needs)) {
      errors.push(`job "${job.name}" needs disagree with its block`);
    }
    for (const need of job.needs) {
      if (!registry.jobs.some((candidate) => candidate.name === need)) {
        errors.push(`job "${job.name}" needs unknown job "${need}"`);
      }
    }
  }
  for (const job of registry.jobs) {
    const blockText = job.block.join("\n");
    if (!/^ {4}(runs-on|uses):/m.test(blockText)) {
      errors.push(
        `job "${job.name}" block carries neither runs-on nor uses — a run-body line ` +
          `matching the job-header shape has likely split a real job (phantom job)`,
      );
    }
  }
  // The emitted document must PARSE — a registry hand-edit must never ship an
  // unloadable workflow while the drift gate reports none (hardening review).
  try {
    parseYaml(renderCiWorkflow(registry));
  } catch (error) {
    errors.push(
      `emitted ci.yml does not parse as YAML: ${error instanceof Error ? error.message.split("\n")[0] : String(error)}`,
    );
  }
  // Artifact names must be unique PER RUN (upload-artifact@v7 rejects
  // duplicates): a matrixed job's uploads must carry a `${{ matrix.` suffix,
  // and static names must be globally unique (g131-S0 per-leg naming rule).
  const staticArtifactNames = new Map<string, string>();
  for (const job of registry.jobs) {
    const hasMatrix = job.block.some((line) => /^ {6,}matrix:\s*$/.test(line));
    for (let index = 0; index < job.block.length; index += 1) {
      if (!/- uses: actions\/upload-artifact@/.test(job.block[index] ?? "")) continue;
      for (let cursor = index + 1; cursor < Math.min(index + 6, job.block.length); cursor += 1) {
        const artifactName = job.block[cursor]?.match(/^\s+name:\s*(\S.*)$/)?.[1];
        if (artifactName === undefined) continue;
        if (hasMatrix && !artifactName.includes("${{ matrix.")) {
          errors.push(
            `job "${job.name}" is matrixed but uploads artifact "${artifactName}" without a ` +
              "${{ matrix. }} suffix — parallel legs would collide on the name",
          );
        }
        if (!artifactName.includes("${{")) {
          const owner = staticArtifactNames.get(artifactName);
          if (owner && owner !== job.name) {
            errors.push(
              `artifact name "${artifactName}" is uploaded by both "${owner}" and "${job.name}" — names must be unique per run`,
            );
          }
          staticArtifactNames.set(artifactName, job.name);
        }
        break;
      }
    }
  }
  const ciRequired = registry.jobs.find((job) => job.name === "ci-required");
  if (!ciRequired) {
    errors.push("registry must declare the ci-required aggregator job");
  } else {
    const required = registry.jobs
      .filter((job) => job.requiredAnnotation === true)
      .map((job) => job.name)
      .toSorted();
    const declared = [...ciRequired.needs].toSorted();
    if (JSON.stringify(required) !== JSON.stringify(declared)) {
      errors.push(
        `ci-required needs [${declared.join(", ")}] must equal the required-annotated jobs [${required.join(", ")}]`,
      );
    }
  }
  return { errors };
}

export function loadCiWorkflowRegistry(rootDir: string): CiWorkflowRegistry | null {
  const registryPath = ciWorkflowRegistryPath(rootDir);
  if (!existsSync(registryPath)) return null;
  return JSON.parse(readFileSync(registryPath, "utf8")) as CiWorkflowRegistry;
}

export interface CiWorkflowCheckOutcome {
  readonly ok: boolean;
  readonly reason?: string;
}

export function checkCiWorkflow(rootDir: string): CiWorkflowCheckOutcome {
  const registry = loadCiWorkflowRegistry(rootDir);
  if (!registry) {
    return { ok: false, reason: "packages/check-orchestrator/ci-workflow.json is absent" };
  }
  const validation = validateCiWorkflowRegistry(registry);
  if (validation.errors.length > 0) {
    return { ok: false, reason: validation.errors.join("; ") };
  }
  const rendered = renderCiWorkflow(registry);
  const committed = readFileSync(ciWorkflowPath(rootDir), "utf8");
  if (rendered !== committed) {
    return {
      ok: false,
      reason:
        ".github/workflows/ci.yml differs from the generated registry output; edit the registry " +
        "and run `omena-check ci-workflow --write`, or adopt an intentional Dependabot action-pin " +
        "change with `omena-check ci-workflow --adopt` (other hand edits to ci.yml are not sanctioned)",
    };
  }
  return { ok: true };
}

export function writeCiWorkflow(rootDir: string): void {
  const registry = loadCiWorkflowRegistry(rootDir);
  assert.ok(registry, "packages/check-orchestrator/ci-workflow.json is absent");
  const validation = validateCiWorkflowRegistry(registry);
  assert.deepEqual(validation.errors, [], validation.errors.join("; "));
  writeFileSync(ciWorkflowPath(rootDir), renderCiWorkflow(registry));
}

export function adoptAndWriteRegistry(rootDir: string): CiWorkflowRegistry {
  const registry = adoptCiWorkflow(readFileSync(ciWorkflowPath(rootDir), "utf8"));
  const validation = validateCiWorkflowRegistry(registry);
  assert.deepEqual(validation.errors, [], validation.errors.join("; "));
  const registryPath = ciWorkflowRegistryPath(rootDir);
  writeFileSync(registryPath, `${JSON.stringify(registry, null, 2)}\n`);
  execFileSync(
    process.execPath,
    [path.join(rootDir, "node_modules/oxfmt/bin/oxfmt"), registryPath],
    {
      cwd: rootDir,
      stdio: "ignore",
    },
  );
  return registry;
}

export type CiWorkflowVerdict = "ok" | "override-warning" | "drift-error";

export function resolveCiWorkflowVerdict(
  outcome: CiWorkflowCheckOutcome,
  overrideReason: string | undefined,
): CiWorkflowVerdict {
  if (outcome.ok) return "ok";
  if (overrideReason && overrideReason.trim().length > 0) return "override-warning";
  return "drift-error";
}
