"use client";

import { Accordion, Accordions } from "fumadocs-ui/components/accordion";
import { Callout } from "fumadocs-ui/components/callout";
import {
  CodeBlock,
  CodeBlockTab,
  CodeBlockTabs,
  CodeBlockTabsList,
  CodeBlockTabsTrigger,
  Pre,
} from "fumadocs-ui/components/codeblock";
import { TabsContent, TabsList, TabsTrigger } from "fumadocs-ui/components/tabs";
import { buttonVariants } from "fumadocs-ui/components/ui/button";
import { Tabs as TabsRoot } from "fumadocs-ui/components/ui/tabs";
import {
  Check,
  FileCode2,
  LoaderCircle,
  Play,
  RotateCcw,
  ShieldCheck,
  TriangleAlert,
} from "lucide-react";
import type { ReactNode } from "react";
import { useEffect, useRef, useState } from "react";
import { withBasePath } from "@/lib/site";

export type WasmPlaygroundScenario = "workspace-diagnostics" | "workspace-build" | "target-build";

interface StyleSourceInput {
  stylePath: string;
  styleSource: string;
}

interface OmenaBrowserModule {
  default: () => Promise<unknown>;
  buildStyleSourceForTargetQuery: (source: string, path: string, targetQuery: string) => unknown;
  buildStyleSourcesWithContext: (
    targetPath: string,
    sources: StyleSourceInput[],
    passIds: string[],
    context: Record<string, never>,
    packageManifests: unknown[],
  ) => unknown;
  readWorkspaceStyleDiagnostics: (
    targetPath: string,
    sources: StyleSourceInput[],
    sourceDocuments: unknown[],
    packageManifests: unknown[],
    externalSifs: unknown[],
    externalMode: string | null,
  ) => unknown;
}

interface DemoFile {
  label: string;
  path: string;
  source: string;
}

interface ScenarioDefinition {
  files: DemoFile[];
  label: string;
  primaryPath: string;
  resultKind: "diagnostics" | "build" | "target";
}

interface SourceRange {
  start?: {
    line?: number;
    character?: number;
  };
  end?: {
    line?: number;
    character?: number;
  };
}

interface DiagnosticSummary {
  code: string;
  message: string;
  provenance: string[];
  range?: SourceRange;
}

interface DiagnosticsResult {
  diagnostics: DiagnosticSummary[];
  fileUri: string;
  originalCount: number;
  suppressedCount: number;
}

interface PassSummary {
  detail: string;
  mutationCount: number;
  passId: string;
  provenancePreserved: boolean;
  status: string;
}

interface BuildSummary {
  appliedPasses: PassSummary[];
  mutationCount: number;
  outputCss: string;
  provenancePreserved: boolean;
  resolvedTargets: string[];
  sourceMapSources: string[];
  targetDataSource?: string;
}

interface PlaygroundResult {
  fingerprint: string;
  raw: unknown;
}

type SourceByPath = Record<string, string>;
type PlaygroundStatus = "idle" | "loading" | "ready" | "error";

const scenarios: Record<WasmPlaygroundScenario, ScenarioDefinition> = {
  "workspace-diagnostics": {
    label: "CSS Modules workspace diagnostics",
    primaryPath: "/workspace/src/App.module.css",
    resultKind: "diagnostics",
    files: [
      {
        label: "App.module.css",
        path: "/workspace/src/App.module.css",
        source: `.button {
  composes: missing from "./Base.module.css";
}

@value absent from "./Tokens.module.css";`,
      },
      {
        label: "Base.module.css",
        path: "/workspace/src/Base.module.css",
        source: `.base {
  color: royalblue;
}`,
      },
      {
        label: "Tokens.module.css",
        path: "/workspace/src/Tokens.module.css",
        source: `@value accent: royalblue;`,
      },
    ],
  },
  "workspace-build": {
    label: "Workspace-aware CSS Modules build",
    primaryPath: "Button.module.css",
    resultKind: "build",
    files: [
      {
        label: "Button.module.css",
        path: "Button.module.css",
        source: `@import "./tokens.css";

.button {
  composes: base;
  color: var(--brand);
}

.base {
  padding: 0.5rem 1rem;
}`,
      },
      {
        label: "tokens.css",
        path: "tokens.css",
        source: `:root {
  --brand: oklch(61% 0.18 29);
}`,
      },
    ],
  },
  "target-build": {
    label: "Target-aware CSS build",
    primaryPath: "Card.module.css",
    resultKind: "target",
    files: [
      {
        label: "Card.module.css",
        path: "Card.module.css",
        source: `.card {
  display: flex;
  color: light-dark(#111, #eee);
}`,
      },
    ],
  },
};

let runtimePromise: Promise<OmenaBrowserModule> | undefined;
const importBrowserModule = new Function("url", "return import(url)") as (
  url: string,
) => Promise<unknown>;

async function loadRuntime(): Promise<OmenaBrowserModule> {
  runtimePromise ??= importBrowserModule(withBasePath("/wasm/omena_wasm.js")).then(
    async (module: unknown) => {
      const runtime = module as OmenaBrowserModule;
      await runtime.default();
      return runtime;
    },
  );
  return runtimePromise;
}

function initialSources(definition: ScenarioDefinition): SourceByPath {
  return Object.fromEntries(definition.files.map((file) => [file.path, file.source]));
}

function orderedStyleSources(
  definition: ScenarioDefinition,
  sources: SourceByPath,
): StyleSourceInput[] {
  return definition.files.map((file) => ({
    stylePath: file.path,
    styleSource: sources[file.path] ?? "",
  }));
}

function sourceFingerprint(definition: ScenarioDefinition, sources: SourceByPath) {
  return JSON.stringify(definition.files.map((file) => [file.path, sources[file.path] ?? ""]));
}

async function runScenario(
  scenario: WasmPlaygroundScenario,
  definition: ScenarioDefinition,
  sources: SourceByPath,
) {
  const runtime = await loadRuntime();
  const styleSources = orderedStyleSources(definition, sources);

  if (scenario === "workspace-diagnostics") {
    return runtime.readWorkspaceStyleDiagnostics(
      definition.primaryPath,
      styleSources,
      [],
      [],
      [],
      null,
    );
  }

  if (scenario === "workspace-build") {
    return runtime.buildStyleSourcesWithContext(
      definition.primaryPath,
      styleSources,
      ["import-inline", "composes-resolution"],
      {},
      [],
    );
  }

  return runtime.buildStyleSourceForTargetQuery(
    sources[definition.primaryPath] ?? "",
    definition.primaryPath,
    "ie 11",
  );
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function asNumber(value: unknown, fallback = 0) {
  return typeof value === "number" ? value : fallback;
}

function asString(value: unknown, fallback = "") {
  return typeof value === "string" ? value : fallback;
}

function asStringArray(value: unknown) {
  return Array.isArray(value)
    ? value.filter((entry): entry is string => typeof entry === "string")
    : [];
}

function summarizeDiagnostics(raw: unknown): DiagnosticsResult {
  if (!isRecord(raw)) {
    return {
      diagnostics: [],
      fileUri: "",
      originalCount: 0,
      suppressedCount: 0,
    };
  }

  const diagnostics = Array.isArray(raw.diagnostics)
    ? raw.diagnostics.flatMap((entry) => {
        if (!isRecord(entry)) return [];
        const range = isRecord(entry.range) ? (entry.range as SourceRange) : undefined;
        return [
          {
            code: asString(entry.code, "diagnostic"),
            message: asString(entry.message, "Omena reported a diagnostic."),
            provenance: asStringArray(entry.provenance),
            range,
          },
        ];
      })
    : [];
  const suppression = isRecord(raw.suppressionSummary) ? raw.suppressionSummary : {};

  return {
    diagnostics,
    fileUri: asString(raw.fileUri),
    originalCount: asNumber(
      suppression.originalDiagnosticCount,
      asNumber(raw.diagnosticCount, diagnostics.length),
    ),
    suppressedCount: asNumber(suppression.suppressedDiagnosticCount),
  };
}

function summarizeBuild(raw: unknown): BuildSummary {
  const root = isRecord(raw) ? raw : {};
  const execution = isRecord(root.execution) ? root.execution : {};
  const sourceMap = isRecord(root.sourceMapV3) ? root.sourceMapV3 : {};
  const targetQuery = isRecord(root.targetQuery) ? root.targetQuery : {};
  const outcomes = Array.isArray(execution.outcomes)
    ? execution.outcomes.flatMap((entry) => {
        if (!isRecord(entry)) return [];
        return [
          {
            detail: asString(entry.detail),
            mutationCount: asNumber(entry.mutationCount),
            passId: asString(entry.passId, "unknown-pass"),
            provenancePreserved: entry.provenancePreserved === true,
            status: asString(entry.status),
          },
        ];
      })
    : [];

  return {
    appliedPasses: outcomes.filter((outcome) => outcome.status === "applied"),
    mutationCount: asNumber(execution.mutationCount),
    outputCss: asString(execution.outputCss),
    provenancePreserved: execution.provenancePreserved === true,
    resolvedTargets: asStringArray(targetQuery.resolvedTargets),
    sourceMapSources: asStringArray(sourceMap.sources),
    targetDataSource:
      typeof targetQuery.targetDataSource === "string" ? targetQuery.targetDataSource : undefined,
  };
}

function locationLabel(range: SourceRange | undefined) {
  const line = range?.start?.line;
  const character = range?.start?.character;
  if (typeof line !== "number") {
    return "Source location unavailable";
  }
  return `line ${line + 1}${typeof character === "number" ? `, column ${character + 1}` : ""}`;
}

function provenanceLabel(value: string) {
  if (value.includes("parser")) return "parser facts";
  if (value.includes("resolution-diagnostics")) return "workspace resolution";
  if (value.includes("style-diagnostics")) return "semantic query";
  if (value.includes("product-diagnostic-gate")) return "product gate";
  if (value.includes("rule-registry")) return "rule registry";
  return value.split(".").at(-1)?.replaceAll("-", " ") ?? value;
}

function sourceOffset(source: string, position: { line?: number; character?: number } | undefined) {
  const line = position?.line;
  const character = position?.character;
  if (typeof line !== "number" || typeof character !== "number") {
    return undefined;
  }

  const lines = source.split("\n");
  if (line >= lines.length) {
    return undefined;
  }
  return lines.slice(0, line).reduce((offset, value) => offset + value.length + 1, 0) + character;
}

function sourceHighlights(source: string, ranges: SourceRange[]) {
  const highlights = ranges
    .flatMap((range) => {
      const start = sourceOffset(source, range.start);
      const end = sourceOffset(source, range.end);
      if (start === undefined || end === undefined || start >= end || start >= source.length) {
        return [];
      }
      return [{ start, end: Math.min(end, source.length) }];
    })
    .reduce<{ start: number; end: number }[]>((ordered, highlight) => {
      const insertionIndex = ordered.findIndex((candidate) => highlight.start < candidate.start);
      if (insertionIndex === -1) {
        ordered.push(highlight);
      } else {
        ordered.splice(insertionIndex, 0, highlight);
      }
      return ordered;
    }, []);

  return highlights.reduce<{ start: number; end: number }[]>((merged, highlight) => {
    const previous = merged.at(-1);
    if (previous && highlight.start <= previous.end) {
      previous.end = Math.max(previous.end, highlight.end);
    } else {
      merged.push({ ...highlight });
    }
    return merged;
  }, []);
}

function HighlightedSource({ ranges, source }: { ranges: SourceRange[]; source: string }) {
  const highlights = sourceHighlights(source, ranges);
  if (highlights.length === 0) {
    return source;
  }

  const parts: ReactNode[] = [];
  let cursor = 0;
  for (const [index, highlight] of highlights.entries()) {
    parts.push(source.slice(cursor, highlight.start));
    parts.push(
      <span
        key={`${highlight.start}-${highlight.end}-${index}`}
        className="[text-decoration-color:var(--color-fd-warning)] [text-decoration-line:underline] [text-decoration-style:wavy] [text-decoration-thickness:1.5px] [text-underline-offset:3px]"
      >
        {source.slice(highlight.start, highlight.end)}
      </span>,
    );
    cursor = highlight.end;
  }
  parts.push(source.slice(cursor));
  return parts;
}

function SourceEditor({
  label,
  highlightRanges,
  onChange,
  source,
}: {
  label: string;
  highlightRanges: SourceRange[];
  onChange: (value: string) => void;
  source: string;
}) {
  const previewRef = useRef<HTMLPreElement>(null);

  return (
    <div className="relative min-h-60">
      <pre
        ref={previewRef}
        aria-hidden="true"
        className="pointer-events-none absolute inset-0 overflow-hidden whitespace-pre px-4 py-3 font-mono text-[0.8125rem] leading-6 text-fd-foreground"
      >
        <HighlightedSource ranges={highlightRanges} source={source} />
      </pre>
      <textarea
        className="relative block min-h-60 w-full resize-y overflow-auto whitespace-pre bg-transparent px-4 py-3 font-mono text-[0.8125rem] leading-6 text-transparent caret-fd-foreground outline-none selection:bg-fd-primary/20 focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-fd-ring"
        aria-label={`${label} source`}
        spellCheck={false}
        wrap="off"
        value={source}
        onChange={(event) => onChange(event.target.value)}
        onScroll={(event) => {
          if (!previewRef.current) return;
          previewRef.current.scrollTop = event.currentTarget.scrollTop;
          previewRef.current.scrollLeft = event.currentTarget.scrollLeft;
        }}
      />
    </div>
  );
}

function DiagnosticsPanel({ raw }: { raw: unknown }) {
  const summary = summarizeDiagnostics(raw);

  if (summary.diagnostics.length === 0) {
    return (
      <div className="flex items-start gap-2 text-sm">
        <Check aria-hidden="true" className="mt-0.5 size-4 text-fd-success" />
        <div>
          <p className="font-medium">No problems found</p>
          <p className="mt-1 text-fd-muted-foreground">
            The in-memory workspace passed the available browser checks.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div>
      <div className="divide-y divide-fd-border">
        {summary.diagnostics.map((diagnostic, index) => (
          <div
            key={`${diagnostic.code}-${index}`}
            className="flex items-start gap-2.5 py-2 first:pt-0"
          >
            <TriangleAlert aria-hidden="true" className="mt-0.5 size-4 shrink-0 text-fd-warning" />
            <div className="min-w-0">
              <p className="text-sm text-fd-foreground">{diagnostic.message}</p>
              <p className="mt-1 text-xs text-fd-muted-foreground">
                {summary.fileUri.split("/").at(-1)} · {locationLabel(diagnostic.range)} ·{" "}
                <code>{diagnostic.code}</code>
              </p>
              {diagnostic.provenance.length > 0 ? (
                <p className="mt-2 text-xs text-fd-muted-foreground">
                  Analysis path: {diagnostic.provenance.map(provenanceLabel).join(" → ")}
                </p>
              ) : null}
            </div>
          </div>
        ))}
      </div>
      <p className="mt-3 border-t border-fd-border pt-3 text-xs text-fd-muted-foreground">
        {summary.originalCount} checked · {summary.suppressedCount} suppressed
      </p>
    </div>
  );
}

function BuildOutputPanel({ raw }: { raw: unknown }) {
  const summary = summarizeBuild(raw);

  return (
    <div>
      <p className="text-sm text-fd-muted-foreground">
        {summary.appliedPasses.length} passes changed the source across {summary.mutationCount}{" "}
        mutations
        {summary.provenancePreserved ? "; source provenance was preserved." : "."}
      </p>
      <CodeBlock title="output.css" allowCopy className="mt-3 mb-0">
        <Pre className="px-4">
          <code>{summary.outputCss}</code>
        </Pre>
      </CodeBlock>
      {summary.sourceMapSources.length > 0 ? (
        <p className="mt-3 text-xs text-fd-muted-foreground">
          Source map origins: {summary.sourceMapSources.join(", ")}
        </p>
      ) : null}
    </div>
  );
}

function PassesPanel({ raw, showTarget }: { raw: unknown; showTarget: boolean }) {
  const summary = summarizeBuild(raw);

  return (
    <div>
      {showTarget ? (
        <div className="mb-4 text-sm">
          <p>
            Resolved target: <code>{summary.resolvedTargets.join(", ") || "unresolved"}</code>
          </p>
          {summary.targetDataSource ? (
            <p className="mt-1 text-xs text-fd-muted-foreground">
              Target data: {summary.targetDataSource}
            </p>
          ) : null}
        </div>
      ) : null}
      <div className="divide-y divide-fd-border border-y border-fd-border">
        {summary.appliedPasses.map((pass) => (
          <div key={pass.passId} className="py-3">
            <div className="flex flex-wrap items-center justify-between gap-2">
              <code className="text-sm font-medium text-fd-foreground">{pass.passId}</code>
              <span className="text-xs text-fd-muted-foreground">
                {pass.mutationCount} {pass.mutationCount === 1 ? "mutation" : "mutations"}
              </span>
            </div>
            <p className="mt-1 text-xs text-fd-muted-foreground">{pass.detail}</p>
          </div>
        ))}
      </div>
    </div>
  );
}

function ResultState({
  children,
  errorMessage,
  hasResult,
  stale,
  status,
}: {
  children: ReactNode;
  errorMessage: string;
  hasResult: boolean;
  stale: boolean;
  status: PlaygroundStatus;
}) {
  if (status === "loading") {
    return (
      <p className="flex items-center gap-2 text-sm text-fd-muted-foreground">
        <LoaderCircle aria-hidden="true" className="size-4 animate-spin" />
        Running the browser engine
      </p>
    );
  }

  if (status === "error") {
    return (
      <Callout
        role="alert"
        type="error"
        title="The browser engine could not start"
        className="my-0"
      >
        {errorMessage}
      </Callout>
    );
  }

  if (stale) {
    return (
      <p className="text-sm text-fd-muted-foreground">
        Source changed. Analyze to refresh this panel.
      </p>
    );
  }

  if (!hasResult) {
    return <p className="text-sm text-fd-muted-foreground">No result is available yet.</p>;
  }

  return children;
}

function ResultTabs({
  definition,
  errorMessage,
  result,
  stale,
  status,
}: {
  definition: ScenarioDefinition;
  errorMessage: string;
  result: PlaygroundResult | undefined;
  stale: boolean;
  status: PlaygroundStatus;
}) {
  const hasResult = result !== undefined;
  const raw = result?.raw;
  const resultState = (children: ReactNode) => (
    <ResultState errorMessage={errorMessage} hasResult={hasResult} stale={stale} status={status}>
      {children}
    </ResultState>
  );

  if (definition.resultKind === "diagnostics") {
    const headingId = `problems-${definition.primaryPath.replaceAll(/[^a-z0-9]+/giu, "-")}`;
    return (
      <section
        aria-labelledby={headingId}
        className="my-4 overflow-hidden rounded-xl border bg-fd-secondary"
      >
        <h3 id={headingId} className="border-b border-fd-border px-4 py-2 text-sm font-medium">
          Problems
        </h3>
        <div className="p-4">{resultState(<DiagnosticsPanel raw={raw} />)}</div>
      </section>
    );
  }

  return (
    <TabsRoot
      defaultValue="output"
      className="my-4 flex flex-col overflow-hidden rounded-xl border bg-fd-secondary"
    >
      <TabsList aria-label={`${definition.label} results`}>
        <TabsTrigger value="output">Output</TabsTrigger>
        <TabsTrigger value="passes">
          {definition.resultKind === "target" ? "Target plan" : "Passes"}
        </TabsTrigger>
      </TabsList>
      <TabsContent value="output">{resultState(<BuildOutputPanel raw={raw} />)}</TabsContent>
      <TabsContent value="passes">
        {resultState(<PassesPanel raw={raw} showTarget={definition.resultKind === "target"} />)}
      </TabsContent>
    </TabsRoot>
  );
}

function RawOutput({ raw }: { raw: unknown }) {
  return (
    <Accordions type="single" className="mt-4 shadow-none">
      <Accordion title="Raw engine output">
        <CodeBlock allowCopy className="my-2">
          <Pre className="px-4">
            <code>{JSON.stringify(raw, null, 2)}</code>
          </Pre>
        </CodeBlock>
      </Accordion>
    </Accordions>
  );
}

function EditorBlock({
  embedded,
  file,
  highlightRanges,
  onAnalyze,
  onChange,
  onReset,
  source,
  stale,
  status,
}: {
  embedded: boolean;
  file: DemoFile;
  highlightRanges: SourceRange[];
  onAnalyze: () => void;
  onChange: (value: string) => void;
  onReset: () => void;
  source: string;
  stale: boolean;
  status: PlaygroundStatus;
}) {
  return (
    <CodeBlock
      title={embedded ? undefined : file.label}
      icon={embedded ? undefined : <FileCode2 aria-hidden="true" />}
      allowCopy={false}
      className={embedded ? "my-0" : "my-4"}
      viewportProps={{ className: "p-0!" }}
      Actions={
        embedded
          ? () => null
          : ({ className }) => (
              <EditorActions
                className={className}
                onAnalyze={onAnalyze}
                onReset={onReset}
                stale={stale}
                status={status}
              />
            )
      }
    >
      <SourceEditor
        label={file.label}
        highlightRanges={highlightRanges}
        source={source}
        onChange={onChange}
      />
    </CodeBlock>
  );
}

function EditorActions({
  className,
  onAnalyze,
  onReset,
  stale,
  status,
}: {
  className?: string;
  onAnalyze: () => void;
  onReset: () => void;
  stale: boolean;
  status: PlaygroundStatus;
}) {
  return (
    <div className={`${className ?? ""} flex shrink-0 items-center gap-1`}>
      <button
        type="button"
        className={buttonVariants({ variant: "ghost", size: "sm" })}
        onClick={onAnalyze}
        disabled={status === "loading"}
      >
        {status === "loading" ? (
          <LoaderCircle aria-hidden="true" className="size-3.5 animate-spin" />
        ) : (
          <Play aria-hidden="true" className="size-3 fill-current" />
        )}
        <span className="hidden sm:inline">{stale ? "Analyze changes" : "Analyze"}</span>
        <span className="sm:hidden">Run</span>
      </button>
      <button
        type="button"
        aria-label="Reset example"
        className={buttonVariants({ variant: "ghost", size: "icon-sm" })}
        onClick={onReset}
        disabled={status === "loading"}
      >
        <RotateCcw aria-hidden="true" className="size-3.5" />
      </button>
    </div>
  );
}

function DemoEditor({
  activePath,
  definition,
  highlightRanges,
  onActivePathChange,
  onAnalyze,
  onChange,
  onReset,
  sources,
  stale,
  status,
}: {
  activePath: string;
  definition: ScenarioDefinition;
  highlightRanges: SourceRange[];
  onActivePathChange: (path: string) => void;
  onAnalyze: () => void;
  onChange: (path: string, value: string) => void;
  onReset: () => void;
  sources: SourceByPath;
  stale: boolean;
  status: PlaygroundStatus;
}) {
  if (definition.files.length === 1) {
    const file = definition.files[0];
    return (
      <EditorBlock
        embedded={false}
        file={file}
        highlightRanges={highlightRanges}
        onAnalyze={onAnalyze}
        onChange={(value) => onChange(file.path, value)}
        onReset={onReset}
        source={sources[file.path] ?? ""}
        stale={stale}
        status={status}
      />
    );
  }

  return (
    <CodeBlockTabs
      value={activePath}
      onValueChange={onActivePathChange}
      className="overflow-hidden"
    >
      <div className="flex min-w-0 items-center">
        <CodeBlockTabsList aria-label={`${definition.label} files`} className="min-w-0 flex-1">
          {definition.files.map((file) => (
            <CodeBlockTabsTrigger key={file.path} value={file.path}>
              <FileCode2 aria-hidden="true" />
              {file.label}
            </CodeBlockTabsTrigger>
          ))}
        </CodeBlockTabsList>
        <EditorActions
          className="px-2"
          onAnalyze={onAnalyze}
          onReset={onReset}
          stale={stale}
          status={status}
        />
      </div>
      {definition.files.map((file) => (
        <CodeBlockTab key={file.path} value={file.path} className="p-0!">
          <EditorBlock
            embedded
            file={file}
            highlightRanges={file.path === activePath ? highlightRanges : []}
            onAnalyze={onAnalyze}
            onChange={(value) => onChange(file.path, value)}
            onReset={onReset}
            source={sources[file.path] ?? ""}
            stale={stale}
            status={status}
          />
        </CodeBlockTab>
      ))}
    </CodeBlockTabs>
  );
}

export function WasmPlayground({
  scenario = "workspace-diagnostics",
}: {
  scenario?: WasmPlaygroundScenario;
}) {
  const definition = scenarios[scenario];
  const [activePath, setActivePath] = useState(definition.primaryPath);
  const [sources, setSources] = useState<SourceByPath>(() => initialSources(definition));
  const [result, setResult] = useState<PlaygroundResult>();
  const [status, setStatus] = useState<PlaygroundStatus>("idle");
  const [errorMessage, setErrorMessage] = useState("");

  const fingerprint = sourceFingerprint(definition, sources);
  const stale = result !== undefined && result.fingerprint !== fingerprint;

  async function run(nextSources = sources) {
    setStatus("loading");
    setErrorMessage("");
    try {
      const raw = await runScenario(scenario, definition, nextSources);
      setResult({
        fingerprint: sourceFingerprint(definition, nextSources),
        raw,
      });
      setStatus("ready");
    } catch (error) {
      setErrorMessage(error instanceof Error ? error.message : String(error));
      setStatus("error");
    }
  }

  useEffect(() => {
    const nextSources = initialSources(definition);
    let active = true;
    setActivePath(definition.primaryPath);
    setSources(nextSources);
    setResult(undefined);
    setStatus("loading");
    setErrorMessage("");
    void runScenario(scenario, definition, nextSources)
      .then((raw) => {
        if (!active) return;
        setResult({
          fingerprint: sourceFingerprint(definition, nextSources),
          raw,
        });
        setStatus("ready");
      })
      .catch((error: unknown) => {
        if (!active) return;
        setErrorMessage(error instanceof Error ? error.message : String(error));
        setStatus("error");
      });
    return () => {
      active = false;
    };
  }, [definition, scenario]);

  const diagnosticSummary =
    definition.resultKind === "diagnostics" && result !== undefined && status === "ready" && !stale
      ? summarizeDiagnostics(result.raw)
      : undefined;
  const highlightRanges =
    diagnosticSummary?.fileUri === activePath
      ? diagnosticSummary.diagnostics.flatMap((diagnostic) =>
          diagnostic.range ? [diagnostic.range] : [],
        )
      : [];
  const currentRaw = result !== undefined && status === "ready" && !stale ? result.raw : undefined;
  const statusMessage =
    status === "loading"
      ? `${definition.label} is running.`
      : status === "error"
        ? `${definition.label} failed.`
        : stale
          ? `${definition.label} source changed. Run the analysis to refresh the result.`
          : status === "ready"
            ? `${definition.label} completed.`
            : `${definition.label} is ready.`;
  const headingId = `playground-${scenario}`;

  function reset() {
    const nextSources = initialSources(definition);
    setSources(nextSources);
    setActivePath(definition.primaryPath);
    void run(nextSources);
  }

  return (
    <section
      className="not-prose my-6"
      aria-busy={status === "loading"}
      aria-labelledby={headingId}
    >
      <h3 id={headingId} className="sr-only">
        {definition.label}
      </h3>
      <span role="status" aria-live="polite" aria-atomic="true" className="sr-only">
        {statusMessage}
      </span>
      <DemoEditor
        activePath={activePath}
        definition={definition}
        highlightRanges={highlightRanges}
        onActivePathChange={setActivePath}
        onAnalyze={() => void run()}
        onChange={(path, value) => setSources((current) => ({ ...current, [path]: value }))}
        onReset={reset}
        sources={sources}
        stale={stale}
        status={status}
      />

      <ResultTabs
        definition={definition}
        errorMessage={errorMessage}
        result={result}
        stale={stale}
        status={status}
      />

      {currentRaw !== undefined ? <RawOutput raw={currentRaw} /> : null}

      <p className="mt-3 flex items-center gap-2 text-xs text-fd-muted-foreground">
        <ShieldCheck aria-hidden="true" className="size-3.5 shrink-0" />
        Runs locally in your browser. No server analysis; your source stays in this tab.
      </p>
    </section>
  );
}
