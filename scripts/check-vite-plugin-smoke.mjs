import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const { omenaCss } = require("../packages/vite-plugin/index.cjs");

const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "omena-vite-plugin-"));
const stylePath = path.join(tempRoot, "App.module.css");
const warnings = [];
const admissionCensusPath = path.join(
  process.cwd(),
  "packages/vite-plugin/virtual-source-admission-census.json",
);
const writeAdmissionCensus = process.argv.includes("--write-admission-census");

function createSmokeEngine() {
  return {
    buildSnapshotIdentity: (input) => {
      const targetSource = input.styleSources.find(
        ({ stylePath }) => stylePath === input.targetPath,
      )?.styleSource;
      return {
        schemaVersion: "0",
        product: "omena-query.build-snapshot-digest",
        contentHashAlgorithm: "blake3",
        digest: `blake3:smoke-${JSON.stringify(input)}`,
        targetSourceDigest: `blake3:smoke-source-${JSON.stringify(targetSource)}`,
      };
    },
    bundlerHostCapabilitiesJson: () =>
      JSON.stringify({
        protocolVersion: "0",
        capabilities: [
          "semanticClassExports",
          "typedExportNamespaces",
          "namedExports",
          "composesEdges",
        ],
      }),
    resolveCssModuleForBundlerHostJson: (requestJson) => {
      const request = JSON.parse(requestJson);
      return JSON.stringify({
        snapshotId: request.snapshotId,
        protocolVersion: "0",
        moduleId: request.stylePath,
        classExports: { root: "_root_0" },
        valueExports: {},
        namedExports: [{ exportedName: "root", kind: "class", value: "_root_0" }],
        typescriptDeclaration:
          "export declare const classExports: Readonly<Record<string, string>>;\nexport declare const valueExports: Readonly<Record<string, string>>;\ndeclare const styles: typeof classExports;\nexport default styles;\n",
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
  verifyAdmissionCensus(writeAdmissionCensus);
} finally {
  fs.rmSync(tempRoot, { recursive: true, force: true });
}

function verifyAdmissionCensus(write) {
  const census = measureAdmissionCensus();
  const serialized = `${JSON.stringify(census, null, 2)}\n`;
  if (write) {
    fs.writeFileSync(admissionCensusPath, serialized, "utf8");
    console.log(`wrote ${path.relative(process.cwd(), admissionCensusPath)}`);
    return;
  }
  const committed = fs.existsSync(admissionCensusPath)
    ? fs.readFileSync(admissionCensusPath, "utf8")
    : "";
  if (committed !== serialized) {
    const committedCensus = committed ? JSON.parse(committed) : null;
    throw new Error(
      `Vite virtual-source admission census drift: committed=${JSON.stringify(committedCensus?.totals ?? null)} measured=${JSON.stringify(census.totals)}. Run node scripts/check-vite-plugin-smoke.mjs --write-admission-census after reviewing the newly admitted inputs.`,
    );
  }
  console.log(
    `Vite virtual-source admission census: existing=${census.totals.existing} newlyAdmitted=${census.totals.newlyAdmitted} total=${census.totals.total}`,
  );
}

function measureAdmissionCensus() {
  const examplesCount = countStyleModules(path.join(process.cwd(), "examples"));
  const corpusCount = countStyleModules(
    path.join(process.cwd(), "test/_fixtures/real-project-corpus"),
  );
  const examplesConfig = fs.readFileSync(
    path.join(process.cwd(), "examples/vite.config.ts"),
    "utf8",
  );
  const viteUnit = fs.readFileSync(
    path.join(process.cwd(), "test/unit/vite-plugin/vite-plugin.test.ts"),
    "utf8",
  );
  const mappedVirtualCount =
    examplesConfig.match(/^\s{4}upstreamVirtualSource\(\),$/gmu)?.length ?? 0;
  const virtualOnlyCount =
    viteUnit.match(/it\("analyzes a virtual-only source when no disk mapping is available"/gu)
      ?.length ?? 0;
  const rows = [
    {
      key: "examples-disk-backed",
      provenanceClass: "disk-backed",
      admission: "existing",
      source: "examples/",
      section: "tracked module style inputs",
      count: examplesCount,
    },
    {
      key: "real-project-corpus-disk-backed",
      provenanceClass: "disk-backed",
      admission: "existing",
      source: "test/_fixtures/real-project-corpus/",
      section: "tracked module style inputs",
      count: corpusCount,
    },
    {
      key: "examples-upstream-transform",
      provenanceClass: "virtual-with-map",
      admission: "newly-admitted",
      source: "examples/vite.config.ts",
      section: "plugins: upstreamVirtualSource()",
      count: mappedVirtualCount,
    },
    {
      key: "virtual-only-regression",
      provenanceClass: "virtual-only",
      admission: "newly-admitted",
      source: "test/unit/vite-plugin/vite-plugin.test.ts",
      section: "analyzes a virtual-only source when no disk mapping is available",
      count: virtualOnlyCount,
    },
  ];
  const existing = rows
    .filter(({ admission }) => admission === "existing")
    .reduce((sum, { count }) => sum + count, 0);
  const newlyAdmitted = rows
    .filter(({ admission }) => admission === "newly-admitted")
    .reduce((sum, { count }) => sum + count, 0);
  return {
    schemaVersion: "0",
    product: "omena-vite.virtual-source-admission-census",
    package: "@omena/vite-plugin",
    semverIntent: "next-pre-1.0-minor",
    rows,
    totals: {
      existing,
      newlyAdmitted,
      total: existing + newlyAdmitted,
    },
  };
}

function countStyleModules(root) {
  let count = 0;
  for (const entry of fs.readdirSync(root, { withFileTypes: true })) {
    if (entry.name === "dist" || entry.name === "node_modules") continue;
    const entryPath = path.join(root, entry.name);
    if (entry.isDirectory()) {
      count += countStyleModules(entryPath);
    } else if (/\.module\.(?:css|less|scss)$/u.test(entry.name)) {
      count += 1;
    }
  }
  return count;
}
