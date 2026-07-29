"use client";

import { Braces, CircleAlert, LoaderCircle, Play, RotateCcw } from "lucide-react";
import { useState } from "react";
import { Button } from "./ui/button";
import { deploymentBasePath } from "@/lib/site";

const exampleSource = `.button {
  display: grid;
  animation: enter 180ms ease-out;
}

.button:hover {
  color: oklch(61% 0.18 29);
}`;

type PlaygroundMode = "diagnostics" | "parse";

interface OmenaBrowserModule {
  default: () => Promise<unknown>;
  readStyleDiagnostics: (source: string, path: string) => unknown;
  parseStylesheet: (source: string, path: string) => unknown;
}

let runtimePromise: Promise<OmenaBrowserModule> | undefined;

async function loadRuntime(): Promise<OmenaBrowserModule> {
  runtimePromise ??= import(
    /* webpackIgnore: true */ `${deploymentBasePath}/wasm/omena_wasm.js`
  ).then(async (module: unknown) => {
    const runtime = module as OmenaBrowserModule;
    await runtime.default();
    return runtime;
  });
  return runtimePromise;
}

export function WasmPlayground() {
  const [mode, setMode] = useState<PlaygroundMode>("diagnostics");
  const [source, setSource] = useState(exampleSource);
  const [result, setResult] = useState<string>(
    "Run the browser engine to inspect this stylesheet without sending source to a server.",
  );
  const [status, setStatus] = useState<"idle" | "loading" | "ready" | "error">("idle");

  async function run() {
    setStatus("loading");
    try {
      const runtime = await loadRuntime();
      const value =
        mode === "diagnostics"
          ? runtime.readStyleDiagnostics(source, "playground.module.css")
          : runtime.parseStylesheet(source, "playground.module.css");
      setResult(JSON.stringify(value, null, 2));
      setStatus("ready");
    } catch (error) {
      setResult(error instanceof Error ? error.message : String(error));
      setStatus("error");
    }
  }

  return (
    <section className="omena-playground not-prose" aria-label="Omena WASM playground">
      <div className="omena-playground-header">
        <div>
          <span className="omena-kicker">Runs locally in your browser</span>
          <h2>Inspect CSS with the shipped Rust engine</h2>
        </div>
        <div className="omena-runtime-status" data-status={status}>
          {status === "loading" ? (
            <LoaderCircle aria-hidden="true" className="animate-spin" />
          ) : status === "error" ? (
            <CircleAlert aria-hidden="true" />
          ) : (
            <span aria-hidden="true" className="omena-status-dot" />
          )}
          {status === "loading"
            ? "Loading WASM"
            : status === "error"
              ? "Runtime error"
              : "On device"}
        </div>
      </div>

      <div className="omena-playground-tabs" role="tablist" aria-label="Analysis mode">
        <button
          type="button"
          role="tab"
          aria-selected={mode === "diagnostics"}
          onClick={() => setMode("diagnostics")}
        >
          Diagnostics
        </button>
        <button
          type="button"
          role="tab"
          aria-selected={mode === "parse"}
          onClick={() => setMode("parse")}
        >
          Parse tree
        </button>
      </div>

      <div className="omena-playground-grid">
        <label className="omena-code-panel">
          <span>playground.module.css</span>
          <textarea
            aria-label="CSS source"
            spellCheck={false}
            value={source}
            onChange={(event) => setSource(event.target.value)}
          />
        </label>
        <div className="omena-code-panel">
          <span className="flex items-center gap-2">
            <Braces aria-hidden="true" className="size-3.5" />
            Typed result
          </span>
          <pre aria-live="polite">{result}</pre>
        </div>
      </div>

      <div className="omena-playground-actions">
        <Button onClick={run} disabled={status === "loading"}>
          <Play aria-hidden="true" className="size-4 fill-current" />
          Run analysis
        </Button>
        <Button
          variant="ghost"
          onClick={() => {
            setSource(exampleSource);
            setResult("Example restored. Run the engine when you are ready.");
            setStatus("idle");
          }}
        >
          <RotateCcw aria-hidden="true" className="size-4" />
          Reset
        </Button>
        <p>Source remains in this tab. The playground makes no network analysis request.</p>
      </div>
    </section>
  );
}
