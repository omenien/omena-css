import type { CheckScopeId } from "./types";

interface ScopeDefinition {
  readonly id: CheckScopeId;
  readonly matches: (scriptName: string) => boolean;
  readonly toGateId: (scriptName: string) => string;
}

const RUST_FAMILIES = [
  "checker-style-recovery",
  "checker-source-missing",
  "checker-style-unused",
  "expression-semantics",
  "expression-domain",
  "source-resolution",
  "selector-usage",
  "input-producers",
  "selected-query",
  "source-side",
  "type-fact",
  "query-plan",
  "checker",
  "parser",
  "omena-abstract-value",
  "omena-bridge",
  "omena-cascade",
  "omena-checker",
  "omena-css",
  "omena-incremental",
  "omena-interner",
  "omena-lsp-server",
  "omena-parser",
  "omena-query",
  "omena-resolver",
  "omena-syntax",
  "omena-testkit",
  "omena-tsgo-client",
  "omena-transform-bundle",
  "omena-transform-cst",
  "omena-transform-egg",
  "omena-transform-passes",
  "omena-transform-print",
  "omena-transform-target",
  "semantic",
  "release",
  "shadow",
  "split",
  "gate",
  "lane",
] as const;

export const SCOPE_DEFINITIONS: readonly ScopeDefinition[] = [
  {
    id: "rust",
    matches: (scriptName) =>
      scriptName.startsWith("check:rust-") ||
      scriptName.startsWith("update:rust-") ||
      scriptName.startsWith("benchmark:z5:"),
    toGateId: (scriptName) => {
      if (scriptName.startsWith("update:")) {
        return `rust/${toRustGatePath(scriptName.replace(/^update:rust-/, ""))}:update`;
      }
      if (scriptName.startsWith("benchmark:")) {
        return `rust/benchmark/${scriptName.slice("benchmark:".length).replaceAll(":", "/")}`;
      }
      return `rust/${toRustGatePath(scriptName.replace(/^check:rust-/, ""))}`;
    },
  },
  {
    id: "ts7",
    matches: (scriptName) => scriptName.startsWith("check:ts7-"),
    toGateId: (scriptName) => `ts7/${toTs7GatePath(scriptName.slice("check:ts7-".length))}`,
  },
  {
    id: "tsgo",
    matches: (scriptName) =>
      scriptName.startsWith("check:tsgo-") ||
      scriptName === "check:release-batch-tsgo" ||
      scriptName === "check:real-project-corpus-tsgo" ||
      scriptName === "check:lsp-server-smoke-tsgo",
    toGateId: (scriptName) =>
      `tsgo/${toTsgoGatePath(
        stripCheckPrefix(scriptName)
          .replace(/^tsgo-/, "")
          .replace(/-tsgo$/, ""),
      )}`,
  },
  {
    id: "plugin",
    matches: (scriptName) =>
      scriptName.includes("plugin-consumer") || scriptName.includes("plugin-smoke"),
    toGateId: (scriptName) => `plugin/${stripCheckPrefix(scriptName).replace(/^plugin-/, "")}`,
  },
  {
    id: "contract",
    matches: (scriptName) =>
      scriptName.includes("contract-parity") || scriptName.includes("type-fact-backend"),
    toGateId: (scriptName) => `contract/${stripCheckPrefix(scriptName).replace(/^contract-/, "")}`,
  },
  {
    id: "editor",
    matches: (scriptName) =>
      scriptName === "check:selected-query-boundary" ||
      scriptName === "check:editor-path-boundary" ||
      scriptName === "check:code-action-query-consumer" ||
      scriptName === "check:completion-query-consumer" ||
      scriptName === "check:rename-query-consumer" ||
      scriptName === "check:source-diagnostics-query-consumer" ||
      scriptName === "check:style-diagnostics-query-consumer" ||
      scriptName === "check:provider-host-routing-boundary" ||
      scriptName.startsWith("check:lsp-server-smoke") ||
      scriptName === "cme" ||
      scriptName.startsWith("explain:"),
    toGateId: (scriptName) => `editor/${toBackendQualifiedPath(stripCheckPrefix(scriptName))}`,
  },
  {
    id: "test",
    matches: (scriptName) => scriptName.startsWith("test:") || scriptName === "test",
    toGateId: (scriptName) => `test/${stripPrefix(scriptName, "test:")}`,
  },
  {
    id: "release",
    matches: (scriptName) =>
      scriptName.startsWith("release:") ||
      scriptName.startsWith("check:release-batch") ||
      scriptName.startsWith("check:real-project-corpus") ||
      scriptName === "check:packaged-engine-shadow-runner" ||
      scriptName === "check:packaged-engine-shadow-runner-matrix" ||
      scriptName === "check:packaged-tsgo-binary" ||
      scriptName === "check:packaged-selected-query-default" ||
      scriptName === "package" ||
      scriptName === "version-packages" ||
      scriptName === "changeset",
    toGateId: (scriptName) => `release/${scriptName.replace(":", "/")}`,
  },
  {
    id: "workspace",
    matches: (scriptName) =>
      scriptName === "check:workspace" ||
      scriptName === "check:semantic-smoke" ||
      scriptName === "check:backend-typecheck-smoke",
    toGateId: (scriptName) =>
      `workspace/${stripCheckPrefix(scriptName).replace(/^workspace$/, "check")}`,
  },
  {
    id: "core",
    matches: (scriptName) =>
      [
        "build",
        "build:engine-shadow-runner",
        "build:omena-lsp-server",
        "build:tsgo-binary",
        "check",
        "clean",
        "format",
        "format:check",
        "lint",
        "lint:fix",
        "typecheck",
        "watch",
      ].includes(scriptName),
    toGateId: (scriptName) => `core/${scriptName.replace(":", "/")}`,
  },
  {
    id: "tooling",
    matches: (scriptName) =>
      scriptName === "cme-check" ||
      scriptName === "check:orchestrator-doctor" ||
      scriptName === "check:orchestrator-inventory" ||
      scriptName === "update:check-inventory",
    toGateId: (scriptName) => `tooling/${stripCheckPrefix(scriptName).replace(":", "/")}`,
  },
];

export function classifyScript(scriptName: string): ScopeDefinition | null {
  return SCOPE_DEFINITIONS.find((scope) => scope.matches(scriptName)) ?? null;
}

function toRustGatePath(rest: string): string {
  const family = RUST_FAMILIES.find(
    (candidate) => rest === candidate || rest.startsWith(`${candidate}-`),
  );
  if (!family) return rest;
  if (rest === family) return family;
  return `${family}/${rest.slice(family.length + 1)}`;
}

function toTs7GatePath(rest: string): string {
  const backendMatch = /^(phase-[abc])-(.+)-tsgo$/.exec(rest);
  if (backendMatch) {
    const [, phase, name] = backendMatch;
    return `${phase}/${name}@tsgo`;
  }

  const phaseMatch = /^(phase-[abc])-(.+)$/.exec(rest);
  if (phaseMatch) {
    const [, phase, name] = phaseMatch;
    return `${phase}/${name}`;
  }

  return rest;
}

function toTsgoGatePath(rest: string): string {
  switch (rest) {
    case "release-bundle":
      return "release/bundle";
    case "operational-lane":
      return "operational/lane";
    case "operational-shadow-review":
      return "operational/shadow-review";
    default:
      return rest;
  }
}

function toBackendQualifiedPath(rest: string): string {
  const backendMatch = /^(.+)-tsgo$/.exec(rest);
  if (!backendMatch) return rest.replace(":", "/");
  const [, name] = backendMatch;
  return `${name}@tsgo`;
}

function stripCheckPrefix(scriptName: string): string {
  return stripPrefix(scriptName, "check:");
}

function stripPrefix(value: string, prefix: string): string {
  return value.startsWith(prefix) ? value.slice(prefix.length) : value;
}
