import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const { omenaCss } = require("../packages/vite-plugin/index.cjs");

const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "omena-vite-plugin-"));
const stylePath = path.join(tempRoot, "App.module.css");
const warnings = [];

function createSmokeEngine() {
  return {
    bundlerHostCapabilitiesJson: () =>
      JSON.stringify({
        protocolVersion: "0",
        capabilities: ["semanticClassMap", "namedExports", "composesEdges"],
      }),
    resolveCssModuleForBundlerHostJson: (requestJson) => {
      const request = JSON.parse(requestJson);
      return JSON.stringify({
        snapshotId: request.snapshotId,
        protocolVersion: "0",
        moduleId: request.stylePath,
        classMap: { root: "_root_0" },
        namedExports: { root: "_root_0" },
        typescriptDeclaration:
          "declare const styles: Readonly<Record<string, string>>;\nexport default styles;\n",
        composesEdges: [],
        diagnostics: [],
        ready: true,
      });
    },
    summarizeTransformBundleFromSourceJson: () =>
      JSON.stringify({ plannedPassIds: ["class-name-rewrite"] }),
    buildStyleSourcesWithContextJson: (targetPath, sourcesJson) => {
      const [source] = JSON.parse(sourcesJson);
      return JSON.stringify({
        execution: {
          outputCss: source.styleSource.replace(/\/\*[\s\S]*?\*\//gu, "").replace(/\s+/gu, " "),
          executedPassIds: ["comment-strip", "whitespace-strip", "class-name-rewrite"],
        },
        sourceMapV3: {
          version: 3,
          sources: [targetPath],
          names: [],
          mappings: "AAAA",
        },
      });
    },
  };
}

try {
  const pluginSource = fs.readFileSync(
    path.join(process.cwd(), "packages/vite-plugin/index.cjs"),
    "utf8",
  );
  if (pluginSource.includes("execFileSync") || pluginSource.includes("cargo run")) {
    throw new Error("Vite plugin hot path must not contain execFileSync/cargo run fallback.");
  }

  fs.writeFileSync(stylePath, ".root {\n  color: red;\n}\n/* remove me */\n", "utf8");
  const plugin = omenaCss({
    engine: createSmokeEngine(),
    passes: ["comment-strip", "whitespace-strip"],
    cwd: process.cwd(),
    configFile: false,
  });
  const input = fs.readFileSync(stylePath, "utf8");
  const result = await plugin.transform.call(
    { warn: (message) => warnings.push(message) },
    input,
    stylePath,
  );

  if (!result || typeof result.code !== "string") {
    throw new Error("Expected Vite plugin to return transformed CSS.");
  }
  if (result.code.includes("remove me")) {
    throw new Error(`Expected comment-strip pass to remove comments, got: ${result.code}`);
  }
  if (!result.code.includes(".root")) {
    throw new Error(`Expected transformed CSS to preserve selector, got: ${result.code}`);
  }
  if (!result.map || result.map.version !== 3) {
    throw new Error(`Expected Source Map V3 output, got: ${JSON.stringify(result.map)}`);
  }
  if (!Array.isArray(result.map.sources) || !result.map.sources.includes(stylePath)) {
    throw new Error(
      `Expected source map to include ${stylePath}, got: ${JSON.stringify(result.map)}`,
    );
  }
  if (typeof result.map.mappings !== "string") {
    throw new Error(`Expected source map mappings, got: ${JSON.stringify(result.map)}`);
  }
  if (warnings.length > 0) {
    throw new Error(`Unexpected Vite plugin warnings: ${warnings.join(" | ")}`);
  }
} finally {
  fs.rmSync(tempRoot, { recursive: true, force: true });
}
