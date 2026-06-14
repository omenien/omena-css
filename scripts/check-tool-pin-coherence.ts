import { findToolPinCoherenceDiagnostics } from "../packages/check-orchestrator/src/manifest/tool-pins";

const diagnostics = findToolPinCoherenceDiagnostics(process.cwd());

for (const diagnostic of diagnostics) {
  const prefix = diagnostic.severity === "error" ? "error" : "warning";
  console.log(`${prefix}: ${diagnostic.code}: ${diagnostic.message}`);
}

const errorCount = diagnostics.filter((diagnostic) => diagnostic.severity === "error").length;
if (errorCount > 0) {
  process.exit(1);
}
