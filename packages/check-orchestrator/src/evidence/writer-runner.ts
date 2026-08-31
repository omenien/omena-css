import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, readFileSync, unlinkSync, writeFileSync } from "node:fs";
import path from "node:path";

export interface DigestPinnedWriterInput {
  readonly repoRoot: string;
  readonly command: readonly string[];
  readonly inputPaths: readonly string[];
  readonly outputPaths: readonly string[];
  readonly onStartedForTest?: () => void | Promise<void>;
}

export interface DigestPinnedWriterResult {
  readonly command: readonly string[];
  readonly outputPaths: readonly string[];
  readonly exitCode: number;
}

export function resolveEvidenceWriterCommand(
  command: readonly string[],
  requiredEnvironmentKeys: readonly string[],
  environment: NodeJS.ProcessEnv = process.env,
): readonly string[] {
  const declaredEnvironmentKeys = new Set(requiredEnvironmentKeys);
  const missingEnvironmentKeys = [...declaredEnvironmentKeys]
    .filter((environmentKey) => !environment[environmentKey])
    .toSorted();
  if (missingEnvironmentKeys.length > 0) {
    throw new Error(
      `evidence writer NOT-PREVIEWABLE external inputs are absent: ${missingEnvironmentKeys.join(", ")}`,
    );
  }
  return command.map((word) =>
    word.replace(/\$\{([A-Z0-9_]+)\}/gu, (_match, environmentKey: string) => {
      if (!declaredEnvironmentKeys.has(environmentKey)) {
        throw new Error(
          `evidence writer command references undeclared environment key ${environmentKey}`,
        );
      }
      const value = environment[environmentKey];
      if (value === undefined) {
        throw new Error(
          `evidence writer command environment key became unavailable ${environmentKey}`,
        );
      }
      return value;
    }),
  );
}

export async function runDigestPinnedWriter(
  input: DigestPinnedWriterInput,
): Promise<DigestPinnedWriterResult> {
  const [executable, ...args] = input.command;
  if (!executable) throw new Error("evidence writer command is empty");
  const inputsBefore = digestPaths(input.repoRoot, input.inputPaths);
  const outputsBefore = snapshotOutputs(input.repoRoot, input.outputPaths);
  const child = spawn(executable, args, {
    cwd: input.repoRoot,
    env: process.env,
    shell: false,
    stdio: "inherit",
  });
  await input.onStartedForTest?.();
  const exitCode = await new Promise<number>((resolve, reject) => {
    child.once("error", reject);
    child.once("close", (code) => resolve(code ?? 1));
  });
  if (exitCode !== 0) {
    restoreOutputs(input.repoRoot, outputsBefore);
    throw new Error(`evidence writer exited ${exitCode}: ${input.command.join(" ")}`);
  }
  const inputsAfter = digestPaths(input.repoRoot, input.inputPaths);
  const changedInput = [...inputsBefore].find(
    ([inputPath, digest]) => inputsAfter.get(inputPath) !== digest,
  );
  if (changedInput) {
    restoreOutputs(input.repoRoot, outputsBefore);
    throw new Error(
      `evidence writer concurrent-skew: input changed before post-exit acceptance: ${changedInput[0]}`,
    );
  }
  const missingOutput = [...new Set(input.outputPaths)].find(
    (outputPath) => !existsSync(path.join(input.repoRoot, outputPath)),
  );
  if (missingOutput) {
    restoreOutputs(input.repoRoot, outputsBefore);
    throw new Error(
      `evidence writer successful no-op: declared output was not reproduced: ${missingOutput}`,
    );
  }
  return { command: [...input.command], outputPaths: [...input.outputPaths], exitCode };
}

function digestPaths(repoRoot: string, paths: readonly string[]): ReadonlyMap<string, string> {
  return new Map(
    [...new Set(paths)].toSorted().map((inputPath) => {
      const absolutePath = path.join(repoRoot, inputPath);
      const digest = existsSync(absolutePath)
        ? createHash("sha256").update(readFileSync(absolutePath)).digest("hex")
        : "ABSENT";
      return [inputPath, digest] as const;
    }),
  );
}

interface OutputSnapshot {
  readonly outputPath: string;
  readonly bytes: Buffer | null;
}

function snapshotOutputs(repoRoot: string, paths: readonly string[]): readonly OutputSnapshot[] {
  return [...new Set(paths)].map((outputPath) => {
    const absolutePath = path.join(repoRoot, outputPath);
    return {
      outputPath,
      bytes: existsSync(absolutePath) ? readFileSync(absolutePath) : null,
    };
  });
}

function restoreOutputs(repoRoot: string, snapshots: readonly OutputSnapshot[]): void {
  for (const snapshot of snapshots) {
    const absolutePath = path.join(repoRoot, snapshot.outputPath);
    if (snapshot.bytes === null) {
      if (existsSync(absolutePath)) unlinkSync(absolutePath);
    } else {
      writeFileSync(absolutePath, snapshot.bytes);
    }
  }
}
