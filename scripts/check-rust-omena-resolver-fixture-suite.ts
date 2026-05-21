import { spawnSync } from "node:child_process";

interface FixtureStep {
  readonly label: string;
  readonly command: string;
  readonly args: readonly string[];
}

const CARGO_MANIFEST = "rust/Cargo.toml";

const STEPS: readonly FixtureStep[] = [
  {
    label: "resolver package exports/imports and Sass pkg URLs",
    command: "cargo",
    args: [
      "test",
      "--manifest-path",
      CARGO_MANIFEST,
      "-p",
      "omena-resolver",
      "package",
      "--",
      "--nocapture",
    ],
  },
  {
    label: "resolver TypeScript path mappings",
    command: "cargo",
    args: [
      "test",
      "--manifest-path",
      CARGO_MANIFEST,
      "-p",
      "omena-resolver",
      "tsconfig",
      "--",
      "--nocapture",
    ],
  },
  {
    label: "resolver Vite/Webpack-style bundler aliases",
    command: "cargo",
    args: [
      "test",
      "--manifest-path",
      CARGO_MANIFEST,
      "-p",
      "omena-resolver",
      "bundler",
      "--",
      "--nocapture",
    ],
  },
  {
    label: "bridge product style-resolution inputs",
    command: "cargo",
    args: [
      "test",
      "--manifest-path",
      CARGO_MANIFEST,
      "-p",
      "omena-bridge",
      "style_resolution",
      "--",
      "--nocapture",
    ],
  },
  {
    label: "LSP product TypeScript path mappings",
    command: "cargo",
    args: [
      "test",
      "--manifest-path",
      CARGO_MANIFEST,
      "-p",
      "omena-lsp-server",
      "tsconfig",
      "--",
      "--nocapture",
    ],
  },
  {
    label: "LSP product Vite/Webpack bundler aliases",
    command: "cargo",
    args: [
      "test",
      "--manifest-path",
      CARGO_MANIFEST,
      "-p",
      "omena-lsp-server",
      "bundler",
      "--",
      "--nocapture",
    ],
  },
  {
    label: "LSP product package manifests and imports",
    command: "cargo",
    args: [
      "test",
      "--manifest-path",
      CARGO_MANIFEST,
      "-p",
      "omena-lsp-server",
      "package",
      "--",
      "--nocapture",
    ],
  },
];

for (const step of STEPS) {
  process.stdout.write(`== ${step.label} ==\n`);
  const result = spawnSync(step.command, [...step.args], {
    cwd: process.cwd(),
    stdio: "inherit",
    shell: false,
  });

  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

process.stdout.write(
  [
    "validated omena resolver fixture suite:",
    "node=package-bare-and-root-fallback",
    "typescript=paths-longest-prefix-and-extends",
    "package=exports-imports-conditions-patterns",
    "sass=node-package-importer-pkg-url-ordering",
    "bundler=vite-webpack-aliases",
    "lsp=product-path-fixtures",
  ].join(" ") + "\n",
);
