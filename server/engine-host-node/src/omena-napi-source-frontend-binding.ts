import { createRequire } from "node:module";
import path from "node:path";

export interface OmenaNapiSourceFrontendBinding {
  readonly readSourceBindingIndexJson?: (
    sourcePath: string,
    source: string,
    sourceLanguage: string,
    importedStyleBindingsJson: string,
    classnamesBindBindingsJson: string,
  ) => string | null | undefined;
  readonly readSourceSyntaxIndexJson?: (
    sourcePath: string,
    source: string,
    sourceLanguage: string,
    importedStyleBindingsJson: string,
    classnamesBindBindingsJson: string,
  ) => string | null | undefined;
  readonly readSourceTypeFactControlFlowGraphJson?: (
    sourcePath: string,
    source: string,
    sourceLanguage: string,
    variableName: string,
    referenceByteOffset: number,
  ) => string | null | undefined;
}

const requireFromHostNode = createRequire(__filename);
const DEFAULT_OMENA_NAPI_BINDING_CANDIDATES = [
  "@omena/napi",
  path.resolve(process.cwd(), "rust/crates/omena-napi/pkg/index.js"),
  path.resolve(__dirname, "../../../rust/crates/omena-napi/pkg/index.js"),
] as const;
let cachedDefaultOmenaNapiBinding: OmenaNapiSourceFrontendBinding | null | undefined;

export function loadDefaultOmenaNapiSourceFrontendBinding(): OmenaNapiSourceFrontendBinding | null {
  if (cachedDefaultOmenaNapiBinding !== undefined) {
    return cachedDefaultOmenaNapiBinding;
  }

  for (const candidate of DEFAULT_OMENA_NAPI_BINDING_CANDIDATES) {
    try {
      const binding = bindingFromModule(requireFromHostNode(candidate) as unknown);
      if (binding) {
        cachedDefaultOmenaNapiBinding = binding;
        return binding;
      }
    } catch {
      // Optional local/package binding. Absence keeps callers on their explicit fallback path.
    }
  }

  cachedDefaultOmenaNapiBinding = null;
  return cachedDefaultOmenaNapiBinding;
}

function bindingFromModule(value: unknown): OmenaNapiSourceFrontendBinding | null {
  if (isOmenaNapiSourceFrontendBinding(value)) return value;
  if (!value || typeof value !== "object") return null;
  const maybeDefault = (value as { readonly default?: unknown }).default;
  return isOmenaNapiSourceFrontendBinding(maybeDefault) ? maybeDefault : null;
}

function isOmenaNapiSourceFrontendBinding(value: unknown): value is OmenaNapiSourceFrontendBinding {
  if (!value || typeof value !== "object") return false;
  const binding = value as OmenaNapiSourceFrontendBinding;
  return (
    typeof binding.readSourceBindingIndexJson === "function" ||
    typeof binding.readSourceSyntaxIndexJson === "function" ||
    typeof binding.readSourceTypeFactControlFlowGraphJson === "function"
  );
}
