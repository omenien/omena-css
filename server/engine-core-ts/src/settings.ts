import type { Connection } from "vscode-languageserver/node";
import type { ClassnameTransformMode } from "./core/scss/classname-transform";
import { isRecord } from "./core/util/value-guards";
import { parseBool, parseFiniteNumber } from "./core/util/value-utils";

export interface WindowSettings {
  readonly features: {
    readonly definition: boolean;
    readonly hover: boolean;
    readonly completion: boolean;
    readonly references: boolean;
    readonly rename: boolean;
  };
  readonly diagnostics: {
    readonly severity: "error" | "warning" | "information" | "hint";
    readonly unusedSelector: boolean;
    readonly missingModule: boolean;
  };
  readonly hover: {
    readonly maxCandidates: number;
  };
}

export interface ResourceSettings {
  readonly scss: {
    readonly classnameTransform: ClassnameTransformMode;
  };
  readonly pathAlias: Readonly<Record<string, string>>;
}

export type Settings = WindowSettings & ResourceSettings;

const DEFAULT_WINDOW_SETTINGS: WindowSettings = {
  features: { definition: true, hover: true, completion: true, references: true, rename: true },
  diagnostics: { severity: "warning", unusedSelector: true, missingModule: true },
  hover: { maxCandidates: 10 },
};

const DEFAULT_RESOURCE_SETTINGS: ResourceSettings = {
  scss: { classnameTransform: "asIs" },
  pathAlias: {},
};

const DEFAULTS: Settings = {
  ...DEFAULT_WINDOW_SETTINGS,
  ...DEFAULT_RESOURCE_SETTINGS,
};

const CLASSNAME_TRANSFORM_VALUES = [
  "asIs",
  "camelCase",
  "camelCaseOnly",
  "dashes",
  "dashesOnly",
] as const;

function isClassnameTransform(v: unknown): v is ClassnameTransformMode {
  return typeof v === "string" && (CLASSNAME_TRANSFORM_VALUES as readonly string[]).includes(v);
}

const SEVERITY_VALUES = ["error", "warning", "information", "hint"] as const;
type Severity = (typeof SEVERITY_VALUES)[number];

function isSeverity(v: unknown): v is Severity {
  return typeof v === "string" && (SEVERITY_VALUES as readonly string[]).includes(v);
}

export function parseWindowSettings(raw: unknown): WindowSettings {
  const r = isRecord(raw) ? raw : {};
  const features = isRecord(r.features) ? r.features : {};
  const diagnostics = isRecord(r.diagnostics) ? r.diagnostics : {};
  const hover = isRecord(r.hover) ? r.hover : {};
  return {
    features: {
      definition: parseBool(features.definition, DEFAULT_WINDOW_SETTINGS.features.definition),
      hover: parseBool(features.hover, DEFAULT_WINDOW_SETTINGS.features.hover),
      completion: parseBool(features.completion, DEFAULT_WINDOW_SETTINGS.features.completion),
      references: parseBool(features.references, DEFAULT_WINDOW_SETTINGS.features.references),
      rename: parseBool(features.rename, DEFAULT_WINDOW_SETTINGS.features.rename),
    },
    diagnostics: {
      severity: isSeverity(diagnostics.severity)
        ? diagnostics.severity
        : DEFAULT_WINDOW_SETTINGS.diagnostics.severity,
      unusedSelector: parseBool(
        diagnostics.unusedSelector,
        DEFAULT_WINDOW_SETTINGS.diagnostics.unusedSelector,
      ),
      missingModule: parseBool(
        diagnostics.missingModule,
        DEFAULT_WINDOW_SETTINGS.diagnostics.missingModule,
      ),
    },
    hover: {
      maxCandidates: parseFiniteNumber(
        hover.maxCandidates,
        DEFAULT_WINDOW_SETTINGS.hover.maxCandidates,
      ),
    },
  };
}

export function parseResourceSettings(raw: unknown): ResourceSettings {
  const r = isRecord(raw) ? raw : {};
  const scss = isRecord(r.scss) ? r.scss : {};
  return {
    scss: {
      classnameTransform: isClassnameTransform(scss.classnameTransform)
        ? scss.classnameTransform
        : DEFAULT_RESOURCE_SETTINGS.scss.classnameTransform,
    },
    pathAlias: parsePathAlias(r.pathAlias),
  };
}

export function mergeSettings(
  windowSettings: WindowSettings,
  resourceSettings: ResourceSettings,
): Settings {
  return {
    ...windowSettings,
    ...resourceSettings,
  };
}

export function resourceSettingsDependencyKey(settings: ResourceSettings): string {
  const pathAlias = Object.entries(settings.pathAlias)
    .toSorted(([a], [b]) => a.localeCompare(b))
    .map(([key, value]) => `${key}=${value}`)
    .join("|");
  return `transform:${settings.scss.classnameTransform};alias:${pathAlias}`;
}

/**
 * Parse a path alias record into `Record<string, string>`.
 * Non-record inputs fall back to `{}`; record values that are not strings
 * are dropped without coercion.
 */
export function parsePathAlias(v: unknown): Record<string, string> {
  if (!isRecord(v)) return {};
  const out: Record<string, string> = {};
  for (const [k, val] of Object.entries(v)) {
    if (typeof val === "string") out[k] = val;
  }
  return out;
}

function getConfigurationForSection(
  connection: Connection,
  section: string,
  scopeUri?: string,
): Promise<unknown> {
  return scopeUri
    ? connection.workspace.getConfiguration({ section, scopeUri })
    : connection.workspace.getConfiguration(section);
}

export async function fetchWindowSettings(connection: Connection): Promise<WindowSettings> {
  const raw = await getConfigurationForSection(connection, "omena");
  return parseWindowSettings(raw);
}

export async function fetchResourceSettings(
  connection: Connection,
  scopeUri?: string,
): Promise<ResourceSettings> {
  const raw = await getConfigurationForSection(connection, "omena", scopeUri);
  return parseResourceSettings(raw);
}

export async function fetchSettings(connection: Connection, scopeUri?: string): Promise<Settings> {
  const [windowSettings, resourceSettings] = await Promise.all([
    fetchWindowSettings(connection),
    fetchResourceSettings(connection, scopeUri),
  ]);
  return mergeSettings(windowSettings, resourceSettings);
}

export { DEFAULTS as DEFAULT_SETTINGS, DEFAULT_RESOURCE_SETTINGS, DEFAULT_WINDOW_SETTINGS };
