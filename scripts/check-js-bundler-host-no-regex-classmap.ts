import { strict as assert } from "node:assert";
import fs from "node:fs";
import path from "node:path";

const repoRoot = path.resolve(import.meta.dirname, "..");
const packageRoots = ["packages/css-build-adapter", "packages/vite-plugin"] as const;
const targets = packageRoots.flatMap((relativePath) =>
  packageSourceFiles(path.join(repoRoot, relativePath)).map((filePath) =>
    path.relative(repoRoot, filePath),
  ),
);
const sources = targets.map((relativePath) => ({
  relativePath,
  source: fs.readFileSync(path.join(repoRoot, relativePath), "utf8"),
}));
if (process.argv.includes("--inject-regex-classmap")) {
  sources.push({
    relativePath: "packages/css-build-adapter/injected-helper.cjs",
    source: "const classMap = Object.fromEntries(emittedCss.matchAll(/\\.([a-z-]+)/gu));",
  });
}

for (const { relativePath, source } of sources) {
  assert.ok(
    !source.includes("extractCssModuleClassMap"),
    `${relativePath} must not restore CSS-text class-map extraction`,
  );
  assert.ok(
    !source.includes(".matchAll("),
    `${relativePath} must not derive class maps by scanning emitted CSS`,
  );
}

const adapter = sources.find((entry) =>
  entry.relativePath.endsWith("css-build-adapter/index.cjs"),
)!.source;
const vite = sources.find((entry) => entry.relativePath.endsWith("vite-plugin/index.cjs"))!.source;
assert.ok(adapter.includes("resolveCssModuleInterface(engine"));
assert.ok(adapter.includes("classMap: moduleInterface.classMap"));
assert.ok(vite.includes("classMap: output.classMap"));

process.stdout.write(
  `${JSON.stringify({
    schemaVersion: "0",
    product: "js-bundler-host.no-regex-classmap",
    scannedFiles: targets.length,
    semanticTransportAnchors: 3,
  })}\n`,
);

function packageSourceFiles(root: string): string[] {
  const files: string[] = [];
  for (const entry of fs.readdirSync(root, { withFileTypes: true })) {
    if (entry.name === "node_modules" || entry.name === "dist") continue;
    const filePath = path.join(root, entry.name);
    if (entry.isDirectory()) {
      files.push(...packageSourceFiles(filePath));
    } else if (entry.isFile() && /\.(?:cjs|mjs|js|cts|mts|ts|tsx)$/u.test(entry.name)) {
      files.push(filePath);
    }
  }
  return files;
}
