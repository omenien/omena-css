import { spawn, type ChildProcess } from "node:child_process";
import { appendFileSync, mkdirSync, writeFileSync } from "node:fs";
import path from "node:path";

export interface CapturedCommand {
  readonly executable: string;
  readonly args: readonly string[];
}

export interface CapturedCommandOptions {
  readonly cwd: string;
  readonly env: NodeJS.ProcessEnv;
  readonly timeoutMs?: number;
  readonly maxOutputBytes: number;
  readonly spillPath?: string;
}

export interface CapturedCommandResult {
  readonly code: number | null;
  readonly error: Error | null;
  readonly output: string;
  readonly outputBytes: number;
  readonly outputTruncated: boolean;
  readonly outputPath: string | null;
  readonly timedOut: boolean;
}

export interface LiveCommandOptions {
  readonly cwd: string;
  readonly env: NodeJS.ProcessEnv;
  readonly timeoutMs?: number;
}

export interface LiveCommandResult {
  readonly code: number | null;
  readonly error: Error | null;
  readonly timedOut: boolean;
}

const FORCE_KILL_GRACE_MS = 2_000;

export function runCapturedCommand(
  command: CapturedCommand,
  options: CapturedCommandOptions,
): Promise<CapturedCommandResult> {
  return new Promise((resolve) => {
    const output = new BoundedCommandOutput(options.maxOutputBytes, options.spillPath);
    let settled = false;
    let timedOut = false;
    let forceKillTimer: NodeJS.Timeout | null = null;
    const child = spawn(command.executable, command.args, {
      cwd: options.cwd,
      shell: false,
      stdio: ["ignore", "pipe", "pipe"],
      env: options.env,
      detached: process.platform !== "win32",
    });

    const finish = (code: number | null, error: Error | null): void => {
      if (settled) return;
      settled = true;
      if (timeoutTimer) clearTimeout(timeoutTimer);
      if (forceKillTimer) clearTimeout(forceKillTimer);
      resolve({
        code: timedOut && (code === null || code === 0) ? 124 : code,
        error,
        output: output.retainedText(),
        outputBytes: output.totalBytes,
        outputTruncated: output.truncated,
        outputPath: output.persistedPath,
        timedOut,
      });
    };

    child.stdout?.on("data", (chunk: Buffer | string) => output.append(chunk));
    child.stderr?.on("data", (chunk: Buffer | string) => output.append(chunk));
    child.once("error", (error) => finish(null, error));
    child.once("close", (code) => finish(code, null));

    const timeoutTimer =
      options.timeoutMs !== undefined && options.timeoutMs > 0
        ? setTimeout(() => {
            timedOut = true;
            terminateProcessTree(child, "SIGTERM");
            forceKillTimer = setTimeout(
              () => terminateProcessTree(child, "SIGKILL"),
              FORCE_KILL_GRACE_MS,
            );
            forceKillTimer.unref();
          }, options.timeoutMs)
        : null;
    timeoutTimer?.unref();
  });
}

export function runLiveCommand(
  command: CapturedCommand,
  options: LiveCommandOptions,
): Promise<LiveCommandResult> {
  return new Promise((resolve) => {
    let settled = false;
    let timedOut = false;
    let forceKillTimer: NodeJS.Timeout | null = null;
    const child = spawn(command.executable, command.args, {
      cwd: options.cwd,
      shell: false,
      stdio: "inherit",
      env: options.env,
      detached: process.platform !== "win32",
    });

    const finish = (code: number | null, error: Error | null): void => {
      if (settled) return;
      settled = true;
      if (timeoutTimer) clearTimeout(timeoutTimer);
      if (forceKillTimer) clearTimeout(forceKillTimer);
      resolve({
        code: timedOut && (code === null || code === 0) ? 124 : code,
        error,
        timedOut,
      });
    };

    child.once("error", (error) => finish(null, error));
    child.once("close", (code) => finish(code, null));

    const timeoutTimer =
      options.timeoutMs !== undefined && options.timeoutMs > 0
        ? setTimeout(() => {
            timedOut = true;
            terminateProcessTree(child, "SIGTERM");
            forceKillTimer = setTimeout(
              () => terminateProcessTree(child, "SIGKILL"),
              FORCE_KILL_GRACE_MS,
            );
            forceKillTimer.unref();
          }, options.timeoutMs)
        : null;
    timeoutTimer?.unref();
  });
}

class BoundedCommandOutput {
  private retained: Buffer<ArrayBufferLike> = Buffer.alloc(0);
  private spilled = false;
  totalBytes = 0;

  constructor(
    private readonly maxOutputBytes: number,
    private readonly spillPath?: string,
  ) {
    if (!Number.isSafeInteger(maxOutputBytes) || maxOutputBytes < 1) {
      throw new Error(`maxOutputBytes must be a positive safe integer, got ${maxOutputBytes}`);
    }
  }

  append(chunk: Buffer | string): void {
    const bytes = Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk);
    this.totalBytes += bytes.byteLength;

    if (!this.spilled && this.retained.byteLength + bytes.byteLength > this.maxOutputBytes) {
      this.startSpill();
    }
    if (this.spilled && this.spillPath) {
      appendFileSync(this.spillPath, bytes);
    }

    this.retained =
      this.retained.byteLength === 0
        ? bytes.subarray(Math.max(0, bytes.byteLength - this.maxOutputBytes))
        : Buffer.concat([this.retained, bytes]).subarray(-this.maxOutputBytes);
  }

  retainedText(): string {
    return this.retained.toString("utf8");
  }

  get truncated(): boolean {
    return this.totalBytes > this.maxOutputBytes;
  }

  get persistedPath(): string | null {
    return this.spilled && this.spillPath ? this.spillPath : null;
  }

  private startSpill(): void {
    this.spilled = true;
    if (!this.spillPath) return;
    mkdirSync(path.dirname(this.spillPath), { recursive: true });
    writeFileSync(this.spillPath, this.retained);
  }
}

function terminateProcessTree(child: ChildProcess, signal: NodeJS.Signals): void {
  if (!child.pid) return;
  if (process.platform === "win32") {
    try {
      const killer = spawn("taskkill", ["/pid", String(child.pid), "/T", "/F"], {
        stdio: "ignore",
        windowsHide: true,
      });
      killer.unref();
      return;
    } catch {
      // Fall through to the direct child signal when taskkill is unavailable.
    }
  } else {
    try {
      process.kill(-child.pid, signal);
      return;
    } catch {
      // The child may have exited between the timeout and process-group signal.
    }
  }
  try {
    child.kill(signal);
  } catch {
    // Exit is reported through the close event.
  }
}
