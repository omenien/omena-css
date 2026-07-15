import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";

type StyleSource = { readonly stylePath: string; readonly styleSource: string };
type ParityCase = {
  readonly id: string;
  readonly targetPath: string;
  readonly sources: readonly StyleSource[];
  readonly emittedCss: string;
  readonly legacyRegexMap: Readonly<Record<string, string>>;
  readonly expectedClassMap: Readonly<Record<string, string>>;
};
type InterfaceModule = {
  readonly stylePath: string;
  readonly classExports: readonly {
    readonly name: string;
    readonly namedExport?: string;
    readonly emittedClasses: readonly string[];
  }[];
  readonly icssExports: readonly {
    readonly name: string;
    readonly namedExport?: string;
    readonly value: string;
  }[];
};

const repoRoot = path.resolve(import.meta.dirname, "..");
const rustRoot = path.join(repoRoot, "rust");
const fixturePath = path.join(repoRoot, "scripts/fixtures/css-module-host-parity/cases.json");
const require = createRequire(import.meta.url);
const { createOmenaBuildState, rebuildAndCache } =
  require("../packages/css-build-adapter/index.cjs") as {
    createOmenaBuildState(options: Record<string, unknown>): unknown;
    rebuildAndCache(
      filePath: string,
      source: string,
      options: Record<string, unknown>,
      state: unknown,
    ): Promise<{
      readonly classMap: Readonly<Record<string, string>>;
      readonly typescriptDeclaration: string;
    }>;
  };

const injectDroppedComposesEdge = process.argv.includes("--inject-dropped-composes-edge");
const injectRenamedDevClass = process.argv.includes("--inject-renamed-dev-class");
const fixtures = (JSON.parse(fs.readFileSync(fixturePath, "utf8")) as readonly ParityCase[]).map(
  applyFixtureFault,
);
const cliPath = path.join(rustRoot, "target/debug/omena");
const boundaryRunnerPath = path.join(rustRoot, "target/debug/engine-shadow-runner");

main().catch((error: unknown) => {
  console.error(error);
  process.exitCode = 1;
});

async function main() {
  assert.ok(fixtures.length >= 6, "bundler host parity requires at least six fixtures");
  assert.deepEqual(
    new Set(fixtures.map((fixture) => path.extname(fixture.targetPath))),
    new Set([".css", ".scss", ".less"]),
  );
  run("cargo", [
    "build",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "omena-cli",
    "-p",
    "engine-shadow-runner",
  ]);

  const outcomes = [];
  for (const fixture of fixtures) {
    outcomes.push(await verifyFixture(fixture));
  }

  process.stdout.write(
    `${JSON.stringify(
      {
        schemaVersion: "0",
        product: "js-bundler-host.parity",
        fixtureCount: outcomes.length,
        outcomes,
      },
      null,
      2,
    )}\n`,
  );
}

async function verifyFixture(fixture: ParityCase) {
  const root = fs.realpathSync.native(
    fs.mkdtempSync(path.join(os.tmpdir(), `omena-bundler-host-${fixture.id}-`)),
  );
  try {
    const materialized = new Map<string, string>();
    for (const source of fixture.sources) {
      const relativePath = source.stylePath.replace(/^\/workspace\//u, "");
      const filePath = path.join(root, relativePath);
      fs.mkdirSync(path.dirname(filePath), { recursive: true });
      fs.writeFileSync(filePath, source.styleSource, "utf8");
      materialized.set(source.stylePath, fs.realpathSync.native(filePath));
    }
    const targetPath = materialized.get(fixture.targetPath);
    assert.ok(targetPath, `missing target source for ${fixture.id}`);
    const interfacePath = path.join(root, "generated/module-interface.json");
    const declarationRoot = path.join(root, "generated/types");

    run(cliPath, [
      "modules",
      "emit",
      root,
      "--interface-file",
      interfacePath,
      "--declaration-dir",
      declarationRoot,
      "--json",
    ]);
    const bundle = JSON.parse(fs.readFileSync(interfacePath, "utf8")) as {
      readonly modules: readonly InterfaceModule[];
    };
    const module = bundle.modules.find((candidate) => candidate.stylePath === targetPath);
    assert.ok(module, `CLI module-interface artifact omitted ${fixture.id}`);
    const cliClassMap = classMapFromModule(module);

    const targetSource = fixture.sources.find((source) => source.stylePath === fixture.targetPath);
    assert.ok(targetSource);
    const otherPaths = fixture.sources
      .filter((source) => source.stylePath !== fixture.targetPath)
      .map((source) => materialized.get(source.stylePath))
      .filter((sourcePath): sourcePath is string => Boolean(sourcePath));
    const engine = createBoundaryEngine(fixture);
    const state = createOmenaBuildState({ cwd: root });
    const adapterOutput = await rebuildAndCache(
      targetPath,
      targetSource.styleSource,
      { cwd: root, configFile: false, engine, sources: otherPaths },
      state,
    );

    assert.deepEqual(canonicalRecord(adapterOutput.classMap), canonicalRecord(cliClassMap));
    assert.deepEqual(
      canonicalRecord(adapterOutput.classMap),
      canonicalRecord(fixture.expectedClassMap),
    );
    assert.notDeepEqual(
      canonicalRecord(fixture.legacyRegexMap),
      canonicalRecord(fixture.expectedClassMap),
    );
    const declarationPath = path.join(
      declarationRoot,
      path.relative(root, targetPath).concat(".d.ts"),
    );
    assert.equal(
      adapterOutput.typescriptDeclaration,
      fs.readFileSync(declarationPath, "utf8"),
      `typed export artifact drifted for ${fixture.id}`,
    );
    const namedExportCount = typecheckConsumer(root, targetPath, declarationPath, module);

    return {
      id: fixture.id,
      classMap: canonicalRecord(adapterOutput.classMap),
      namedExportCount,
      parity: true,
      typescriptConsumer: true,
    };
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
}

function createBoundaryEngine(fixture: ParityCase) {
  return {
    summarizeTransformBundleFromSourceJson: () =>
      JSON.stringify({
        plannedPassIds: ["composes-resolution", "css-modules-class-hashing"],
      }),
    buildStyleSourcesWithContextJson: () =>
      JSON.stringify({
        execution: {
          outputCss: fixture.emittedCss,
          executedPassIds: ["composes-resolution", "css-modules-class-hashing"],
        },
      }),
    bundlerHostCapabilitiesJson: () =>
      JSON.stringify({
        protocolVersion: "0",
        capabilities: ["semanticClassMap", "namedExports", "composesEdges"],
      }),
    resolveCssModuleForBundlerHostJson: (requestJson: string) => {
      const response = JSON.parse(
        capture(boundaryRunnerPath, ["bundler-host-resolve-module"], requestJson),
      ) as {
        classMap: Record<string, string>;
        namedExports: Record<string, string>;
      };
      if (injectRenamedDevClass && fixture.id === "css-local-class") {
        response.classMap.root = `${response.classMap.root} renamed`;
        response.namedExports.root = response.classMap.root;
      }
      return JSON.stringify(response);
    },
  };
}

function applyFixtureFault(fixture: ParityCase): ParityCase {
  if (!injectDroppedComposesEdge || fixture.id !== "css-imported-composes") return fixture;
  const sources = fixture.sources.map((source) => ({
    ...source,
    styleSource: source.styleSource.replace(
      /\s*composes:\s*base\s+from\s+["']\.\/base\.module\.css["'];/u,
      "",
    ),
  }));
  assert.notDeepEqual(sources, fixture.sources, "composes fault must alter the fixture source");
  return { ...fixture, sources };
}

function typecheckConsumer(
  root: string,
  targetPath: string,
  declarationPath: string,
  module: InterfaceModule,
) {
  const namedExports = [...module.classExports, ...module.icssExports]
    .map((entry) => entry.namedExport)
    .filter((name): name is string => Boolean(name))
    .toSorted();
  assert.ok(namedExports.length > 0, `fixture ${targetPath} must expose a named export`);
  const importPath = `./${path
    .relative(root, declarationPath)
    .replaceAll(path.sep, "/")
    .replace(/\.d\.ts$/u, "")}`;
  const defaultKey = module.classExports[0]?.name ?? module.icssExports[0]?.name;
  assert.ok(defaultKey, `fixture ${targetPath} must expose a default-map key`);
  const consumerPath = path.join(root, "consumer.ts");
  fs.writeFileSync(
    consumerPath,
    [
      `import styles, { ${namedExports.join(", ")} } from ${JSON.stringify(importPath)};`,
      `const defaultValue: string = styles[${JSON.stringify(defaultKey)}];`,
      ...namedExports.map((name) => `const ${name}Value: string = ${name};`),
      `void [defaultValue, ${namedExports.map((name) => `${name}Value`).join(", ")}];`,
      "",
    ].join("\n"),
    "utf8",
  );
  run("pnpm", [
    "exec",
    "tsc",
    "--ignoreConfig",
    "--noEmit",
    "--strict",
    "--moduleResolution",
    "Bundler",
    "--module",
    "ESNext",
    "--target",
    "ES2022",
    "--allowArbitraryExtensions",
    "--skipLibCheck",
    consumerPath,
  ]);
  return namedExports.length;
}

function classMapFromModule(module: InterfaceModule) {
  return Object.fromEntries([
    ...module.classExports.map((entry) => [entry.name, entry.emittedClasses.join(" ")] as const),
    ...module.icssExports.map((entry) => [entry.name, entry.value] as const),
  ]);
}

function canonicalRecord(value: Readonly<Record<string, string>>) {
  return Object.fromEntries(
    Object.entries(value).sort(([left], [right]) => left.localeCompare(right)),
  );
}

function run(command: string, args: readonly string[]) {
  const result = spawnSync(command, args, { cwd: repoRoot, encoding: "utf8" });
  assert.equal(result.status, 0, result.stderr || result.stdout);
}

function capture(command: string, args: readonly string[], input: string) {
  const result = spawnSync(command, args, { cwd: repoRoot, encoding: "utf8", input });
  assert.equal(result.status, 0, result.stderr || result.stdout);
  return result.stdout;
}
