import { spawn } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";

const require = createRequire(import.meta.url);
const examplesRequire = createRequire(path.join(process.cwd(), "examples/package.json"));
const { createServer } = await import(examplesRequire.resolve("vite"));
const { omenaCss } = require("../packages/vite-plugin/index.cjs");

async function main() {
  await runHookConvergenceGate();
  await runBrowserDevHmrGate();
}

async function runHookConvergenceGate() {
  const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "omena-vite-hmr-hook-"));
  const srcRoot = path.join(tempRoot, "src");
  const stylePath = path.join(srcRoot, "App.module.scss");
  const warnings = [];
  const invalidatedModules = [];
  let server;

  try {
    fs.mkdirSync(srcRoot, { recursive: true });
    fs.writeFileSync(
      path.join(tempRoot, "index.html"),
      `<script>sessionStorage.setItem("omena-vite-document-load-count", String(Number(sessionStorage.getItem("omena-vite-document-load-count") || "0") + 1)); window.__omenaViteDocumentLoadCount = Number(sessionStorage.getItem("omena-vite-document-load-count"));</script><div id="app"></div><script type="module" src="/src/main.js"></script>`,
    );
    fs.writeFileSync(
      path.join(srcRoot, "main.js"),
      `import styles from "./App.module.scss";\ndocument.querySelector("#app").className = styles.root;\n`,
    );
    fs.writeFileSync(stylePath, "/* omena */\n.root { color: red; }\n", "utf8");

    const plugin = omenaCss({
      include: /\.module\.scss$/,
      passes: ["comment-strip"],
      sourceMap: true,
      configFile: false,
      cwd: tempRoot,
    });
    server = await createServer({
      root: tempRoot,
      logLevel: "silent",
      plugins: [plugin],
      server: {
        port: 0,
        hmr: false,
      },
    });
    await server.listen();
    plugin.configureServer?.(server);

    const first = await plugin.transform.call(
      { warn: (message) => warnings.push(message) },
      fs.readFileSync(stylePath, "utf8"),
      stylePath,
    );
    assertTransformIncludes(first, "red", "initial transform");
    assertSourceMap(first, "initial transform");

    const module = { id: stylePath };
    const updateResults = [];
    server.moduleGraph.invalidateModule = (mod) => {
      invalidatedModules.push(mod);
    };
    for (const color of ["green", "orange", "blue"]) {
      fs.writeFileSync(stylePath, `/* omena */\n.root { color: ${color}; }\n`, "utf8");
      updateResults.push(
        await plugin.handleHotUpdate({
          file: stylePath,
          modules: [module],
          server,
        }),
      );
    }

    const second = await plugin.transform.call(
      { warn: (message) => warnings.push(message) },
      fs.readFileSync(stylePath, "utf8"),
      stylePath,
    );
    assertTransformIncludes(second, "blue", "rapid-edit transform");
    assertSourceMap(second, "rapid-edit transform");
    if (second?.code?.includes("green") || second?.code?.includes("orange")) {
      throw new Error("Vite HMR transform cache retained a stale intermediate edit.");
    }
    const usedCustomRuntimeUpdates = updateResults.every(
      (result) => Array.isArray(result) && result.length === 0,
    );
    if (!usedCustomRuntimeUpdates && !invalidatedModules.includes(module)) {
      throw new Error("Vite module graph was not invalidated for the changed style module.");
    }
    if (warnings.length > 0) {
      throw new Error(`Unexpected Vite plugin warnings: ${warnings.join(" | ")}`);
    }
  } finally {
    await server?.close();
    fs.rmSync(tempRoot, { recursive: true, force: true });
  }
}

async function runBrowserDevHmrGate() {
  const chromePath = findChromeBinary();
  if (!chromePath) {
    throw new Error(
      "Chrome/Chromium executable not found. Set CHROME_BIN to run the Vite HMR gate.",
    );
  }

  const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "omena-vite-hmr-browser-"));
  const srcRoot = path.join(tempRoot, "src");
  const stylePath = path.join(srcRoot, "App.module.scss");
  let server;
  let chrome;
  let page;

  try {
    fs.mkdirSync(srcRoot, { recursive: true });
    fs.writeFileSync(
      path.join(tempRoot, "index.html"),
      `<script>sessionStorage.setItem("omena-vite-document-load-count", String(Number(sessionStorage.getItem("omena-vite-document-load-count") || "0") + 1));</script><div id="app"></div><script type="module" src="/src/main.js"></script>`,
    );
    fs.writeFileSync(
      path.join(srcRoot, "main.js"),
      [
        `import styles from "./App.module.scss";`,
        `window.__omenaViteModuleEvalCount = (window.__omenaViteModuleEvalCount ?? 0) + 1;`,
        `window.__omenaViteConnected = false;`,
        `if (import.meta.hot) import.meta.hot.on("vite:ws:connect", () => { window.__omenaViteConnected = true; });`,
        `const app = document.querySelector("#app");`,
        `app.className = styles.root;`,
        `app.dataset.className = styles.root;`,
        `window.__readOmenaViteState = () => ({`,
        `  color: getComputedStyle(app).color,`,
        `  className: app.className,`,
        `  dataClassName: app.dataset.className,`,
        `  documentLoadCount: Number(sessionStorage.getItem("omena-vite-document-load-count") || "0"),`,
        `  moduleEvalCount: window.__omenaViteModuleEvalCount,`,
        `  timeOrigin: performance.timeOrigin,`,
        `});`,
        `window.__readOmenaBuildSummary = async () => (await import("virtual:omena-css/build-summary")).default;`,
      ].join("\n"),
    );
    fs.writeFileSync(stylePath, "/* omena */\n.root { color: red; }\n", "utf8");

    server = await createServer({
      root: tempRoot,
      logLevel: "silent",
      plugins: [
        omenaCss({
          include: /\.module\.scss$/,
          passes: ["comment-strip"],
          sourceMap: true,
          configFile: false,
          cwd: tempRoot,
        }),
      ],
      server: {
        host: "127.0.0.1",
        port: 0,
      },
    });
    await server.listen();
    const address = server.httpServer.address();
    const url = `http://127.0.0.1:${address.port}/`;

    chrome = await launchChrome(chromePath, tempRoot);
    page = await openCdpPage(chrome.debugPort, url);
    const browserDiagnostics = collectBrowserDiagnostics(page);

    const initial = await waitForPageState(page, (state) => state?.color === "rgb(255, 0, 0)");
    if (typeof initial.timeOrigin !== "number" || initial.timeOrigin <= 0) {
      throw new Error(`Expected initial browser timeOrigin, got ${JSON.stringify(initial)}.`);
    }
    await waitForValue(
      () => evaluate(page, `window.__omenaViteConnected === true`),
      (connected) => connected === true,
      5_000,
    );
    await delay(150);
    const editBaseline = await waitForPageState(page, (state) => state?.color === "rgb(255, 0, 0)");

    for (const color of ["green", "orange", "blue"]) {
      fs.writeFileSync(stylePath, `/* omena */\n.root { color: ${color}; }\n`, "utf8");
      await delay(35);
    }

    const finalState = await waitForPageState(
      page,
      (state) => state?.color === "rgb(0, 0, 255)",
      12_000,
    );
    if (finalState.timeOrigin !== editBaseline.timeOrigin) {
      throw new Error(
        `Vite CSS HMR triggered a full reload during style edits: initial=${editBaseline.timeOrigin} final=${finalState.timeOrigin}.`,
      );
    }
    if (!finalState.className || finalState.className !== finalState.dataClassName) {
      throw new Error(`CSS Module class binding drifted after HMR: ${JSON.stringify(finalState)}`);
    }

    const summary = await evaluate(page, `(async () => await window.__readOmenaBuildSummary())()`);
    const sourceMapSources = summary.flatMap((entry) => entry.sourceMapSources ?? []);
    if (!sourceMapSources.some((source) => source.endsWith("App.module.scss"))) {
      throw new Error(
        `Expected browser-visible Omena build summary to retain App.module.scss source map provenance, got ${JSON.stringify(summary)}`,
      );
    }
    if (browserDiagnostics.length > 0) {
      throw new Error(
        `Browser emitted diagnostics during Vite HMR: ${browserDiagnostics.join(" | ")}`,
      );
    }
  } finally {
    page?.close();
    if (chrome) await stopChrome(chrome.process);
    await server?.close();
    fs.rmSync(tempRoot, { recursive: true, force: true });
  }
}

function assertTransformIncludes(result, expected, label) {
  if (!result || typeof result.code !== "string") {
    throw new Error(`${label}: expected Vite transform result.`);
  }
  if (!result.code.includes(expected)) {
    throw new Error(
      `${label}: expected transformed module to include ${expected}, got ${result.code}`,
    );
  }
}

function assertSourceMap(result, label) {
  if (!result?.map || result.map.version !== 3) {
    throw new Error(`${label}: expected Source Map V3 payload, got ${JSON.stringify(result?.map)}`);
  }
  const sources = result.map.sources ?? [];
  if (!sources.some((source) => source.endsWith("App.module.scss"))) {
    throw new Error(
      `${label}: expected source map to retain App.module.scss, got ${JSON.stringify(result.map)}`,
    );
  }
}

function findChromeBinary() {
  const candidates = [
    process.env.CHROME_BIN,
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    "/Applications/Chromium.app/Contents/MacOS/Chromium",
    "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
    "/usr/bin/google-chrome",
    "/usr/bin/google-chrome-stable",
    "/usr/bin/chromium",
    "/usr/bin/chromium-browser",
  ].filter(Boolean);
  return (
    candidates.find((candidate) => fs.existsSync(candidate) && isExecutable(candidate)) ?? null
  );
}

function isExecutable(filePath) {
  try {
    fs.accessSync(filePath, fs.constants.X_OK);
    return true;
  } catch {
    return false;
  }
}

async function launchChrome(chromePath, workspaceRoot) {
  const userDataDir = fs.mkdtempSync(path.join(os.tmpdir(), "omena-vite-hmr-chrome-"));
  const chrome = spawn(
    chromePath,
    [
      "--headless=new",
      "--remote-debugging-address=127.0.0.1",
      "--remote-debugging-port=0",
      `--user-data-dir=${userDataDir}`,
      "--disable-background-networking",
      "--disable-dev-shm-usage",
      "--disable-gpu",
      "--no-default-browser-check",
      "--no-first-run",
      "--disable-extensions",
      "about:blank",
    ],
    {
      cwd: workspaceRoot,
      stdio: ["ignore", "ignore", "pipe"],
    },
  );

  const debugUrl = await new Promise((resolve, reject) => {
    let stderr = "";
    const timeout = setTimeout(() => {
      reject(new Error(`Timed out waiting for Chrome DevTools endpoint. stderr=${stderr}`));
    }, 10_000);
    chrome.once("exit", (code, signal) => {
      clearTimeout(timeout);
      reject(
        new Error(
          `Chrome exited before DevTools was ready: code=${code} signal=${signal} stderr=${stderr}`,
        ),
      );
    });
    chrome.stderr.setEncoding("utf8");
    chrome.stderr.on("data", (chunk) => {
      stderr += chunk;
      const match = /DevTools listening on (ws:\/\/[^\s]+)/.exec(stderr);
      if (match) {
        clearTimeout(timeout);
        resolve(match[1]);
      }
    });
  });

  const debugPort = Number(new URL(debugUrl).port);
  return { debugPort, process: chrome, userDataDir };
}

async function openCdpPage(debugPort, url) {
  const response = await fetch(
    `http://127.0.0.1:${debugPort}/json/new?${encodeURIComponent("about:blank")}`,
    {
      method: "PUT",
    },
  );
  if (!response.ok) {
    throw new Error(`Failed to create Chrome target: ${response.status} ${await response.text()}`);
  }
  const target = await response.json();
  const page = await CdpConnection.open(target.webSocketDebuggerUrl);
  await page.send("Runtime.enable");
  await page.send("Page.enable");
  const loaded = waitForCdpEvent(page, "Page.loadEventFired", 10_000);
  await page.send("Page.navigate", { url });
  await loaded;
  return page;
}

function collectBrowserDiagnostics(page) {
  const diagnostics = [];
  page.on("Runtime.exceptionThrown", (event) => {
    diagnostics.push(event.exceptionDetails?.text ?? JSON.stringify(event.exceptionDetails));
  });
  page.on("Runtime.consoleAPICalled", (event) => {
    const type = event.type ?? "log";
    if (!["error", "warning"].includes(type)) return;
    const text = (event.args ?? []).map((arg) => arg.value ?? arg.description ?? "").join(" ");
    if (text.includes("favicon.ico")) return;
    diagnostics.push(`${type}: ${text}`);
  });
  page.on("Log.entryAdded", (event) => {
    if (event.entry?.level === "error") {
      if (event.entry.text.includes("favicon.ico")) return;
      if (
        event.entry.text ===
        "Failed to load resource: the server responded with a status of 404 (Not Found)"
      ) {
        return;
      }
      diagnostics.push(event.entry.text);
    }
  });
  page.on("Network.responseReceived", (event) => {
    const status = event.response?.status ?? 0;
    const url = event.response?.url ?? "";
    if (status >= 400 && !url.endsWith("/favicon.ico")) {
      diagnostics.push(`${status} ${url}`);
    }
  });
  page.send("Log.enable").catch((error) => {
    diagnostics.push(`Log.enable failed: ${error.message}`);
  });
  page.send("Network.enable").catch((error) => {
    diagnostics.push(`Network.enable failed: ${error.message}`);
  });
  return diagnostics;
}

async function waitForPageState(page, predicate, timeoutMs = 8_000) {
  return waitForValue(
    async () =>
      evaluate(
        page,
        `(() => {
          if (!window.__readOmenaViteState) return null;
          return window.__readOmenaViteState();
        })()`,
      ),
    predicate,
    timeoutMs,
  );
}

async function waitForValue(read, predicate, timeoutMs) {
  const deadline = Date.now() + timeoutMs;
  let lastValue;
  while (Date.now() < deadline) {
    lastValue = await read();
    if (predicate(lastValue)) return lastValue;
    await delay(100);
  }
  throw new Error(`Timed out waiting for browser state. Last value: ${JSON.stringify(lastValue)}`);
}

async function evaluate(page, expression) {
  const response = await page.send("Runtime.evaluate", {
    expression,
    awaitPromise: true,
    returnByValue: true,
    userGesture: true,
  });
  if (response.exceptionDetails) {
    throw new Error(`Browser evaluation failed: ${JSON.stringify(response.exceptionDetails)}`);
  }
  return response.result?.value;
}

function waitForCdpEvent(connection, eventName, timeoutMs) {
  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => {
      cleanup();
      reject(new Error(`Timed out waiting for CDP event ${eventName}.`));
    }, timeoutMs);
    const cleanup = connection.on(eventName, (event) => {
      clearTimeout(timeout);
      cleanup();
      resolve(event);
    });
  });
}

async function stopChrome(chrome) {
  if (chrome.exitCode !== null || chrome.signalCode !== null) return;
  chrome.kill("SIGTERM");
  await Promise.race([
    new Promise((resolve) => chrome.once("exit", resolve)),
    delay(2_000).then(() => chrome.kill("SIGKILL")),
  ]);
}

function delay(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

class CdpConnection {
  static async open(url) {
    const socket = new WebSocket(url);
    const connection = new CdpConnection(socket);
    await new Promise((resolve, reject) => {
      const timeout = setTimeout(
        () => reject(new Error(`Timed out connecting to ${url}.`)),
        10_000,
      );
      socket.addEventListener("open", () => {
        clearTimeout(timeout);
        resolve();
      });
      socket.addEventListener("error", (event) => {
        clearTimeout(timeout);
        reject(new Error(`CDP websocket error: ${event.message ?? "unknown"}`));
      });
    });
    return connection;
  }

  constructor(socket) {
    this.socket = socket;
    this.nextId = 1;
    this.pending = new Map();
    this.listeners = new Map();
    this.socket.addEventListener("message", (event) => this.handleMessage(event.data));
    this.socket.addEventListener("close", () => {
      for (const { reject } of this.pending.values()) {
        reject(new Error("CDP websocket closed."));
      }
      this.pending.clear();
    });
  }

  send(method, params = {}) {
    const id = this.nextId++;
    const payload = JSON.stringify({ id, method, params });
    return new Promise((resolve, reject) => {
      this.pending.set(id, { resolve, reject });
      this.socket.send(payload);
    });
  }

  on(eventName, listener) {
    const listeners = this.listeners.get(eventName) ?? new Set();
    listeners.add(listener);
    this.listeners.set(eventName, listeners);
    return () => listeners.delete(listener);
  }

  close() {
    this.socket.close();
  }

  handleMessage(data) {
    const message = JSON.parse(data);
    if (message.id) {
      const pending = this.pending.get(message.id);
      if (!pending) return;
      this.pending.delete(message.id);
      if (message.error) {
        pending.reject(
          new Error(`${message.error.message}: ${JSON.stringify(message.error.data)}`),
        );
      } else {
        pending.resolve(message.result ?? {});
      }
      return;
    }

    const listeners = this.listeners.get(message.method);
    if (!listeners) return;
    for (const listener of listeners) {
      listener(message.params ?? {});
    }
  }
}

await main();
