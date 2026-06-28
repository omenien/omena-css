import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import path from "node:path";

const repoRoot = process.cwd();
const rustManifest = path.join(repoRoot, "rust/Cargo.toml");
const runnerBinary = path.join(
  repoRoot,
  "rust/target/debug",
  process.platform === "win32" ? "engine-shadow-runner.exe" : "engine-shadow-runner",
);
const stylePath = "/tmp/DaemonSmoke.module.scss";
const sourcePath = "/tmp/DaemonSmoke.tsx";

if (!existsSync(runnerBinary)) {
  const runnerBuild = spawnSync(
    "cargo",
    ["build", "--quiet", "--manifest-path", rustManifest, "-p", "engine-shadow-runner"],
    {
      cwd: repoRoot,
      encoding: "utf8",
      maxBuffer: 8 * 1024 * 1024,
    },
  );

  assert.equal(runnerBuild.error, undefined);
  assert.equal(runnerBuild.status, 0, runnerBuild.stderr);
}

assert.equal(
  existsSync(runnerBinary),
  true,
  `Missing engine-shadow-runner binary: ${runnerBinary}`,
);

const engineInput = {
  version: "2",
  workspace: {
    root: "/tmp",
    classnameTransform: "asIs",
    settingsKey: "engine-shadow-runner-daemon-smoke",
  },
  sources: [
    {
      filePath: sourcePath,
      document: {
        classExpressions: [
          {
            id: "expr:button",
            kind: "literal",
            scssModulePath: stylePath,
            range: range(4, 12, 4, 20),
            className: "button",
            rootBindingDeclId: null,
            accessPath: null,
          },
        ],
      },
    },
  ],
  styles: [
    {
      filePath: stylePath,
      document: {
        selectors: [
          {
            name: "button",
            viewKind: "canonical",
            canonicalName: "button",
            range: range(0, 1, 0, 7),
            nestedSafety: "flat",
            composes: null,
            bemSuffix: null,
          },
        ],
      },
    },
  ],
  typeFacts: [
    {
      filePath: sourcePath,
      expressionId: "expr:button",
      facts: {
        kind: "exact",
        constraintKind: null,
        values: ["button"],
        prefix: null,
        suffix: null,
        minLen: null,
        maxLen: null,
        charMust: null,
        charMay: null,
        mayIncludeOtherChars: null,
      },
    },
  ],
};

const child = spawn(runnerBinary, ["--daemon"], {
  cwd: repoRoot,
  stdio: ["pipe", "pipe", "pipe"],
});

const pending = new Map();
const stderr = [];
let stdoutBuffer = "";
let requestId = 0;
let closeWaitReject = null;

child.stdout.setEncoding("utf8");
child.stderr.setEncoding("utf8");
child.stdout.on("data", (chunk) => {
  stdoutBuffer += chunk;
  while (stdoutBuffer.includes("\n")) {
    const newlineIndex = stdoutBuffer.indexOf("\n");
    const line = stdoutBuffer.slice(0, newlineIndex).trim();
    stdoutBuffer = stdoutBuffer.slice(newlineIndex + 1);
    if (!line) continue;

    const response = JSON.parse(line);
    const slot = pending.get(response.id);
    if (!slot) {
      child.kill("SIGTERM");
      throw new Error(`Unexpected daemon response id: ${String(response.id)}`);
    }
    pending.delete(response.id);
    if (response.ok) {
      slot.resolve(response.result);
    } else {
      slot.reject(new Error(response.error ?? "daemon request failed"));
    }
  }
});
child.stderr.on("data", (chunk) => stderr.push(chunk));
child.once("error", (error) => {
  for (const slot of pending.values()) slot.reject(error);
  pending.clear();
});
child.once("close", (code) => {
  if (pending.size === 0) return;
  const error = new Error(
    [
      `engine-shadow-runner daemon exited before all requests completed: ${code}`,
      stderr.join("").trim(),
    ]
      .filter(Boolean)
      .join("\n"),
  );
  for (const slot of pending.values()) slot.reject(error);
  pending.clear();
});

const timeout = setTimeout(() => {
  const error = new Error("engine-shadow-runner daemon smoke timed out");
  child.kill("SIGTERM");
  for (const slot of pending.values()) {
    slot.reject(error);
  }
  pending.clear();
  closeWaitReject?.(error);
}, 30_000);

try {
  const sourceResolution = await sendDaemonRequest(
    "input-source-resolution-canonical-producer",
    engineInput,
  );
  assert.equal(sourceResolution.schemaVersion, "0");
  assert.equal(sourceResolution.evaluatorCandidates.results.length, 1);
  assert.deepEqual(sourceResolution.evaluatorCandidates.results[0].payload.selectorNames, [
    "button",
  ]);

  const firstRuntime = await sendDaemonRequest("input-omena-query-evaluation-runtime", engineInput);
  assert.equal(firstRuntime.product, "omena-query.evaluation-runtime");
  assert.equal(firstRuntime.expressionDomainGraphCount, 1);
  assert.equal(firstRuntime.expressionDomainDirtyGraphCount, 1);
  assert.equal(firstRuntime.expressionDomainReusedGraphCount, 0);

  const secondRuntime = await sendDaemonRequest(
    "input-omena-query-evaluation-runtime",
    engineInput,
  );
  assert.equal(secondRuntime.product, "omena-query.evaluation-runtime");
  assert.equal(secondRuntime.expressionDomainRevision, firstRuntime.expressionDomainRevision + 1);
  assert.equal(secondRuntime.expressionDomainGraphCount, firstRuntime.expressionDomainGraphCount);
  assert.equal(secondRuntime.expressionDomainDirtyGraphCount, 0);
  assert.equal(
    secondRuntime.expressionDomainReusedGraphCount,
    secondRuntime.expressionDomainGraphCount,
  );

  const graphBatch = await sendDaemonRequest("style-semantic-graph-batch", {
    styles: [{ stylePath, styleSource: ".button { color: red; }" }],
    engineInput,
  });
  assert.equal(graphBatch.product, "omena-semantic.style-semantic-graph-batch");
  assert.equal(graphBatch.graphs.length, 1);
  assert.equal(graphBatch.graphs[0].stylePath, stylePath);
  assert.equal(graphBatch.graphs[0].graph.selectorReferenceEngine.totalReferenceSites, 1);

  const childClosed = onceClosed(child);
  const closeTimedOut = new Promise((_, reject) => {
    closeWaitReject = reject;
  });
  child.stdin.end();
  await Promise.race([childClosed, closeTimedOut]);
  clearTimeout(timeout);
  process.stdout.write(
    "engine-shadow-runner daemon ok: requests=4 mode=selected-query incrementalReuse=on\n",
  );
} catch (error) {
  clearTimeout(timeout);
  child.kill("SIGTERM");
  throw error;
}

function sendDaemonRequest(command, input) {
  const id = `request:${++requestId}`;
  return new Promise((resolve, reject) => {
    pending.set(id, { resolve, reject });
    child.stdin.write(`${JSON.stringify({ id, command, input })}\n`);
  });
}

function onceClosed(processHandle) {
  return new Promise((resolve, reject) => {
    processHandle.once("close", (code) => {
      if (code === 0) {
        resolve();
        return;
      }
      reject(
        new Error(
          [`engine-shadow-runner daemon exited with code ${code}`, stderr.join("").trim()]
            .filter(Boolean)
            .join("\n"),
        ),
      );
    });
  });
}

function range(startLine, startCharacter, endLine, endCharacter) {
  return {
    start: { line: startLine, character: startCharacter },
    end: { line: endLine, character: endCharacter },
  };
}
