import path from "node:path";

export interface OmenaLspServerInvocation {
  readonly command: string;
  readonly args: readonly string[];
}

export function resolveOmenaLspServerInvocation(
  env: NodeJS.ProcessEnv = process.env,
): OmenaLspServerInvocation {
  const explicitPath = env.OMENA_LSP_SERVER_PATH?.trim();
  if (explicitPath) {
    return {
      command: path.resolve(process.cwd(), explicitPath),
      args: [],
    };
  }

  return {
    command: "cargo",
    args: [
      "run",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-lsp-server",
      "--bin",
      "omena-lsp-server",
      "--quiet",
    ],
  };
}
