import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import {
  runCapturedCommand,
  runLiveCommand,
} from "../../../packages/check-orchestrator/src/cli/execution";

describe("check orchestrator command execution", () => {
  const tempDirs: string[] = [];

  afterEach(() => {
    for (const tempDir of tempDirs.splice(0)) {
      rmSync(tempDir, { force: true, recursive: true });
    }
  });

  it("terminates commands that exceed their declared timeout", async () => {
    const result = await runCapturedCommand(
      {
        executable: process.execPath,
        args: ["-e", "setInterval(() => {}, 1_000)"],
      },
      {
        cwd: process.cwd(),
        env: process.env,
        timeoutMs: 75,
        maxOutputBytes: 1_024,
      },
    );

    expect(result).toMatchObject({
      code: 124,
      error: null,
      timedOut: true,
    });
  });

  it("applies the same process-tree timeout to live commands", async () => {
    const result = await runLiveCommand(
      {
        executable: process.execPath,
        args: ["-e", "setInterval(() => {}, 1_000)"],
      },
      {
        cwd: process.cwd(),
        env: process.env,
        timeoutMs: 75,
      },
    );

    expect(result).toMatchObject({
      code: 124,
      error: null,
      timedOut: true,
    });
  });

  it("keeps a bounded tail and spills the complete output", async () => {
    const tempDir = mkdtempSync(path.join(os.tmpdir(), "omena-check-output-"));
    tempDirs.push(tempDir);
    const spillPath = path.join(tempDir, "gate.log");
    const result = await runCapturedCommand(
      {
        executable: process.execPath,
        args: ["-e", "process.stdout.write('a'.repeat(128))"],
      },
      {
        cwd: process.cwd(),
        env: process.env,
        maxOutputBytes: 32,
        spillPath,
      },
    );

    expect(result).toMatchObject({
      code: 0,
      output: "a".repeat(32),
      outputBytes: 128,
      outputTruncated: true,
      outputPath: spillPath,
      timedOut: false,
    });
    expect(readFileSync(spillPath, "utf8")).toBe("a".repeat(128));
  });

  it("does not create a spill file for bounded output", async () => {
    const tempDir = mkdtempSync(path.join(os.tmpdir(), "omena-check-output-"));
    tempDirs.push(tempDir);
    const spillPath = path.join(tempDir, "gate.log");
    const result = await runCapturedCommand(
      {
        executable: process.execPath,
        args: ["-e", "process.stdout.write('ok')"],
      },
      {
        cwd: process.cwd(),
        env: process.env,
        maxOutputBytes: 32,
        spillPath,
      },
    );

    expect(result).toMatchObject({
      code: 0,
      output: "ok",
      outputBytes: 2,
      outputTruncated: false,
      outputPath: null,
      timedOut: false,
    });
  });
});
